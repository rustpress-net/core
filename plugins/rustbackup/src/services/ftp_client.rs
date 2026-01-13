//! FTP Client
//!
//! FTP/FTPS storage provider implementation.

use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use chrono::{DateTime, Utc, TimeZone};

use suppaftp::{AsyncNativeTlsFtpStream, AsyncNativeTlsConnector, FtpError};
use suppaftp::async_native_tls::TlsConnector;
use suppaftp::types::FileType;

use crate::models::storage::FTPStorageConfig;
use super::storage::{StorageFile, StorageError};

/// FTP client wrapper
pub struct FTPClient {
    config: FTPStorageConfig,
}

impl FTPClient {
    /// Create a new FTP client from config
    pub fn new(config: &FTPStorageConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Connect to FTP server
    async fn connect(&self) -> Result<AsyncNativeTlsFtpStream, StorageError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);

        let mut ftp_stream = AsyncNativeTlsFtpStream::connect(&addr).await
            .map_err(|e| StorageError::ConnectionFailed(format!("FTP connection failed: {}", e)))?;

        // Switch to secure mode if SSL is enabled
        if self.config.use_ssl {
            let tls_connector = TlsConnector::new();
            let connector = AsyncNativeTlsConnector::from(tls_connector);
            ftp_stream = ftp_stream
                .into_secure(connector, &self.config.host).await
                .map_err(|e| StorageError::ConnectionFailed(format!("FTPS upgrade failed: {}", e)))?;
        }

        // Login
        let password = self.config.password.as_deref().unwrap_or("");
        ftp_stream.login(&self.config.username, password).await
            .map_err(|e| StorageError::ConnectionFailed(format!("FTP login failed: {}", e)))?;

        // Set passive mode if configured
        if self.config.passive_mode {
            ftp_stream.set_mode(suppaftp::Mode::Passive);
        } else {
            ftp_stream.set_mode(suppaftp::Mode::Active);
        }

        // Set binary mode for file transfers
        ftp_stream.transfer_type(FileType::Binary).await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to set binary mode: {}", e)))?;

        // Change to base path if specified
        if !self.config.path.is_empty() && self.config.path != "/" {
            // Try to create and change to directory
            let path = self.config.path.trim_matches('/');
            for part in path.split('/') {
                if !part.is_empty() {
                    // Try to create directory (ignore error if exists)
                    let _ = ftp_stream.mkdir(part).await;
                    ftp_stream.cwd(part).await
                        .map_err(|e| StorageError::ConnectionFailed(format!("Failed to change to directory '{}': {}", part, e)))?;
                }
            }
        }

        Ok(ftp_stream)
    }

    /// Test FTP connection
    pub async fn test_connection(&self) -> Result<Option<i64>, StorageError> {
        let mut ftp = self.connect().await?;

        // Get current directory to verify connection
        let _pwd = ftp.pwd().await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to get working directory: {}", e)))?;

        // Try to quit gracefully
        let _ = ftp.quit().await;

        Ok(None) // FTP doesn't report available space in a standard way
    }

    /// Upload a file to FTP server
    pub async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<String, StorageError> {
        let mut ftp = self.connect().await?;

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

        // Create any necessary subdirectories in remote path
        let remote_path = Path::new(remote_name);
        if let Some(parent) = remote_path.parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() && parent_str != "." {
                for part in parent_str.split('/').chain(parent_str.split('\\')) {
                    if !part.is_empty() {
                        let _ = ftp.mkdir(part).await;
                        let _ = ftp.cwd(part).await;
                    }
                }
                // Go back to root
                ftp.cwd("/").await
                    .map_err(|e| StorageError::UploadFailed(format!("Failed to return to root: {}", e)))?;

                // Navigate to base path again
                if !self.config.path.is_empty() && self.config.path != "/" {
                    let base_path = self.config.path.trim_matches('/');
                    for part in base_path.split('/') {
                        if !part.is_empty() {
                            ftp.cwd(part).await
                                .map_err(|e| StorageError::UploadFailed(format!("Failed to navigate: {}", e)))?;
                        }
                    }
                }
            }
        }

        // Upload file
        let filename = remote_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| remote_name.to_string());

        // Navigate to parent directory if needed
        if let Some(parent) = remote_path.parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() && parent_str != "." {
                for part in parent_str.split('/').chain(parent_str.split('\\')) {
                    if !part.is_empty() {
                        ftp.cwd(part).await
                            .map_err(|e| StorageError::UploadFailed(format!("Failed to navigate to {}: {}", part, e)))?;
                    }
                }
            }
        }

        ftp.put_file(&filename, &mut buffer.as_slice()).await
            .map_err(|e| StorageError::UploadFailed(format!("FTP upload failed: {}", e)))?;

        let _ = ftp.quit().await;

        let remote_url = format!("ftp://{}:{}/{}/{}",
            self.config.host,
            self.config.port,
            self.config.path.trim_matches('/'),
            remote_name
        );

        Ok(remote_url)
    }

    /// Download a file from FTP server
    pub async fn download(&self, remote_path: &str, local_path: &Path) -> Result<(), StorageError> {
        let mut ftp = self.connect().await?;

        // Navigate to parent directory if path contains subdirectories
        let path = Path::new(remote_path);
        let filename = if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() && parent_str != "." {
                for part in parent_str.split('/').chain(parent_str.split('\\')) {
                    if !part.is_empty() {
                        ftp.cwd(part).await
                            .map_err(|e| StorageError::DownloadFailed(format!("Failed to navigate to {}: {}", part, e)))?;
                    }
                }
            }
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| remote_path.to_string())
        } else {
            remote_path.to_string()
        };

        // Download file
        let data = ftp.retr_as_buffer(&filename).await
            .map_err(|e| StorageError::DownloadFailed(format!("FTP download failed: {}", e)))?;

        let _ = ftp.quit().await;

        // Write to local file
        tokio::fs::write(local_path, data.into_inner()).await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Delete a file from FTP server
    pub async fn delete(&self, remote_path: &str) -> Result<(), StorageError> {
        let mut ftp = self.connect().await?;

        // Navigate to parent directory if path contains subdirectories
        let path = Path::new(remote_path);
        let filename = if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() && parent_str != "." {
                for part in parent_str.split('/').chain(parent_str.split('\\')) {
                    if !part.is_empty() {
                        ftp.cwd(part).await
                            .map_err(|e| StorageError::DeleteFailed(format!("Failed to navigate to {}: {}", part, e)))?;
                    }
                }
            }
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| remote_path.to_string())
        } else {
            remote_path.to_string()
        };

        ftp.rm(&filename).await
            .map_err(|e| StorageError::DeleteFailed(format!("FTP delete failed: {}", e)))?;

        let _ = ftp.quit().await;

        Ok(())
    }

    /// List files in FTP directory
    pub async fn list_files(&self, prefix: Option<&str>) -> Result<Vec<StorageFile>, StorageError> {
        let mut ftp = self.connect().await?;

        // Navigate to prefix directory if specified
        if let Some(p) = prefix {
            for part in p.split('/') {
                if !part.is_empty() {
                    ftp.cwd(part).await
                        .map_err(|e| StorageError::ListFailed(format!("Failed to navigate to {}: {}", part, e)))?;
                }
            }
        }

        // Get current path for building full paths
        let current_path = ftp.pwd().await
            .map_err(|e| StorageError::ListFailed(format!("Failed to get current directory: {}", e)))?;

        // List files with details
        let list = ftp.nlst(None).await
            .map_err(|e| StorageError::ListFailed(format!("FTP list failed: {}", e)))?;

        let mut files = Vec::new();

        for entry in list {
            // Skip directories (. and ..)
            if entry == "." || entry == ".." {
                continue;
            }

            // Try to get file size
            let size = ftp.size(&entry).await.unwrap_or(0) as i64;

            // Try to get modification time
            let modified = ftp.mdtm(&entry).await
                .ok()
                .and_then(|dt| Some(dt.and_utc()));

            let full_path = format!("ftp://{}:{}{}/{}",
                self.config.host,
                self.config.port,
                current_path.trim_end_matches('/'),
                entry
            );

            files.push(StorageFile {
                name: entry,
                path: full_path,
                size,
                modified,
            });
        }

        let _ = ftp.quit().await;

        Ok(files)
    }

    /// Check if a file exists on FTP server
    pub async fn exists(&self, remote_path: &str) -> Result<bool, StorageError> {
        let mut ftp = self.connect().await?;

        // Navigate to parent directory if path contains subdirectories
        let path = Path::new(remote_path);
        let filename = if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() && parent_str != "." {
                for part in parent_str.split('/').chain(parent_str.split('\\')) {
                    if !part.is_empty() {
                        match ftp.cwd(part).await {
                            Ok(_) => {}
                            Err(_) => {
                                let _ = ftp.quit().await;
                                return Ok(false);
                            }
                        }
                    }
                }
            }
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| remote_path.to_string())
        } else {
            remote_path.to_string()
        };

        // Try to get file size to check if it exists
        let exists = ftp.size(&filename).await.is_ok();

        let _ = ftp.quit().await;

        Ok(exists)
    }

    /// Get file metadata from FTP server
    pub async fn get_metadata(&self, remote_path: &str) -> Result<StorageFile, StorageError> {
        let mut ftp = self.connect().await?;

        // Navigate to parent directory if path contains subdirectories
        let path = Path::new(remote_path);
        let filename = if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() && parent_str != "." {
                for part in parent_str.split('/').chain(parent_str.split('\\')) {
                    if !part.is_empty() {
                        ftp.cwd(part).await
                            .map_err(|e| StorageError::ListFailed(format!("Failed to navigate to {}: {}", part, e)))?;
                    }
                }
            }
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| remote_path.to_string())
        } else {
            remote_path.to_string()
        };

        let size = ftp.size(&filename).await
            .map_err(|e| StorageError::ListFailed(format!("Failed to get file size: {}", e)))? as i64;

        let modified = ftp.mdtm(&filename).await
            .ok()
            .and_then(|dt| Some(dt.and_utc()));

        let _ = ftp.quit().await;

        let full_path = format!("ftp://{}:{}/{}/{}",
            self.config.host,
            self.config.port,
            self.config.path.trim_matches('/'),
            remote_path
        );

        Ok(StorageFile {
            name: filename,
            path: full_path,
            size,
            modified,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ftp_client_creation() {
        let config = FTPStorageConfig {
            host: "ftp.example.com".to_string(),
            port: 21,
            username: "testuser".to_string(),
            password: Some("testpass".to_string()),
            path: "/backups".to_string(),
            use_ssl: false,
            passive_mode: true,
        };

        let client = FTPClient::new(&config);
        assert_eq!(client.config.host, "ftp.example.com");
        assert_eq!(client.config.port, 21);
    }
}
