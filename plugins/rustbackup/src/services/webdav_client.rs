//! WebDAV Client
//!
//! WebDAV storage provider implementation using reqwest.

use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use chrono::{DateTime, Utc};

use reqwest::{Client, Method, StatusCode};
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::models::storage::WebDAVStorageConfig;
use super::storage::{StorageFile, StorageError};

/// WebDAV client wrapper
pub struct WebDAVClient {
    client: Client,
    config: WebDAVStorageConfig,
}

impl WebDAVClient {
    /// Create a new WebDAV client from config
    pub fn new(config: &WebDAVStorageConfig) -> Result<Self, StorageError> {
        let client = Client::builder()
            .build()
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    /// Get the full URL for a given path
    fn get_url(&self, path: &str) -> String {
        let base_url = self.config.url.trim_end_matches('/');
        let prefix = self.config.path_prefix.as_deref().unwrap_or("").trim_matches('/');
        let path = path.trim_start_matches('/');

        if prefix.is_empty() {
            if path.is_empty() {
                format!("{}/", base_url)
            } else {
                format!("{}/{}", base_url, path)
            }
        } else {
            if path.is_empty() {
                format!("{}/{}/", base_url, prefix)
            } else {
                format!("{}/{}/{}", base_url, prefix, path)
            }
        }
    }

    /// Build a request with authentication
    fn build_request(&self, method: Method, url: &str) -> reqwest::RequestBuilder {
        let mut builder = self.client.request(method, url);

        if let Some(ref password) = self.config.password {
            builder = builder.basic_auth(&self.config.username, Some(password));
        } else {
            builder = builder.basic_auth(&self.config.username, Option::<&str>::None);
        }

        builder
    }

    /// Test WebDAV connection
    pub async fn test_connection(&self) -> Result<Option<i64>, StorageError> {
        let url = self.get_url("");

        // Use PROPFIND to check if we can access the directory
        let response = self.build_request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .header("Depth", "0")
            .header("Content-Type", "application/xml")
            .body(r#"<?xml version="1.0" encoding="utf-8"?>
                <propfind xmlns="DAV:">
                    <prop>
                        <resourcetype/>
                    </prop>
                </propfind>"#)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("WebDAV connection failed: {}", e)))?;

        if response.status().is_success() || response.status() == StatusCode::MULTI_STATUS {
            Ok(None)
        } else if response.status() == StatusCode::UNAUTHORIZED {
            Err(StorageError::ConnectionFailed("Authentication failed".to_string()))
        } else {
            Err(StorageError::ConnectionFailed(format!(
                "WebDAV server returned status: {}",
                response.status()
            )))
        }
    }

    /// Upload a file to WebDAV server
    pub async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<String, StorageError> {
        // Read file into memory
        let mut file = File::open(local_path).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to open file: {}", e)))?;

        let metadata = file.metadata().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to get file metadata: {}", e)))?;

        let file_size = metadata.len();

        // Read file contents
        let mut buffer = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut buffer).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to read file: {}", e)))?;

        // Ensure parent directories exist
        let remote_path = Path::new(remote_name);
        if let Some(parent) = remote_path.parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() && parent_str != "." {
                let mut current_path = String::new();
                for part in parent_str.split('/') {
                    if !part.is_empty() {
                        if current_path.is_empty() {
                            current_path = part.to_string();
                        } else {
                            current_path = format!("{}/{}", current_path, part);
                        }
                        // Try to create directory (MKCOL)
                        let _ = self.create_directory(&current_path).await;
                    }
                }
            }
        }

        let url = self.get_url(remote_name);

        // Determine content type based on file extension
        let content_type = Self::get_content_type(local_path);

        let response = self.build_request(Method::PUT, &url)
            .header("Content-Type", content_type)
            .body(buffer)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("WebDAV upload failed: {}", e)))?;

        if response.status().is_success() || response.status() == StatusCode::CREATED || response.status() == StatusCode::NO_CONTENT {
            Ok(url)
        } else {
            Err(StorageError::UploadFailed(format!(
                "WebDAV upload failed with status: {}",
                response.status()
            )))
        }
    }

    /// Create a directory on WebDAV server
    async fn create_directory(&self, path: &str) -> Result<(), StorageError> {
        let url = self.get_url(path);

        let response = self.build_request(Method::from_bytes(b"MKCOL").unwrap(), &url)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to create directory: {}", e)))?;

        // 201 Created, 405 Method Not Allowed (already exists), or 409 Conflict (parent doesn't exist but we ignore)
        if response.status().is_success()
            || response.status() == StatusCode::METHOD_NOT_ALLOWED
            || response.status() == StatusCode::CONFLICT {
            Ok(())
        } else {
            Err(StorageError::UploadFailed(format!(
                "Failed to create directory, status: {}",
                response.status()
            )))
        }
    }

    /// Get content type based on file extension
    fn get_content_type(path: &Path) -> &'static str {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "gz" | "gzip" => "application/gzip",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "sql" => "application/sql",
            "json" => "application/json",
            "xml" => "application/xml",
            _ => "application/octet-stream",
        }
    }

    /// Download a file from WebDAV server
    pub async fn download(&self, remote_path: &str, local_path: &Path) -> Result<(), StorageError> {
        let url = self.get_url(remote_path);

        let response = self.build_request(Method::GET, &url)
            .send()
            .await
            .map_err(|e| StorageError::DownloadFailed(format!("WebDAV download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(StorageError::DownloadFailed(format!(
                "WebDAV download failed with status: {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to read response: {}", e)))?;

        // Write to local file
        tokio::fs::write(local_path, bytes).await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Delete a file from WebDAV server
    pub async fn delete(&self, remote_path: &str) -> Result<(), StorageError> {
        let url = self.get_url(remote_path);

        let response = self.build_request(Method::DELETE, &url)
            .send()
            .await
            .map_err(|e| StorageError::DeleteFailed(format!("WebDAV delete failed: {}", e)))?;

        if response.status().is_success() || response.status() == StatusCode::NO_CONTENT {
            Ok(())
        } else if response.status() == StatusCode::NOT_FOUND {
            Ok(()) // Already deleted
        } else {
            Err(StorageError::DeleteFailed(format!(
                "WebDAV delete failed with status: {}",
                response.status()
            )))
        }
    }

    /// List files in WebDAV directory
    pub async fn list_files(&self, prefix: Option<&str>) -> Result<Vec<StorageFile>, StorageError> {
        let url = self.get_url(prefix.unwrap_or(""));

        // Use PROPFIND with Depth: 1 to list directory contents
        let response = self.build_request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(r#"<?xml version="1.0" encoding="utf-8"?>
                <propfind xmlns="DAV:">
                    <prop>
                        <displayname/>
                        <getcontentlength/>
                        <getlastmodified/>
                        <resourcetype/>
                    </prop>
                </propfind>"#)
            .send()
            .await
            .map_err(|e| StorageError::ListFailed(format!("WebDAV list failed: {}", e)))?;

        if !response.status().is_success() && response.status() != StatusCode::MULTI_STATUS {
            return Err(StorageError::ListFailed(format!(
                "WebDAV list failed with status: {}",
                response.status()
            )));
        }

        let body = response.text().await
            .map_err(|e| StorageError::ListFailed(format!("Failed to read response: {}", e)))?;

        self.parse_propfind_response(&body, &url)
    }

    /// Parse PROPFIND XML response
    fn parse_propfind_response(&self, xml: &str, base_url: &str) -> Result<Vec<StorageFile>, StorageError> {
        let mut files = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut current_href = String::new();
        let mut current_name = String::new();
        let mut current_size: i64 = 0;
        let mut current_modified: Option<DateTime<Utc>> = None;
        let mut is_collection = false;
        let mut in_response = false;
        let mut in_href = false;
        let mut in_displayname = false;
        let mut in_contentlength = false;
        let mut in_lastmodified = false;
        let mut in_resourcetype = false;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    match local_name.as_ref() {
                        b"response" => {
                            in_response = true;
                            current_href.clear();
                            current_name.clear();
                            current_size = 0;
                            current_modified = None;
                            is_collection = false;
                        }
                        b"href" if in_response => in_href = true,
                        b"displayname" if in_response => in_displayname = true,
                        b"getcontentlength" if in_response => in_contentlength = true,
                        b"getlastmodified" if in_response => in_lastmodified = true,
                        b"resourcetype" if in_response => in_resourcetype = true,
                        b"collection" if in_resourcetype => is_collection = true,
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    match local_name.as_ref() {
                        b"response" if in_response => {
                            in_response = false;

                            // Skip collections (directories) and the base directory itself
                            if !is_collection && !current_href.is_empty() {
                                // Check if this is not the base directory
                                let normalized_base = base_url.trim_end_matches('/');
                                let normalized_href = current_href.trim_end_matches('/');

                                if normalized_href != normalized_base && !normalized_href.is_empty() {
                                    // Extract filename from href if displayname is empty
                                    if current_name.is_empty() {
                                        current_name = current_href
                                            .split('/')
                                            .filter(|s| !s.is_empty())
                                            .last()
                                            .unwrap_or("")
                                            .to_string();

                                        // URL decode the name
                                        current_name = urlencoding::decode(&current_name)
                                            .unwrap_or(std::borrow::Cow::Borrowed(&current_name))
                                            .to_string();
                                    }

                                    if !current_name.is_empty() {
                                        files.push(StorageFile {
                                            name: current_name.clone(),
                                            path: if current_href.starts_with("http") {
                                                current_href.clone()
                                            } else {
                                                format!("{}{}", self.config.url.trim_end_matches('/'), current_href)
                                            },
                                            size: current_size,
                                            modified: current_modified,
                                        });
                                    }
                                }
                            }
                        }
                        b"href" => in_href = false,
                        b"displayname" => in_displayname = false,
                        b"getcontentlength" => in_contentlength = false,
                        b"getlastmodified" => in_lastmodified = false,
                        b"resourcetype" => in_resourcetype = false,
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if in_href {
                        current_href = text;
                    } else if in_displayname {
                        current_name = text;
                    } else if in_contentlength {
                        current_size = text.parse().unwrap_or(0);
                    } else if in_lastmodified {
                        // Parse RFC 2822 date format
                        current_modified = DateTime::parse_from_rfc2822(&text)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(StorageError::ListFailed(format!("XML parse error: {}", e)));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(files)
    }

    /// Check if a file exists on WebDAV server
    pub async fn exists(&self, remote_path: &str) -> Result<bool, StorageError> {
        let url = self.get_url(remote_path);

        let response = self.build_request(Method::HEAD, &url)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("WebDAV check failed: {}", e)))?;

        Ok(response.status().is_success())
    }

    /// Get file metadata from WebDAV server
    pub async fn get_metadata(&self, remote_path: &str) -> Result<StorageFile, StorageError> {
        let url = self.get_url(remote_path);

        let response = self.build_request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .header("Depth", "0")
            .header("Content-Type", "application/xml")
            .body(r#"<?xml version="1.0" encoding="utf-8"?>
                <propfind xmlns="DAV:">
                    <prop>
                        <displayname/>
                        <getcontentlength/>
                        <getlastmodified/>
                    </prop>
                </propfind>"#)
            .send()
            .await
            .map_err(|e| StorageError::ListFailed(format!("WebDAV metadata failed: {}", e)))?;

        if !response.status().is_success() && response.status() != StatusCode::MULTI_STATUS {
            return Err(StorageError::ListFailed(format!(
                "WebDAV metadata failed with status: {}",
                response.status()
            )));
        }

        let body = response.text().await
            .map_err(|e| StorageError::ListFailed(format!("Failed to read response: {}", e)))?;

        let files = self.parse_propfind_response(&body, &url)?;

        files.into_iter().next().ok_or_else(|| {
            StorageError::ListFailed("File not found".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webdav_client_creation() {
        let config = WebDAVStorageConfig {
            url: "https://webdav.example.com".to_string(),
            username: "testuser".to_string(),
            password: Some("testpass".to_string()),
            path_prefix: Some("/backups".to_string()),
        };

        let client = WebDAVClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_get_url() {
        let config = WebDAVStorageConfig {
            url: "https://webdav.example.com".to_string(),
            username: "testuser".to_string(),
            password: Some("testpass".to_string()),
            path_prefix: Some("/backups".to_string()),
        };

        let client = WebDAVClient::new(&config).unwrap();
        assert_eq!(
            client.get_url("test.tar.gz"),
            "https://webdav.example.com/backups/test.tar.gz"
        );
    }

    #[test]
    fn test_get_url_no_prefix() {
        let config = WebDAVStorageConfig {
            url: "https://webdav.example.com".to_string(),
            username: "testuser".to_string(),
            password: Some("testpass".to_string()),
            path_prefix: None,
        };

        let client = WebDAVClient::new(&config).unwrap();
        assert_eq!(
            client.get_url("test.tar.gz"),
            "https://webdav.example.com/test.tar.gz"
        );
    }
}
