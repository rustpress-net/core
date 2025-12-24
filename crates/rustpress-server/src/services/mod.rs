//! Server services module
//!
//! Contains service layers that coordinate between handlers and repositories.

pub mod theme_service;
pub mod render_service;
pub mod email_service;

pub use theme_service::{
    ThemeService, ThemeInfo, ThemeScanResult, ThemePreviewResult,
    ThemeInstallResult, ThemeValidationResult, ThemeSettingsInfo, DefaultThemeInfo,
};

pub use render_service::{
    RenderService, RenderedPage, SiteInfo, PostData, AuthorData,
    MediaData, TermData, MenuItemData, MenuData, WidgetData,
    WidgetAreaData, PaginationData, ArchiveData,
};

pub use email_service::{
    EmailService, EmailConfig, EmailTemplate, EmailResult, EmailError,
};
