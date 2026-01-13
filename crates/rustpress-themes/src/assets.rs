//! Theme Asset Compilation
//!
//! CSS/JS bundling, SCSS compilation, and asset optimization.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::fs;
use blake3::Hasher;

/// Asset compilation errors
#[derive(Debug, Error)]
pub enum AssetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSS compilation error: {0}")]
    CssCompilation(String),

    #[error("SCSS compilation error: {0}")]
    ScssCompilation(String),

    #[error("JavaScript error: {0}")]
    JavaScript(String),

    #[error("Asset not found: {0}")]
    NotFound(String),

    #[error("Invalid asset path: {0}")]
    InvalidPath(String),
}

/// Asset compiler configuration
#[derive(Debug, Clone)]
pub struct AssetConfig {
    pub minify: bool,
    pub sourcemaps: bool,
    pub bundle: bool,
    pub cache_bust: bool,
    pub output_dir: PathBuf,
    pub public_url: String,
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            minify: true,
            sourcemaps: true,
            bundle: true,
            cache_bust: true,
            output_dir: PathBuf::from("dist/assets"),
            public_url: "/assets".to_string(),
        }
    }
}

/// Asset compiler for themes
pub struct AssetCompiler {
    config: AssetConfig,
    /// Compiled asset cache
    cache: Arc<RwLock<HashMap<String, CompiledAsset>>>,
    /// Asset manifest
    manifest: Arc<RwLock<AssetManifest>>,
}

/// Compiled asset information
#[derive(Debug, Clone)]
pub struct CompiledAsset {
    pub original_path: PathBuf,
    pub output_path: PathBuf,
    pub hash: String,
    pub size: u64,
    pub mime_type: String,
    pub compiled_at: chrono::DateTime<chrono::Utc>,
    pub dependencies: Vec<String>,
}

/// Asset manifest for cache busting
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AssetManifest {
    pub entries: HashMap<String, ManifestEntry>,
    pub generated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Manifest entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ManifestEntry {
    pub file: String,
    pub hash: String,
    pub integrity: String,
    pub size: u64,
}

impl AssetCompiler {
    pub fn new(config: AssetConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            manifest: Arc::new(RwLock::new(AssetManifest::default())),
        }
    }

    /// Compile all assets in a directory
    pub async fn compile_all(&self, source_dir: &Path) -> Result<Vec<CompiledAsset>, AssetError> {
        let mut results = Vec::new();

        // Compile CSS files
        let css_files = self.find_files(source_dir, &["css"]).await?;
        for file in css_files {
            let compiled = self.compile_css(&file).await?;
            results.push(compiled);
        }

        // Compile SCSS files
        let scss_files = self.find_files(source_dir, &["scss", "sass"]).await?;
        for file in scss_files {
            let compiled = self.compile_scss(&file).await?;
            results.push(compiled);
        }

        // Process JavaScript files
        let js_files = self.find_files(source_dir, &["js"]).await?;
        for file in js_files {
            let compiled = self.compile_js(&file).await?;
            results.push(compiled);
        }

        // Update manifest
        self.update_manifest(&results).await?;

        Ok(results)
    }

    async fn find_files(&self, dir: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, AssetError> {
        let mut files = Vec::new();

        if !dir.exists() {
            return Ok(files);
        }

        for entry in walkdir::WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if extensions.iter().any(|e| *e == ext_str) {
                        // Skip partials (files starting with _)
                        if let Some(name) = path.file_name() {
                            if !name.to_string_lossy().starts_with('_') {
                                files.push(path.to_path_buf());
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Compile a CSS file
    pub async fn compile_css(&self, path: &Path) -> Result<CompiledAsset, AssetError> {
        let content = fs::read_to_string(path).await?;
        let compiled = self.process_css(&content)?;

        let hash = self.hash_content(&compiled);
        let output_path = self.get_output_path(path, &hash, "css");

        // Write compiled CSS
        fs::create_dir_all(output_path.parent().unwrap()).await?;
        fs::write(&output_path, &compiled).await?;

        let asset = CompiledAsset {
            original_path: path.to_path_buf(),
            output_path: output_path.clone(),
            hash: hash.clone(),
            size: compiled.len() as u64,
            mime_type: "text/css".to_string(),
            compiled_at: chrono::Utc::now(),
            dependencies: Vec::new(),
        };

        // Add to cache
        self.cache.write().insert(
            path.to_string_lossy().to_string(),
            asset.clone(),
        );

        Ok(asset)
    }

    fn process_css(&self, content: &str) -> Result<String, AssetError> {
        use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
        use lightningcss::targets::Targets;

        let stylesheet = StyleSheet::parse(content, ParserOptions::default())
            .map_err(|e| AssetError::CssCompilation(format!("{:?}", e)))?;

        // Use default targets for broad browser support
        let targets = Targets::default();

        let mut stylesheet = stylesheet;

        if self.config.minify {
            stylesheet.minify(MinifyOptions {
                targets: targets.clone(),
                ..Default::default()
            })
            .map_err(|e| AssetError::CssCompilation(format!("{:?}", e)))?;
        }

        let result = stylesheet.to_css(PrinterOptions {
            targets,
            minify: self.config.minify,
            ..Default::default()
        })
        .map_err(|e| AssetError::CssCompilation(format!("{:?}", e)))?;

        Ok(result.code)
    }

    /// Compile SCSS to CSS
    pub async fn compile_scss(&self, path: &Path) -> Result<CompiledAsset, AssetError> {
        let content = fs::read_to_string(path).await?;

        // Use grass for SCSS compilation
        let options = grass::Options::default()
            .style(if self.config.minify {
                grass::OutputStyle::Compressed
            } else {
                grass::OutputStyle::Expanded
            });

        let css = grass::from_string(content, &options)
            .map_err(|e| AssetError::ScssCompilation(e.to_string()))?;

        // Further process with lightningcss for autoprefixing
        let processed = self.process_css(&css)?;

        let hash = self.hash_content(&processed);
        let output_path = self.get_output_path(path, &hash, "css");

        fs::create_dir_all(output_path.parent().unwrap()).await?;
        fs::write(&output_path, &processed).await?;

        let asset = CompiledAsset {
            original_path: path.to_path_buf(),
            output_path: output_path.clone(),
            hash: hash.clone(),
            size: processed.len() as u64,
            mime_type: "text/css".to_string(),
            compiled_at: chrono::Utc::now(),
            dependencies: self.extract_scss_imports(path).await?,
        };

        self.cache.write().insert(
            path.to_string_lossy().to_string(),
            asset.clone(),
        );

        Ok(asset)
    }

    async fn extract_scss_imports(&self, path: &Path) -> Result<Vec<String>, AssetError> {
        let content = fs::read_to_string(path).await?;
        let mut imports = Vec::new();

        // Simple regex-based import extraction
        let import_regex = regex::Regex::new(r#"@import\s+["']([^"']+)["']"#).unwrap();
        let use_regex = regex::Regex::new(r#"@use\s+["']([^"']+)["']"#).unwrap();

        for cap in import_regex.captures_iter(&content) {
            imports.push(cap[1].to_string());
        }

        for cap in use_regex.captures_iter(&content) {
            imports.push(cap[1].to_string());
        }

        Ok(imports)
    }

    /// Compile/process JavaScript
    pub async fn compile_js(&self, path: &Path) -> Result<CompiledAsset, AssetError> {
        let content = fs::read_to_string(path).await?;

        // Basic minification (in production, you'd use a proper JS minifier)
        let processed = if self.config.minify {
            self.minify_js(&content)
        } else {
            content
        };

        let hash = self.hash_content(&processed);
        let output_path = self.get_output_path(path, &hash, "js");

        fs::create_dir_all(output_path.parent().unwrap()).await?;
        fs::write(&output_path, &processed).await?;

        let asset = CompiledAsset {
            original_path: path.to_path_buf(),
            output_path: output_path.clone(),
            hash: hash.clone(),
            size: processed.len() as u64,
            mime_type: "application/javascript".to_string(),
            compiled_at: chrono::Utc::now(),
            dependencies: Vec::new(),
        };

        self.cache.write().insert(
            path.to_string_lossy().to_string(),
            asset.clone(),
        );

        Ok(asset)
    }

    fn minify_js(&self, content: &str) -> String {
        // Basic JavaScript minification
        // In production, use a proper minifier like swc or terser
        let mut result = String::with_capacity(content.len());
        let mut prev_char = '\0';
        let mut in_string = false;
        let mut string_char = '\0';
        let mut in_single_comment = false;
        let mut in_multi_comment = false;
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            if in_single_comment {
                if c == '\n' {
                    in_single_comment = false;
                    if !result.ends_with(' ') && !result.ends_with('\n') {
                        result.push(' ');
                    }
                }
                continue;
            }

            if in_multi_comment {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    in_multi_comment = false;
                    if !result.ends_with(' ') {
                        result.push(' ');
                    }
                }
                continue;
            }

            if in_string {
                result.push(c);
                if c == string_char && prev_char != '\\' {
                    in_string = false;
                }
                prev_char = c;
                continue;
            }

            match c {
                '"' | '\'' | '`' => {
                    in_string = true;
                    string_char = c;
                    result.push(c);
                }
                '/' => {
                    if chars.peek() == Some(&'/') {
                        chars.next();
                        in_single_comment = true;
                    } else if chars.peek() == Some(&'*') {
                        chars.next();
                        in_multi_comment = true;
                    } else {
                        result.push(c);
                    }
                }
                ' ' | '\t' | '\n' | '\r' => {
                    // Collapse whitespace
                    if !result.is_empty() && !result.ends_with(' ') && !result.ends_with('{')
                        && !result.ends_with('}') && !result.ends_with(';') && !result.ends_with(',')
                        && !result.ends_with('(') && !result.ends_with(')') {
                        // Check next non-whitespace
                        let next = chars.clone().find(|c| !c.is_whitespace());
                        if let Some(next_c) = next {
                            if !matches!(next_c, '{' | '}' | ';' | ',' | '(' | ')') {
                                result.push(' ');
                            }
                        }
                    }
                }
                _ => {
                    result.push(c);
                }
            }
            prev_char = c;
        }

        result.trim().to_string()
    }

    fn hash_content(&self, content: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        hash.to_hex()[..8].to_string()
    }

    fn get_output_path(&self, original: &Path, hash: &str, ext: &str) -> PathBuf {
        let stem = original.file_stem().unwrap().to_string_lossy();

        if self.config.cache_bust {
            self.config.output_dir.join(format!("{}.{}.{}", stem, hash, ext))
        } else {
            self.config.output_dir.join(format!("{}.{}", stem, ext))
        }
    }

    async fn update_manifest(&self, assets: &[CompiledAsset]) -> Result<(), AssetError> {
        let mut manifest = self.manifest.write();

        for asset in assets {
            let key = asset.original_path.to_string_lossy().to_string();
            let file = asset.output_path.file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            manifest.entries.insert(key, ManifestEntry {
                file,
                hash: asset.hash.clone(),
                integrity: format!("sha256-{}", asset.hash),
                size: asset.size,
            });
        }

        manifest.generated_at = Some(chrono::Utc::now());

        // Write manifest file
        let manifest_path = self.config.output_dir.join("manifest.json");
        let content = serde_json::to_string_pretty(&*manifest)
            .map_err(|e| AssetError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        fs::write(manifest_path, content).await?;

        Ok(())
    }

    /// Get asset URL for template
    pub fn get_asset_url(&self, original_path: &str) -> Option<String> {
        self.manifest.read().entries.get(original_path).map(|entry| {
            format!("{}/{}", self.config.public_url, entry.file)
        })
    }

    /// Get asset with integrity hash
    pub fn get_asset_with_integrity(&self, original_path: &str) -> Option<(String, String)> {
        self.manifest.read().entries.get(original_path).map(|entry| {
            (
                format!("{}/{}", self.config.public_url, entry.file),
                entry.integrity.clone(),
            )
        })
    }

    /// Bundle multiple CSS files
    pub async fn bundle_css(&self, files: &[PathBuf], output_name: &str) -> Result<CompiledAsset, AssetError> {
        let mut combined = String::new();

        for file in files {
            let content = fs::read_to_string(file).await?;
            combined.push_str(&format!("/* {} */\n", file.display()));
            combined.push_str(&content);
            combined.push('\n');
        }

        let processed = self.process_css(&combined)?;
        let hash = self.hash_content(&processed);
        let output_path = self.config.output_dir.join(if self.config.cache_bust {
            format!("{}.{}.css", output_name, hash)
        } else {
            format!("{}.css", output_name)
        });

        fs::create_dir_all(output_path.parent().unwrap()).await?;
        fs::write(&output_path, &processed).await?;

        Ok(CompiledAsset {
            original_path: PathBuf::from(output_name),
            output_path,
            hash,
            size: processed.len() as u64,
            mime_type: "text/css".to_string(),
            compiled_at: chrono::Utc::now(),
            dependencies: files.iter().map(|f| f.to_string_lossy().to_string()).collect(),
        })
    }

    /// Bundle multiple JS files
    pub async fn bundle_js(&self, files: &[PathBuf], output_name: &str) -> Result<CompiledAsset, AssetError> {
        let mut combined = String::new();

        for file in files {
            let content = fs::read_to_string(file).await?;
            combined.push_str(&format!("/* {} */\n", file.display()));
            combined.push_str(&content);
            combined.push_str(";\n");
        }

        let processed = if self.config.minify {
            self.minify_js(&combined)
        } else {
            combined
        };

        let hash = self.hash_content(&processed);
        let output_path = self.config.output_dir.join(if self.config.cache_bust {
            format!("{}.{}.js", output_name, hash)
        } else {
            format!("{}.js", output_name)
        });

        fs::create_dir_all(output_path.parent().unwrap()).await?;
        fs::write(&output_path, &processed).await?;

        Ok(CompiledAsset {
            original_path: PathBuf::from(output_name),
            output_path,
            hash,
            size: processed.len() as u64,
            mime_type: "application/javascript".to_string(),
            compiled_at: chrono::Utc::now(),
            dependencies: files.iter().map(|f| f.to_string_lossy().to_string()).collect(),
        })
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Get cached asset
    pub fn get_cached(&self, path: &str) -> Option<CompiledAsset> {
        self.cache.read().get(path).cloned()
    }
}

/// Asset watcher for development
pub struct AssetWatcher {
    compiler: Arc<AssetCompiler>,
    watch_paths: Vec<PathBuf>,
}

impl AssetWatcher {
    pub fn new(compiler: Arc<AssetCompiler>) -> Self {
        Self {
            compiler,
            watch_paths: Vec::new(),
        }
    }

    pub fn watch(&mut self, path: PathBuf) {
        self.watch_paths.push(path);
    }

    /// Start watching for changes
    pub async fn start(&self) -> Result<(), AssetError> {
        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
        use tokio::sync::mpsc;

        let (tx, mut rx) = mpsc::channel(100);
        let compiler = self.compiler.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| AssetError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        for path in &self.watch_paths {
            watcher.watch(path, RecursiveMode::Recursive)
                .map_err(|e| AssetError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        }

        // Process events
        while let Some(event) = rx.recv().await {
            if let notify::EventKind::Modify(_) | notify::EventKind::Create(_) = event.kind {
                for path in event.paths {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        match ext_str.as_str() {
                            "css" => {
                                let _ = compiler.compile_css(&path).await;
                            }
                            "scss" | "sass" => {
                                let _ = compiler.compile_scss(&path).await;
                            }
                            "js" => {
                                let _ = compiler.compile_js(&path).await;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Inline critical CSS
pub struct CriticalCssInliner {
    max_size: usize,
}

impl CriticalCssInliner {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Inline critical CSS into HTML
    pub fn inline(&self, html: &str, critical_css: &str) -> String {
        if critical_css.len() > self.max_size {
            return html.to_string();
        }

        let style_tag = format!(
            "<style id=\"critical-css\">{}</style>",
            critical_css
        );

        // Insert before </head>
        if let Some(pos) = html.to_lowercase().find("</head>") {
            let mut result = String::with_capacity(html.len() + style_tag.len());
            result.push_str(&html[..pos]);
            result.push_str(&style_tag);
            result.push_str(&html[pos..]);
            result
        } else {
            html.to_string()
        }
    }

    /// Add preload for non-critical CSS
    pub fn add_preload(&self, html: &str, css_url: &str) -> String {
        let preload = format!(
            r#"<link rel="preload" href="{}" as="style" onload="this.onload=null;this.rel='stylesheet'">"#,
            css_url
        );
        let noscript = format!(
            r#"<noscript><link rel="stylesheet" href="{}"></noscript>"#,
            css_url
        );

        // Insert before </head>
        if let Some(pos) = html.to_lowercase().find("</head>") {
            let mut result = String::with_capacity(html.len() + preload.len() + noscript.len());
            result.push_str(&html[..pos]);
            result.push_str(&preload);
            result.push_str(&noscript);
            result.push_str(&html[pos..]);
            result
        } else {
            html.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_css_compilation() {
        let dir = tempdir().unwrap();
        let config = AssetConfig {
            output_dir: dir.path().to_path_buf(),
            ..Default::default()
        };

        let compiler = AssetCompiler::new(config);

        // Create test CSS file
        let css_path = dir.path().join("test.css");
        fs::write(&css_path, "body { color: red; }").await.unwrap();

        let result = compiler.compile_css(&css_path).await.unwrap();
        assert!(result.output_path.exists());
        assert_eq!(result.mime_type, "text/css");
    }

    #[test]
    fn test_js_minification() {
        let compiler = AssetCompiler::new(AssetConfig::default());

        let js = r#"
            // Comment
            function hello() {
                console.log("Hello");
            }
        "#;

        let minified = compiler.minify_js(js);
        assert!(!minified.contains("// Comment"));
        assert!(minified.contains("function"));
    }

    #[test]
    fn test_critical_css_inliner() {
        let inliner = CriticalCssInliner::new(10000);

        let html = "<html><head></head><body></body></html>";
        let css = "body { color: red; }";

        let result = inliner.inline(html, css);
        assert!(result.contains("<style id=\"critical-css\">"));
        assert!(result.contains("body { color: red; }"));
    }
}
