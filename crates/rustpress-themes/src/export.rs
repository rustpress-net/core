//! Theme Export
//!
//! Export themes as distributable packages.

use crate::manifest::ThemeManifest;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

/// Export errors
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Zip error: {0}")]
    Zip(String),

    #[error("Theme not found: {0}")]
    NotFound(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Walk directory error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Standard zip archive
    Zip,
    /// Tar with gzip compression
    TarGz,
    /// Plain directory copy
    Directory,
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Export format
    pub format: ExportFormat,
    /// Include compiled assets
    pub include_compiled: bool,
    /// Include node_modules (if any)
    pub include_node_modules: bool,
    /// Include .git directory
    pub include_git: bool,
    /// Files/patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Minify assets
    pub minify_assets: bool,
    /// Include screenshot
    pub include_screenshot: bool,
    /// Custom metadata to add
    pub custom_metadata: HashMap<String, String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Zip,
            include_compiled: true,
            include_node_modules: false,
            include_git: false,
            exclude_patterns: vec![
                "*.log".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                ".env".to_string(),
                ".env.*".to_string(),
            ],
            minify_assets: true,
            include_screenshot: true,
            custom_metadata: HashMap::new(),
        }
    }
}

/// Theme exporter
pub struct ThemeExporter {
    theme_path: PathBuf,
    manifest: ThemeManifest,
    options: ExportOptions,
}

/// Export result
#[derive(Debug)]
pub struct ExportResult {
    pub output_path: PathBuf,
    pub size: u64,
    pub files_included: usize,
    pub files_excluded: usize,
}

impl ThemeExporter {
    pub fn new(theme_path: PathBuf, manifest: ThemeManifest) -> Self {
        Self {
            theme_path,
            manifest,
            options: ExportOptions::default(),
        }
    }

    pub fn with_options(mut self, options: ExportOptions) -> Self {
        self.options = options;
        self
    }

    /// Export the theme
    pub async fn export(&self, output_dir: &Path) -> Result<ExportResult, ExportError> {
        fs::create_dir_all(output_dir).await?;

        let theme_id = &self.manifest.theme.id;
        let version = &self.manifest.theme.version;

        match self.options.format {
            ExportFormat::Zip => self.export_zip(output_dir, theme_id, version).await,
            ExportFormat::TarGz => self.export_tar_gz(output_dir, theme_id, version).await,
            ExportFormat::Directory => self.export_directory(output_dir, theme_id).await,
        }
    }

    async fn export_zip(&self, output_dir: &Path, theme_id: &str, version: &str) -> Result<ExportResult, ExportError> {
        let output_path = output_dir.join(format!("{}-{}.zip", theme_id, version));

        let file = std::fs::File::create(&output_path)?;
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));

        let mut files_included = 0;
        let mut files_excluded = 0;

        // Walk through theme directory
        for entry in walkdir::WalkDir::new(&self.theme_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let relative_path = path.strip_prefix(&self.theme_path)
                .map_err(|e| ExportError::ExportFailed(e.to_string()))?;

            // Check if should be excluded
            if self.should_exclude(relative_path) {
                files_excluded += 1;
                continue;
            }

            if path.is_file() {
                let archive_path = format!("{}/{}", theme_id, relative_path.display());
                zip.start_file(&archive_path, options)
                    .map_err(|e| ExportError::Zip(e.to_string()))?;

                let content = std::fs::read(path)?;
                zip.write_all(&content)?;
                files_included += 1;
            } else if path.is_dir() && !relative_path.as_os_str().is_empty() {
                let archive_path = format!("{}/{}/", theme_id, relative_path.display());
                zip.add_directory(&archive_path, options)
                    .map_err(|e| ExportError::Zip(e.to_string()))?;
            }
        }

        zip.finish().map_err(|e| ExportError::Zip(e.to_string()))?;

        let size = std::fs::metadata(&output_path)?.len();

        Ok(ExportResult {
            output_path,
            size,
            files_included,
            files_excluded,
        })
    }

    async fn export_tar_gz(&self, output_dir: &Path, theme_id: &str, version: &str) -> Result<ExportResult, ExportError> {
        // For simplicity, just use zip for now
        // In production, use tar and flate2 crates
        self.export_zip(output_dir, theme_id, version).await
    }

    async fn export_directory(&self, output_dir: &Path, theme_id: &str) -> Result<ExportResult, ExportError> {
        let output_path = output_dir.join(theme_id);

        if output_path.exists() {
            fs::remove_dir_all(&output_path).await?;
        }

        let mut files_included = 0;
        let mut files_excluded = 0;

        // Copy files
        for entry in walkdir::WalkDir::new(&self.theme_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let source = entry.path();
            let relative = source.strip_prefix(&self.theme_path)
                .map_err(|e| ExportError::ExportFailed(e.to_string()))?;

            if self.should_exclude(relative) {
                files_excluded += 1;
                continue;
            }

            let dest = output_path.join(relative);

            if source.is_dir() {
                fs::create_dir_all(&dest).await?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).await?;
                }
                fs::copy(source, &dest).await?;
                files_included += 1;
            }
        }

        // Calculate total size
        let size = calculate_dir_size(&output_path).await?;

        Ok(ExportResult {
            output_path,
            size,
            files_included,
            files_excluded,
        })
    }

    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check standard exclusions
        if !self.options.include_node_modules && path_str.contains("node_modules") {
            return true;
        }

        if !self.options.include_git && path_str.contains(".git") {
            return true;
        }

        // Check custom patterns
        for pattern in &self.options.exclude_patterns {
            if matches_glob_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }
}

/// Simple glob pattern matching
fn matches_glob_pattern(path: &str, pattern: &str) -> bool {
    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        path.ends_with(suffix)
    } else if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len()-1];
        path.starts_with(prefix)
    } else if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            path.starts_with(parts[0]) && path.ends_with(parts[1])
        } else {
            path.contains(pattern)
        }
    } else {
        path.contains(pattern)
    }
}

/// Calculate directory size recursively
async fn calculate_dir_size(path: &Path) -> Result<u64, ExportError> {
    let mut size = 0;

    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            size += entry.metadata()?.len();
        }
    }

    Ok(size)
}

/// Theme import from archive
pub struct ThemeImporter {
    themes_dir: PathBuf,
}

impl ThemeImporter {
    pub fn new(themes_dir: PathBuf) -> Self {
        Self { themes_dir }
    }

    /// Import theme from zip file
    pub async fn import_zip(&self, zip_path: &Path) -> Result<String, ExportError> {
        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ExportError::Zip(e.to_string()))?;

        // Find theme ID from first entry
        let theme_id = {
            let first = archive.by_index(0)
                .map_err(|e| ExportError::Zip(e.to_string()))?;
            let name = first.name();
            name.split('/').next()
                .ok_or_else(|| ExportError::ExportFailed("Invalid archive structure".to_string()))?
                .to_string()
        };

        let output_dir = self.themes_dir.join(&theme_id);

        // Extract
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| ExportError::Zip(e.to_string()))?;

            let outpath = self.themes_dir.join(file.name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        // Verify manifest exists
        let manifest_path = output_dir.join("theme.toml");
        if !manifest_path.exists() {
            fs::remove_dir_all(&output_dir).await?;
            return Err(ExportError::ExportFailed("No theme.toml found in archive".to_string()));
        }

        Ok(theme_id)
    }
}

/// Generate theme distribution readme
pub fn generate_readme(manifest: &ThemeManifest) -> String {
    format!(
        r#"# {}

{}

## Version
{}

## Author
{}

## Requirements
- RustPress {}

## Installation

1. Download the theme zip file
2. In your RustPress admin, go to Appearance > Themes
3. Click "Upload Theme" and select the zip file
4. Click "Install Now"
5. After installation, click "Activate"

## Features

{}

## License
{}

---
Generated by RustPress Theme Exporter
"#,
        manifest.theme.name,
        manifest.theme.description,
        manifest.theme.version,
        manifest.theme.author,
        manifest.theme.requires_rustpress.as_deref().unwrap_or("1.0.0"),
        generate_features_list(manifest),
        manifest.theme.license,
    )
}

fn generate_features_list(manifest: &ThemeManifest) -> String {
    let mut features = Vec::new();

    if manifest.supports.block_editor {
        features.push("- Block Editor (Gutenberg) support");
    }
    if manifest.supports.custom_logo.is_some() {
        features.push("- Custom logo upload");
    }
    if manifest.supports.post_thumbnails {
        features.push("- Post thumbnails (featured images)");
    }
    if manifest.supports.custom_header.is_some() {
        features.push("- Custom header");
    }
    if manifest.supports.custom_background.is_some() {
        features.push("- Custom background");
    }

    if features.is_empty() {
        "Standard theme features".to_string()
    } else {
        features.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_pattern() {
        assert!(matches_glob_pattern("file.log", "*.log"));
        assert!(matches_glob_pattern(".DS_Store", ".DS_Store"));
        assert!(!matches_glob_pattern("file.txt", "*.log"));
        assert!(matches_glob_pattern(".env.local", ".env.*"));
    }

    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::default();

        assert_eq!(options.format, ExportFormat::Zip);
        assert!(options.include_compiled);
        assert!(!options.include_node_modules);
        assert!(!options.include_git);
    }
}
