//! Media library with folder organization
//!
//! Provides a hierarchical folder structure for organizing media files
//! with support for browsing, searching, and bulk operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use crate::{MediaError, MediaFolder, MediaItem, MediaResult, MediaType};

/// Media library service
pub struct MediaLibrary {
    pool: PgPool,
}

impl MediaLibrary {
    /// Create new media library service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Folder Operations
    // ========================================================================

    /// Create a new folder
    pub async fn create_folder(&self, name: &str, parent_id: Option<Uuid>) -> MediaResult<MediaFolder> {
        // Build path
        let path = if let Some(pid) = parent_id {
            let parent = self.get_folder(pid).await?;
            format!("{}/{}", parent.path, name)
        } else {
            format!("/{}", name)
        };

        let folder: MediaFolder = sqlx::query_as(
            r#"
            INSERT INTO media_folders (name, parent_id, path)
            VALUES ($1, $2, $3)
            RETURNING id, name, parent_id, path, created_at, updated_at
            "#
        )
        .bind(name)
        .bind(parent_id)
        .bind(&path)
        .fetch_one(&self.pool)
        .await?;

        Ok(folder)
    }

    /// Get folder by ID
    pub async fn get_folder(&self, id: Uuid) -> MediaResult<MediaFolder> {
        let folder: Option<MediaFolder> = sqlx::query_as(
            "SELECT id, name, parent_id, path, created_at, updated_at FROM media_folders WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        folder.ok_or(MediaError::NotFound(id))
    }

    /// List root folders
    pub async fn list_root_folders(&self) -> MediaResult<Vec<MediaFolder>> {
        let folders: Vec<MediaFolder> = sqlx::query_as(
            "SELECT id, name, parent_id, path, created_at, updated_at FROM media_folders WHERE parent_id IS NULL ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(folders)
    }

    /// List child folders
    pub async fn list_child_folders(&self, parent_id: Uuid) -> MediaResult<Vec<MediaFolder>> {
        let folders: Vec<MediaFolder> = sqlx::query_as(
            "SELECT id, name, parent_id, path, created_at, updated_at FROM media_folders WHERE parent_id = $1 ORDER BY name"
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(folders)
    }

    /// Rename folder
    pub async fn rename_folder(&self, id: Uuid, new_name: &str) -> MediaResult<MediaFolder> {
        // Get current folder
        let folder = self.get_folder(id).await?;

        // Calculate new path
        let new_path = if let Some(parent_id) = folder.parent_id {
            let parent = self.get_folder(parent_id).await?;
            format!("{}/{}", parent.path, new_name)
        } else {
            format!("/{}", new_name)
        };

        // Update folder and all child paths
        let updated: MediaFolder = sqlx::query_as(
            r#"
            UPDATE media_folders
            SET name = $2, path = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, parent_id, path, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(new_name)
        .bind(&new_path)
        .fetch_one(&self.pool)
        .await?;

        // Update child folder paths
        let old_path = folder.path;
        sqlx::query(
            "UPDATE media_folders SET path = $2 || substring(path from $3), updated_at = NOW() WHERE path LIKE $1"
        )
        .bind(format!("{}/%", old_path))
        .bind(&new_path)
        .bind((old_path.len() + 1) as i32)
        .execute(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Move folder to new parent
    pub async fn move_folder(&self, id: Uuid, new_parent_id: Option<Uuid>) -> MediaResult<MediaFolder> {
        let folder = self.get_folder(id).await?;

        // Prevent moving to self or descendant
        if let Some(new_pid) = new_parent_id {
            if new_pid == id {
                return Err(MediaError::ProcessingError("Cannot move folder to itself".to_string()));
            }

            let new_parent = self.get_folder(new_pid).await?;
            if new_parent.path.starts_with(&folder.path) {
                return Err(MediaError::ProcessingError("Cannot move folder to its descendant".to_string()));
            }
        }

        // Calculate new path
        let new_path = if let Some(new_pid) = new_parent_id {
            let new_parent = self.get_folder(new_pid).await?;
            format!("{}/{}", new_parent.path, folder.name)
        } else {
            format!("/{}", folder.name)
        };

        let old_path = folder.path.clone();

        // Update folder
        let updated: MediaFolder = sqlx::query_as(
            r#"
            UPDATE media_folders
            SET parent_id = $2, path = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, parent_id, path, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(new_parent_id)
        .bind(&new_path)
        .fetch_one(&self.pool)
        .await?;

        // Update child folder paths
        sqlx::query(
            "UPDATE media_folders SET path = $2 || substring(path from $3), updated_at = NOW() WHERE path LIKE $1"
        )
        .bind(format!("{}/%", old_path))
        .bind(&new_path)
        .bind((old_path.len() + 1) as i32)
        .execute(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Delete folder (moves contents to parent or root)
    pub async fn delete_folder(&self, id: Uuid, delete_contents: bool) -> MediaResult<()> {
        let folder = self.get_folder(id).await?;

        if delete_contents {
            // Delete all media in folder and subfolders
            sqlx::query(
                "DELETE FROM media_items WHERE folder_id IN (SELECT id FROM media_folders WHERE path LIKE $1 OR id = $2)"
            )
            .bind(format!("{}/%", folder.path))
            .bind(id)
            .execute(&self.pool)
            .await?;

            // Delete all subfolders
            sqlx::query("DELETE FROM media_folders WHERE path LIKE $1")
                .bind(format!("{}/%", folder.path))
                .execute(&self.pool)
                .await?;
        } else {
            // Move contents to parent folder
            sqlx::query("UPDATE media_items SET folder_id = $2 WHERE folder_id = $1")
                .bind(id)
                .bind(folder.parent_id)
                .execute(&self.pool)
                .await?;

            // Move subfolders to parent
            sqlx::query("UPDATE media_folders SET parent_id = $2 WHERE parent_id = $1")
                .bind(id)
                .bind(folder.parent_id)
                .execute(&self.pool)
                .await?;
        }

        // Delete the folder
        sqlx::query("DELETE FROM media_folders WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get folder tree (recursive structure)
    pub async fn get_folder_tree(&self) -> MediaResult<Vec<FolderNode>> {
        let folders: Vec<MediaFolder> = sqlx::query_as(
            "SELECT id, name, parent_id, path, created_at, updated_at FROM media_folders ORDER BY path"
        )
        .fetch_all(&self.pool)
        .await?;

        // Build tree structure
        let mut tree = Vec::new();
        let mut folder_map: std::collections::HashMap<Uuid, Vec<MediaFolder>> = std::collections::HashMap::new();

        // Group by parent
        for folder in folders {
            if let Some(parent_id) = folder.parent_id {
                folder_map.entry(parent_id).or_default().push(folder);
            } else {
                tree.push(folder);
            }
        }

        // Convert to FolderNode tree
        fn build_tree(folders: Vec<MediaFolder>, folder_map: &std::collections::HashMap<Uuid, Vec<MediaFolder>>) -> Vec<FolderNode> {
            folders.into_iter().map(|folder| {
                let children = folder_map.get(&folder.id).cloned().unwrap_or_default();
                FolderNode {
                    folder,
                    children: build_tree(children, folder_map),
                }
            }).collect()
        }

        Ok(build_tree(tree, &folder_map))
    }

    // ========================================================================
    // Media Operations
    // ========================================================================

    /// Get folder contents (folders + media)
    pub async fn get_folder_contents(&self, folder_id: Option<Uuid>) -> MediaResult<FolderContents> {
        let folders = if let Some(fid) = folder_id {
            self.list_child_folders(fid).await?
        } else {
            self.list_root_folders().await?
        };

        let media: Vec<MediaItem> = sqlx::query_as(
            r#"
            SELECT
                id, filename, title, alt_text, caption, description,
                media_type, mime_type, file_size, path, url, thumbnail_url,
                width, height, duration, file_hash, folder_id,
                metadata, uploaded_by, created_at, updated_at
            FROM media_items
            WHERE ($1::uuid IS NULL AND folder_id IS NULL) OR folder_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(folder_id)
        .fetch_all(&self.pool)
        .await?;

        let total_items = media.len();
        let total_size: i64 = media.iter().map(|m| m.file_size).sum();

        Ok(FolderContents {
            folders,
            media,
            total_items,
            total_size,
        })
    }

    /// Search media across all folders
    pub async fn search(
        &self,
        query: &str,
        media_type: Option<MediaType>,
        folder_id: Option<Uuid>,
        include_subfolders: bool,
        limit: i64,
        offset: i64,
    ) -> MediaResult<SearchResults> {
        let search_pattern = format!("%{}%", query);

        let media: Vec<MediaItem> = if include_subfolders && folder_id.is_some() {
            let folder = self.get_folder(folder_id.unwrap()).await?;
            let path_pattern = format!("{}%", folder.path);

            sqlx::query_as(
                r#"
                SELECT
                    m.id, m.filename, m.title, m.alt_text, m.caption, m.description,
                    m.media_type, m.mime_type, m.file_size, m.path, m.url, m.thumbnail_url,
                    m.width, m.height, m.duration, m.file_hash, m.folder_id,
                    m.metadata, m.uploaded_by, m.created_at, m.updated_at
                FROM media_items m
                LEFT JOIN media_folders f ON m.folder_id = f.id
                WHERE (m.title ILIKE $1 OR m.filename ILIKE $1 OR m.alt_text ILIKE $1)
                AND ($2::text IS NULL OR m.media_type::text = $2)
                AND (m.folder_id = $3 OR f.path LIKE $4)
                ORDER BY m.created_at DESC
                LIMIT $5 OFFSET $6
                "#
            )
            .bind(&search_pattern)
            .bind(media_type.map(|t| t.to_string()))
            .bind(folder_id)
            .bind(&path_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT
                    id, filename, title, alt_text, caption, description,
                    media_type, mime_type, file_size, path, url, thumbnail_url,
                    width, height, duration, file_hash, folder_id,
                    metadata, uploaded_by, created_at, updated_at
                FROM media_items
                WHERE (title ILIKE $1 OR filename ILIKE $1 OR alt_text ILIKE $1)
                AND ($2::text IS NULL OR media_type::text = $2)
                AND ($3::uuid IS NULL OR folder_id = $3)
                ORDER BY created_at DESC
                LIMIT $4 OFFSET $5
                "#
            )
            .bind(&search_pattern)
            .bind(media_type.map(|t| t.to_string()))
            .bind(folder_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        // Get total count (simplified - in production use COUNT query)
        let total = media.len() as i64;

        Ok(SearchResults {
            media,
            total,
            limit,
            offset,
        })
    }

    /// Bulk move media to folder
    pub async fn bulk_move(&self, media_ids: &[Uuid], folder_id: Option<Uuid>) -> MediaResult<u64> {
        let result = sqlx::query(
            "UPDATE media_items SET folder_id = $1, updated_at = NOW() WHERE id = ANY($2)"
        )
        .bind(folder_id)
        .bind(media_ids)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Bulk delete media
    pub async fn bulk_delete(&self, media_ids: &[Uuid]) -> MediaResult<u64> {
        let result = sqlx::query("DELETE FROM media_items WHERE id = ANY($1)")
            .bind(media_ids)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> MediaResult<StorageStats> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_count,
                COALESCE(SUM(file_size), 0) as total_size,
                COUNT(*) FILTER (WHERE media_type = 'image') as image_count,
                COUNT(*) FILTER (WHERE media_type = 'video') as video_count,
                COUNT(*) FILTER (WHERE media_type = 'audio') as audio_count,
                COUNT(*) FILTER (WHERE media_type = 'document') as document_count
            FROM media_items
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let folder_row = sqlx::query("SELECT COUNT(*) as count FROM media_folders")
            .fetch_one(&self.pool)
            .await?;

        Ok(StorageStats {
            total_files: row.get::<i64, _>("total_count") as u64,
            total_size: row.get::<i64, _>("total_size") as u64,
            image_count: row.get::<i64, _>("image_count") as u64,
            video_count: row.get::<i64, _>("video_count") as u64,
            audio_count: row.get::<i64, _>("audio_count") as u64,
            document_count: row.get::<i64, _>("document_count") as u64,
            folder_count: folder_row.get::<i64, _>("count") as u64,
        })
    }

    /// Get recent uploads
    pub async fn get_recent_uploads(&self, limit: i64) -> MediaResult<Vec<MediaItem>> {
        let media: Vec<MediaItem> = sqlx::query_as(
            r#"
            SELECT
                id, filename, title, alt_text, caption, description,
                media_type, mime_type, file_size, path, url, thumbnail_url,
                width, height, duration, file_hash, folder_id,
                metadata, uploaded_by, created_at, updated_at
            FROM media_items
            ORDER BY created_at DESC
            LIMIT $1
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(media)
    }

    /// Find duplicates by hash
    pub async fn find_duplicates(&self) -> MediaResult<Vec<DuplicateGroup>> {
        let groups: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT file_hash, COUNT(*) as count
            FROM media_items
            GROUP BY file_hash
            HAVING COUNT(*) > 1
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut duplicates = Vec::new();

        for (file_hash, count) in groups {
            let items: Vec<MediaItem> = sqlx::query_as(
                r#"
                SELECT
                    id, filename, title, alt_text, caption, description,
                    media_type, mime_type, file_size, path, url, thumbnail_url,
                    width, height, duration, file_hash, folder_id,
                    metadata, uploaded_by, created_at, updated_at
                FROM media_items
                WHERE file_hash = $1
                ORDER BY created_at
                "#
            )
            .bind(&file_hash)
            .fetch_all(&self.pool)
            .await?;

            duplicates.push(DuplicateGroup {
                file_hash,
                count: count as u64,
                items,
            });
        }

        Ok(duplicates)
    }
}

/// Folder tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderNode {
    pub folder: MediaFolder,
    pub children: Vec<FolderNode>,
}

/// Folder contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderContents {
    pub folders: Vec<MediaFolder>,
    pub media: Vec<MediaItem>,
    pub total_items: usize,
    pub total_size: i64,
}

/// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub media: Vec<MediaItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_files: u64,
    pub total_size: u64,
    pub image_count: u64,
    pub video_count: u64,
    pub audio_count: u64,
    pub document_count: u64,
    pub folder_count: u64,
}

impl StorageStats {
    /// Get human-readable total size
    pub fn human_total_size(&self) -> String {
        let size = self.total_size as f64;
        if size < 1024.0 {
            format!("{} B", self.total_size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Duplicate file group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub file_hash: String,
    pub count: u64,
    pub items: Vec<MediaItem>,
}
