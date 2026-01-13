//! SFTP Client
//!
//! SFTP storage provider implementation using russh.

use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use chrono::{DateTime, Utc};

use russh::client::{self, Config, Handle, Handler};
use russh::keys::key::PublicKey;
use russh_sftp::client::SftpSession;

use crate::models::storage::SFTPStorageConfig;
use super::storage::{StorageFile, StorageError};

/// SFTP client wrapper
pub struct SFTPClient {
    config: SFTPStorageConfig,
}

/// SSH client handler for russh
struct SshHandler;

#[async_trait::async_trait]
impl Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // In production, you should verify the server's host key
        // For now, we accept all keys (similar to StrictHostKeyChecking=no)
        Ok(true)
    }
}

impl SFTPClient {
    /// Create a new SFTP client from config
    pub fn new(config: &SFTPStorageConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Connect to SFTP server and return session
    async fn connect(&self) -> Result<SftpSession, StorageError> {
        let config = Config::default();
        let config = Arc::new(config);

        let addr = format!("{}:{}", self.config.host, self.config.port);

        // Connect to SSH server
        let mut session = client::connect(config, &addr, SshHandler)
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("SSH connection failed: {}", e)))?;

        // Authenticate
        let authenticated = if let Some(ref private_key) = self.config.private_key {
            // Private key authentication
            let passphrase = self.config.private_key_passphrase.as_deref();

            // Parse the private key
            let key_pair = if let Some(pass) = passphrase {
                russh_keys::decode_secret_key(private_key, Some(pass))
                    .map_err(|e| StorageError::ConnectionFailed(format!("Failed to decode private key: {}", e)))?
            } else {
                russh_keys::decode_secret_key(private_key, None)
                    .map_err(|e| StorageError::ConnectionFailed(format!("Failed to decode private key: {}", e)))?
            };

            session
                .authenticate_publickey(&self.config.username, Arc::new(key_pair))
                .await
                .map_err(|e| StorageError::ConnectionFailed(format!("Public key authentication failed: {}", e)))?
        } else if let Some(ref password) = self.config.password {
            // Password authentication
            session
                .authenticate_password(&self.config.username, password)
                .await
                .map_err(|e| StorageError::ConnectionFailed(format!("Password authentication failed: {}", e)))?
        } else {
            return Err(StorageError::ConnectionFailed(
                "No authentication method provided (password or private key required)".to_string()
            ));
        };

        if !authenticated {
            return Err(StorageError::ConnectionFailed("Authentication failed".to_string()));
        }

        // Open SFTP channel
        let channel = session
            .channel_open_session()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to open channel: {}", e)))?;

        // Request SFTP subsystem
        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to request SFTP subsystem: {}", e)))?;

        // Create SFTP session
        let sftp = SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to create SFTP session: {}", e)))?;

        Ok(sftp)
    }

    /// Get the full remote path for a given filename
    fn get_remote_path(&self, name: &str) -> String {
        let base = self.config.path.trim_matches('/');
        if base.is_empty() {
            format!("/{}", name)
        } else {
            format!("/{}/{}", base, name)
        }
    }

    /// Test SFTP connection
    pub async fn test_connection(&self) -> Result<Option<i64>, StorageError> {
        let sftp = self.connect().await?;

        // Try to read the base directory
        let base_path = if self.config.path.is_empty() {
            "/".to_string()
        } else {
            self.config.path.clone()
        };

        sftp.read_dir(&base_path)
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to read directory: {}", e)))?;

        // Close session
        sftp.close().await.ok();

        Ok(None) // SFTP doesn't easily report available space
    }

    /// Upload a file to SFTP server
    pub async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<String, StorageError> {
        let sftp = self.connect().await?;

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

        // Ensure base directory exists
        let base_path = self.config.path.trim_matches('/');
        if !base_path.is_empty() {
            let mut current_path = String::new();
            for part in base_path.split('/') {
                if !part.is_empty() {
                    current_path = format!("{}/{}", current_path, part);
                    // Try to create directory (ignore error if exists)
                    let _ = sftp.create_dir(&current_path).await;
                }
            }
        }

        // Create parent directories for the remote file if needed
        let remote_path = self.get_remote_path(remote_name);
        if let Some(parent) = Path::new(&remote_path).parent() {
            let parent_str = parent.to_string_lossy();
            if parent_str != "/" && !parent_str.is_empty() {
                let mut current_path = String::new();
                for part in parent_str.split('/') {
                    if !part.is_empty() {
                        current_path = format!("{}/{}", current_path, part);
                        let _ = sftp.create_dir(&current_path).await;
                    }
                }
            }
        }

        // Create/overwrite the file
        let mut remote_file = sftp
            .create(&remote_path)
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to create remote file: {}", e)))?;

        // Write data
        use tokio::io::AsyncWriteExt;
        remote_file
            .write_all(&buffer)
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to write to remote file: {}", e)))?;

        remote_file
            .shutdown()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to close remote file: {}", e)))?;

        sftp.close().await.ok();

        let remote_url = format!("sftp://{}:{}{}",
            self.config.host,
            self.config.port,
            remote_path
        );

        Ok(remote_url)
    }

    /// Download a file from SFTP server
    pub async fn download(&self, remote_path: &str, local_path: &Path) -> Result<(), StorageError> {
        let sftp = self.connect().await?;

        let full_remote_path = self.get_remote_path(remote_path);

        // Open remote file
        let mut remote_file = sftp
            .open(&full_remote_path)
            .await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to open remote file: {}", e)))?;

        // Read all data
        use tokio::io::AsyncReadExt;
        let mut buffer = Vec::new();
        remote_file
            .read_to_end(&mut buffer)
            .await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to read remote file: {}", e)))?;

        sftp.close().await.ok();

        // Write to local file
        tokio::fs::write(local_path, buffer).await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to write local file: {}", e)))?;

        Ok(())
    }

    /// Delete a file from SFTP server
    pub async fn delete(&self, remote_path: &str) -> Result<(), StorageError> {
        let sftp = self.connect().await?;

        let full_remote_path = self.get_remote_path(remote_path);

        sftp.remove_file(&full_remote_path)
            .await
            .map_err(|e| StorageError::DeleteFailed(format!("Failed to delete remote file: {}", e)))?;

        sftp.close().await.ok();

        Ok(())
    }

    /// List files in SFTP directory
    pub async fn list_files(&self, prefix: Option<&str>) -> Result<Vec<StorageFile>, StorageError> {
        let sftp = self.connect().await?;

        let search_path = match prefix {
            Some(p) => self.get_remote_path(p),
            None => {
                if self.config.path.is_empty() {
                    "/".to_string()
                } else {
                    format!("/{}", self.config.path.trim_matches('/'))
                }
            }
        };

        let entries = sftp
            .read_dir(&search_path)
            .await
            .map_err(|e| StorageError::ListFailed(format!("Failed to list directory: {}", e)))?;

        let mut files = Vec::new();

        for entry in entries {
            let filename = entry.file_name();

            // Skip . and ..
            if filename == "." || filename == ".." {
                continue;
            }

            // Get file attributes
            let attrs = entry.metadata();

            // Skip directories
            if attrs.is_dir() {
                continue;
            }

            let size = attrs.size.unwrap_or(0) as i64;

            let modified: Option<DateTime<Utc>> = attrs.mtime
                .and_then(|t| DateTime::from_timestamp(t as i64, 0));

            let full_path = format!("sftp://{}:{}{}/{}",
                self.config.host,
                self.config.port,
                search_path,
                filename
            );

            files.push(StorageFile {
                name: filename,
                path: full_path,
                size,
                modified,
            });
        }

        sftp.close().await.ok();

        Ok(files)
    }

    /// Check if a file exists on SFTP server
    pub async fn exists(&self, remote_path: &str) -> Result<bool, StorageError> {
        let sftp = self.connect().await?;

        let full_remote_path = self.get_remote_path(remote_path);

        let exists = match sftp.metadata(&full_remote_path).await {
            Ok(_) => true,
            Err(_) => false,
        };

        sftp.close().await.ok();

        Ok(exists)
    }

    /// Get file metadata from SFTP server
    pub async fn get_metadata(&self, remote_path: &str) -> Result<StorageFile, StorageError> {
        let sftp = self.connect().await?;

        let full_remote_path = self.get_remote_path(remote_path);

        let attrs = sftp
            .metadata(&full_remote_path)
            .await
            .map_err(|e| StorageError::ListFailed(format!("Failed to get file metadata: {}", e)))?;

        sftp.close().await.ok();

        let filename = Path::new(&full_remote_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| remote_path.to_string());

        let size = attrs.size.unwrap_or(0) as i64;

        let modified: Option<DateTime<Utc>> = attrs.mtime
            .and_then(|t| DateTime::from_timestamp(t as i64, 0));

        let full_url = format!("sftp://{}:{}{}",
            self.config.host,
            self.config.port,
            full_remote_path
        );

        Ok(StorageFile {
            name: filename,
            path: full_url,
            size,
            modified,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sftp_client_creation() {
        let config = SFTPStorageConfig {
            host: "sftp.example.com".to_string(),
            port: 22,
            username: "testuser".to_string(),
            password: Some("testpass".to_string()),
            private_key: None,
            private_key_passphrase: None,
            path: "/backups".to_string(),
        };

        let client = SFTPClient::new(&config);
        assert_eq!(client.config.host, "sftp.example.com");
        assert_eq!(client.config.port, 22);
    }

    #[test]
    fn test_get_remote_path() {
        let config = SFTPStorageConfig {
            host: "sftp.example.com".to_string(),
            port: 22,
            username: "testuser".to_string(),
            password: Some("testpass".to_string()),
            private_key: None,
            private_key_passphrase: None,
            path: "/backups".to_string(),
        };

        let client = SFTPClient::new(&config);
        assert_eq!(client.get_remote_path("test.tar.gz"), "/backups/test.tar.gz");
    }

    #[test]
    fn test_get_remote_path_empty_base() {
        let config = SFTPStorageConfig {
            host: "sftp.example.com".to_string(),
            port: 22,
            username: "testuser".to_string(),
            password: Some("testpass".to_string()),
            private_key: None,
            private_key_passphrase: None,
            path: "".to_string(),
        };

        let client = SFTPClient::new(&config);
        assert_eq!(client.get_remote_path("test.tar.gz"), "/test.tar.gz");
    }
}
