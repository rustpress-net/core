//! RustBuilder - Page Builder Plugin for RustPress
//!
//! A visual page builder for creating content with drag-and-drop blocks.

use async_trait::async_trait;
use rustpress_core::plugin::{Plugin, PluginInfo};
use rustpress_core::{AppContext, Result};
use semver::Version;

pub mod api;
pub mod builder;

pub use builder::*;

/// Plugin metadata
pub const PLUGIN_NAME: &str = "RustBuilder";
pub const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PLUGIN_ID: &str = "rustbuilder";

/// RustBuilder plugin struct
pub struct RustBuilderPlugin {
    info: PluginInfo,
}

impl RustBuilderPlugin {
    /// Create a new RustBuilder plugin instance
    pub fn new() -> Self {
        let mut info = PluginInfo::new(
            PLUGIN_ID,
            PLUGIN_NAME,
            Version::new(0, 1, 0),
        );
        info.description = "Visual page builder for creating content with drag-and-drop blocks".to_string();
        info.author = "RustPress Team".to_string();
        info.author_url = None;
        info.homepage = None;
        info.license = "MIT".to_string();
        info.tags = vec!["page-builder".to_string(), "visual-editor".to_string()];

        Self { info }
    }
}

impl Default for RustBuilderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for RustBuilderPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    async fn activate(&self, _ctx: &AppContext) -> Result<()> {
        // Plugin activation logic
        Ok(())
    }

    async fn deactivate(&self, _ctx: &AppContext) -> Result<()> {
        // Plugin deactivation logic
        Ok(())
    }
}
