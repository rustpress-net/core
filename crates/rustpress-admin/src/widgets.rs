//! Dashboard widgets for the admin panel

use serde::{Deserialize, Serialize};

/// Widget types available in the dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    /// Stats card with single value
    StatsCard,
    /// Line chart
    LineChart,
    /// Bar chart
    BarChart,
    /// Pie/doughnut chart
    PieChart,
    /// Progress bar
    ProgressBar,
    /// Activity feed
    ActivityFeed,
    /// Status indicators
    StatusList,
    /// Quick actions
    QuickActions,
    /// Table
    Table,
    /// Custom HTML
    Custom,
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    /// Widget ID
    pub id: String,

    /// Widget type
    pub widget_type: WidgetType,

    /// Widget title
    pub title: String,

    /// Widget size (1-4 columns)
    pub size: u8,

    /// Widget data source URL
    pub data_source: Option<String>,

    /// Refresh interval in seconds (0 = no refresh)
    pub refresh_interval: u64,

    /// Widget configuration
    pub config: serde_json::Value,

    /// Is widget visible
    pub visible: bool,

    /// Widget order
    pub order: i32,
}

impl Default for Widget {
    fn default() -> Self {
        Self {
            id: String::new(),
            widget_type: WidgetType::StatsCard,
            title: String::new(),
            size: 1,
            data_source: None,
            refresh_interval: 30,
            config: serde_json::json!({}),
            visible: true,
            order: 0,
        }
    }
}

/// Default dashboard layout
pub fn default_dashboard_widgets() -> Vec<Widget> {
    vec![
        // Row 1: Quick stats
        Widget {
            id: "total_posts".to_string(),
            widget_type: WidgetType::StatsCard,
            title: "Total Posts".to_string(),
            size: 1,
            data_source: Some("/admin/api/stats/posts".to_string()),
            config: serde_json::json!({
                "icon": "📝",
                "color": "blue"
            }),
            ..Default::default()
        },
        Widget {
            id: "total_users".to_string(),
            widget_type: WidgetType::StatsCard,
            title: "Total Users".to_string(),
            size: 1,
            data_source: Some("/admin/api/stats/users".to_string()),
            config: serde_json::json!({
                "icon": "👥",
                "color": "green"
            }),
            ..Default::default()
        },
        Widget {
            id: "views_today".to_string(),
            widget_type: WidgetType::StatsCard,
            title: "Views Today".to_string(),
            size: 1,
            data_source: Some("/admin/api/stats/views".to_string()),
            config: serde_json::json!({
                "icon": "👁️",
                "color": "yellow"
            }),
            ..Default::default()
        },
        Widget {
            id: "comments".to_string(),
            widget_type: WidgetType::StatsCard,
            title: "Comments".to_string(),
            size: 1,
            data_source: Some("/admin/api/stats/comments".to_string()),
            config: serde_json::json!({
                "icon": "💬",
                "color": "purple"
            }),
            ..Default::default()
        },

        // Row 2: System resources and status
        Widget {
            id: "system_resources".to_string(),
            widget_type: WidgetType::ProgressBar,
            title: "System Resources".to_string(),
            size: 2,
            data_source: Some("/admin/api/system/status".to_string()),
            config: serde_json::json!({
                "metrics": ["cpu", "memory", "disk"]
            }),
            ..Default::default()
        },
        Widget {
            id: "service_status".to_string(),
            widget_type: WidgetType::StatusList,
            title: "Service Status".to_string(),
            size: 2,
            data_source: Some("/admin/api/services/status".to_string()),
            config: serde_json::json!({
                "services": ["app", "database", "cache", "cdn"]
            }),
            ..Default::default()
        },

        // Row 3: Charts
        Widget {
            id: "traffic_chart".to_string(),
            widget_type: WidgetType::LineChart,
            title: "Traffic (Last 7 Days)".to_string(),
            size: 2,
            data_source: Some("/admin/api/analytics/traffic".to_string()),
            config: serde_json::json!({
                "period": "7d",
                "metrics": ["pageviews", "visitors"]
            }),
            ..Default::default()
        },
        Widget {
            id: "top_posts".to_string(),
            widget_type: WidgetType::Table,
            title: "Top Posts".to_string(),
            size: 2,
            data_source: Some("/admin/api/analytics/top-posts".to_string()),
            config: serde_json::json!({
                "limit": 5,
                "columns": ["title", "views", "comments"]
            }),
            ..Default::default()
        },

        // Row 4: Activity and actions
        Widget {
            id: "recent_activity".to_string(),
            widget_type: WidgetType::ActivityFeed,
            title: "Recent Activity".to_string(),
            size: 3,
            data_source: Some("/admin/api/activity".to_string()),
            config: serde_json::json!({
                "limit": 10
            }),
            ..Default::default()
        },
        Widget {
            id: "quick_actions".to_string(),
            widget_type: WidgetType::QuickActions,
            title: "Quick Actions".to_string(),
            size: 1,
            config: serde_json::json!({
                "actions": [
                    {"label": "New Post", "url": "/admin/posts/new", "icon": "➕"},
                    {"label": "Purge Cache", "action": "purge_cache", "icon": "🗑️"},
                    {"label": "Create Backup", "action": "create_backup", "icon": "💾"},
                    {"label": "View Site", "url": "/", "icon": "🔗", "target": "_blank"}
                ]
            }),
            ..Default::default()
        },
    ]
}

/// Widget renderer trait
pub trait WidgetRenderer {
    /// Render widget to HTML
    fn render(&self, widget: &Widget, data: &serde_json::Value) -> String;
}

/// Default widget renderer
pub struct DefaultWidgetRenderer;

impl WidgetRenderer for DefaultWidgetRenderer {
    fn render(&self, widget: &Widget, data: &serde_json::Value) -> String {
        match widget.widget_type {
            WidgetType::StatsCard => render_stats_card(widget, data),
            WidgetType::ProgressBar => render_progress_bar(widget, data),
            WidgetType::StatusList => render_status_list(widget, data),
            WidgetType::ActivityFeed => render_activity_feed(widget, data),
            WidgetType::QuickActions => render_quick_actions(widget, data),
            _ => format!("<div>Widget: {}</div>", widget.title),
        }
    }
}

fn render_stats_card(widget: &Widget, data: &serde_json::Value) -> String {
    let icon = widget.config.get("icon").and_then(|v| v.as_str()).unwrap_or("📊");
    let color = widget.config.get("color").and_then(|v| v.as_str()).unwrap_or("blue");
    let value = data.get("value").and_then(|v| v.as_u64()).unwrap_or(0);

    format!(r#"
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <div class="flex items-center">
                <div class="p-3 bg-{}-100 dark:bg-{}-900 rounded-full">
                    <span class="text-2xl">{}</span>
                </div>
                <div class="ml-4">
                    <p class="text-sm text-gray-500 dark:text-gray-400">{}</p>
                    <p class="text-2xl font-bold text-gray-800 dark:text-white">{}</p>
                </div>
            </div>
        </div>
    "#, color, color, icon, widget.title, value)
}

fn render_progress_bar(widget: &Widget, data: &serde_json::Value) -> String {
    let mut html = format!(r#"
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h3 class="text-lg font-semibold text-gray-800 dark:text-white mb-4">{}</h3>
            <div class="space-y-4">
    "#, widget.title);

    if let Some(metrics) = data.as_object() {
        for (name, value) in metrics {
            let percent = value.as_f64().unwrap_or(0.0);
            let color = if percent > 90.0 {
                "red"
            } else if percent > 70.0 {
                "yellow"
            } else {
                "green"
            };

            html.push_str(&format!(r#"
                <div>
                    <div class="flex justify-between mb-1">
                        <span class="text-sm text-gray-600 dark:text-gray-400">{}</span>
                        <span class="text-sm font-medium text-gray-800 dark:text-white">{:.1}%</span>
                    </div>
                    <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                        <div class="bg-{}-600 h-2 rounded-full" style="width: {}%"></div>
                    </div>
                </div>
            "#, name, percent, color, percent));
        }
    }

    html.push_str("</div></div>");
    html
}

fn render_status_list(widget: &Widget, data: &serde_json::Value) -> String {
    let mut html = format!(r#"
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h3 class="text-lg font-semibold text-gray-800 dark:text-white mb-4">{}</h3>
            <div class="space-y-3">
    "#, widget.title);

    if let Some(services) = data.as_object() {
        for (name, status) in services {
            let is_healthy = status.as_bool().unwrap_or(false);
            let (badge_class, badge_text) = if is_healthy {
                ("bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300", "Online")
            } else {
                ("bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300", "Offline")
            };

            html.push_str(&format!(r#"
                <div class="flex items-center justify-between">
                    <span class="text-gray-600 dark:text-gray-400">{}</span>
                    <span class="px-2 py-1 text-xs rounded-full {}">{}</span>
                </div>
            "#, name, badge_class, badge_text));
        }
    }

    html.push_str("</div></div>");
    html
}

fn render_activity_feed(widget: &Widget, data: &serde_json::Value) -> String {
    let mut html = format!(r#"
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h3 class="text-lg font-semibold text-gray-800 dark:text-white mb-4">{}</h3>
            <div class="space-y-4">
    "#, widget.title);

    if let Some(activities) = data.as_array() {
        for activity in activities.iter().take(10) {
            let description = activity.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let timestamp = activity.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");

            html.push_str(&format!(r#"
                <div class="flex items-center p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                    <div class="flex-1">
                        <p class="text-sm text-gray-800 dark:text-white">{}</p>
                        <p class="text-xs text-gray-500 dark:text-gray-400">{}</p>
                    </div>
                </div>
            "#, description, timestamp));
        }
    }

    html.push_str("</div></div>");
    html
}

fn render_quick_actions(widget: &Widget, _data: &serde_json::Value) -> String {
    let mut html = format!(r#"
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h3 class="text-lg font-semibold text-gray-800 dark:text-white mb-4">{}</h3>
            <div class="space-y-2">
    "#, widget.title);

    if let Some(actions) = widget.config.get("actions").and_then(|v| v.as_array()) {
        for action in actions {
            let label = action.get("label").and_then(|v| v.as_str()).unwrap_or("");
            let icon = action.get("icon").and_then(|v| v.as_str()).unwrap_or("▶");

            if let Some(url) = action.get("url").and_then(|v| v.as_str()) {
                let target = action.get("target").and_then(|v| v.as_str()).unwrap_or("");
                html.push_str(&format!(r#"
                    <a href="{}" target="{}" class="block w-full text-left px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600">
                        <span class="mr-2">{}</span> {}
                    </a>
                "#, url, target, icon, label));
            } else {
                let action_name = action.get("action").and_then(|v| v.as_str()).unwrap_or("");
                html.push_str(&format!(r#"
                    <button onclick="executeAction('{}')" class="block w-full text-left px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600">
                        <span class="mr-2">{}</span> {}
                    </button>
                "#, action_name, icon, label));
            }
        }
    }

    html.push_str("</div></div>");
    html
}
