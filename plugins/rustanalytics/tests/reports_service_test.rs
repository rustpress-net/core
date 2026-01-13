//! Reports Service Tests
//!
//! Comprehensive tests for the ReportService functionality.

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use rustanalytics::models::reports::*;
use rustanalytics::models::{DateRange, DateRangePreset, ReportFormat, ReportFrequency};
use rustanalytics::services::cache::CacheService;
use rustanalytics::services::client::GoogleAnalyticsClient;
use rustanalytics::services::reports::ReportService;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_db() -> Arc<dyn std::any::Any + Send + Sync> {
    Arc::new(())
}

async fn create_test_report_service() -> ReportService {
    let db = create_test_db();
    let cache = Arc::new(CacheService::new(db.clone(), 15));

    let client = Arc::new(
        GoogleAnalyticsClient::new("properties/12345".to_string(), None)
            .await
            .unwrap()
    );

    ReportService::new(client, cache, db)
}

fn sample_custom_report() -> CustomReport {
    let now = Utc::now();
    CustomReport {
        id: Uuid::new_v4(),
        name: "Test Report".to_string(),
        description: Some("A test report for unit testing".to_string()),
        metrics: vec![
            ReportMetric {
                id: "sessions".to_string(),
                name: "Sessions".to_string(),
                category: MetricCategory::Session,
                data_type: MetricDataType::Integer,
                aggregation: MetricAggregation::Total,
            },
            ReportMetric {
                id: "totalUsers".to_string(),
                name: "Total Users".to_string(),
                category: MetricCategory::User,
                data_type: MetricDataType::Integer,
                aggregation: MetricAggregation::Total,
            },
        ],
        dimensions: vec![
            ReportDimension {
                id: "date".to_string(),
                name: "Date".to_string(),
                category: DimensionCategory::Time,
            },
        ],
        filters: vec![],
        segments: vec![],
        date_range: DateRangePreset::Last30Days,
        chart_type: ChartType::Line,
        created_at: now,
        updated_at: now,
        created_by: Uuid::new_v4(),
        is_public: false,
        is_favorite: false,
    }
}

fn sample_report_with_filters() -> CustomReport {
    let mut report = sample_custom_report();
    report.filters = vec![
        ReportFilter {
            dimension: "country".to_string(),
            operator: FilterOperator::Equals,
            value: "United States".to_string(),
            case_sensitive: false,
        },
    ];
    report
}

fn sample_report_result() -> ReportResult {
    ReportResult {
        report_id: Uuid::new_v4(),
        date_range: DateRange::last_n_days(7),
        rows: vec![
            ReportRow {
                dimensions: vec!["2024-01-01".to_string()],
                metrics: vec![
                    ReportMetricValue {
                        metric_id: "sessions".to_string(),
                        value: serde_json::Value::String("1000".to_string()),
                        formatted_value: "1,000".to_string(),
                    },
                    ReportMetricValue {
                        metric_id: "totalUsers".to_string(),
                        value: serde_json::Value::String("500".to_string()),
                        formatted_value: "500".to_string(),
                    },
                ],
            },
            ReportRow {
                dimensions: vec!["2024-01-02".to_string()],
                metrics: vec![
                    ReportMetricValue {
                        metric_id: "sessions".to_string(),
                        value: serde_json::Value::String("1200".to_string()),
                        formatted_value: "1,200".to_string(),
                    },
                    ReportMetricValue {
                        metric_id: "totalUsers".to_string(),
                        value: serde_json::Value::String("600".to_string()),
                        formatted_value: "600".to_string(),
                    },
                ],
            },
        ],
        totals: Some(ReportRow {
            dimensions: vec![],
            metrics: vec![
                ReportMetricValue {
                    metric_id: "sessions".to_string(),
                    value: serde_json::Value::String("2200".to_string()),
                    formatted_value: "2,200".to_string(),
                },
                ReportMetricValue {
                    metric_id: "totalUsers".to_string(),
                    value: serde_json::Value::String("1100".to_string()),
                    formatted_value: "1,100".to_string(),
                },
            ],
        }),
        row_count: 2,
        sampling_info: None,
        generated_at: Utc::now(),
    }
}

fn sample_scheduled_report() -> ScheduledReport {
    ScheduledReport {
        id: Uuid::new_v4(),
        report_id: Uuid::new_v4(),
        name: "Weekly Summary".to_string(),
        frequency: ReportFrequency::Weekly,
        format: ReportFormat::Pdf,
        recipients: vec!["test@example.com".to_string()],
        include_comparison: true,
        enabled: true,
        next_run: Utc::now() + chrono::Duration::days(7),
        last_run: None,
        last_status: None,
        created_at: Utc::now(),
    }
}

// ============================================================================
// ReportService Creation Tests
// ============================================================================

#[tokio::test]
async fn test_report_service_creation() {
    let service = create_test_report_service().await;
    let debug_str = format!("{:?}", service);
    assert!(debug_str.contains("ReportService"));
}

// ============================================================================
// Report CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_list_reports_empty() {
    let service = create_test_report_service().await;
    let reports = service.list_reports(None).await.unwrap();
    assert!(reports.is_empty());
}

#[tokio::test]
async fn test_list_reports_with_user_id() {
    let service = create_test_report_service().await;
    let user_id = Uuid::new_v4();
    let reports = service.list_reports(Some(user_id)).await.unwrap();
    assert!(reports.is_empty());
}

#[tokio::test]
async fn test_get_report_not_found() {
    let service = create_test_report_service().await;
    let report = service.get_report(Uuid::new_v4()).await.unwrap();
    assert!(report.is_none());
}

#[tokio::test]
async fn test_create_report() {
    let service = create_test_report_service().await;
    let report = sample_custom_report();

    let created = service.create_report(report.clone()).await.unwrap();

    // ID should be different (new one generated)
    assert_ne!(created.id, report.id);
    assert_eq!(created.name, report.name);
    assert_eq!(created.description, report.description);
    assert_eq!(created.metrics.len(), report.metrics.len());
    assert_eq!(created.dimensions.len(), report.dimensions.len());
}

#[tokio::test]
async fn test_create_report_with_description() {
    let service = create_test_report_service().await;
    let mut report = sample_custom_report();
    report.description = Some("Custom description".to_string());

    let created = service.create_report(report).await.unwrap();
    assert_eq!(created.description, Some("Custom description".to_string()));
}

#[tokio::test]
async fn test_create_report_without_description() {
    let service = create_test_report_service().await;
    let mut report = sample_custom_report();
    report.description = None;

    let created = service.create_report(report).await.unwrap();
    assert!(created.description.is_none());
}

#[tokio::test]
async fn test_update_report() {
    let service = create_test_report_service().await;
    let report_id = Uuid::new_v4();
    let mut report = sample_custom_report();
    report.name = "Updated Report Name".to_string();

    let updated = service.update_report(report_id, report).await.unwrap();

    assert_eq!(updated.id, report_id);
    assert_eq!(updated.name, "Updated Report Name");
}

#[tokio::test]
async fn test_delete_report() {
    let service = create_test_report_service().await;
    let result = service.delete_report(Uuid::new_v4()).await.unwrap();
    assert!(result);
}

// ============================================================================
// Report Templates Tests
// ============================================================================

#[tokio::test]
async fn test_get_templates() {
    let service = create_test_report_service().await;
    let templates = service.get_templates();

    assert!(!templates.is_empty());
    assert!(templates.len() >= 5);

    // Check for specific templates
    assert!(templates.iter().any(|t| t.id == "audience_overview"));
    assert!(templates.iter().any(|t| t.id == "traffic_sources"));
    assert!(templates.iter().any(|t| t.id == "top_pages"));
    assert!(templates.iter().any(|t| t.id == "ecommerce_overview"));
    assert!(templates.iter().any(|t| t.id == "conversions"));
}

#[tokio::test]
async fn test_get_templates_categories() {
    let service = create_test_report_service().await;
    let templates = service.get_templates();

    // Check various categories exist
    assert!(templates.iter().any(|t| t.category == ReportTemplateCategory::Audience));
    assert!(templates.iter().any(|t| t.category == ReportTemplateCategory::Acquisition));
    assert!(templates.iter().any(|t| t.category == ReportTemplateCategory::Behavior));
    assert!(templates.iter().any(|t| t.category == ReportTemplateCategory::Ecommerce));
    assert!(templates.iter().any(|t| t.category == ReportTemplateCategory::Conversions));
}

#[tokio::test]
async fn test_get_templates_premium_flag() {
    let service = create_test_report_service().await;
    let templates = service.get_templates();

    // Check premium templates exist
    let premium_count = templates.iter().filter(|t| t.is_premium).count();
    let free_count = templates.iter().filter(|t| !t.is_premium).count();

    assert!(premium_count > 0);
    assert!(free_count > 0);
}

#[tokio::test]
async fn test_create_from_template_audience_overview() {
    let service = create_test_report_service().await;
    let user_id = Uuid::new_v4();

    let report = service.create_from_template("audience_overview", user_id).unwrap();

    assert_eq!(report.name, "Audience Overview");
    assert!(report.description.is_some());
    assert!(!report.metrics.is_empty());
    assert!(!report.dimensions.is_empty());
    assert_eq!(report.created_by, user_id);
    assert_eq!(report.chart_type, ChartType::Line);
}

#[tokio::test]
async fn test_create_from_template_traffic_sources() {
    let service = create_test_report_service().await;
    let user_id = Uuid::new_v4();

    let report = service.create_from_template("traffic_sources", user_id).unwrap();

    assert_eq!(report.name, "Traffic Sources");
    assert_eq!(report.chart_type, ChartType::Pie);
}

#[tokio::test]
async fn test_create_from_template_top_pages() {
    let service = create_test_report_service().await;
    let user_id = Uuid::new_v4();

    let report = service.create_from_template("top_pages", user_id).unwrap();

    assert_eq!(report.name, "Top Pages");
    assert_eq!(report.chart_type, ChartType::Bar);
}

#[tokio::test]
async fn test_create_from_template_not_found() {
    let service = create_test_report_service().await;
    let user_id = Uuid::new_v4();

    let result = service.create_from_template("nonexistent_template", user_id);
    assert!(result.is_err());
}

// ============================================================================
// Report Export Tests
// ============================================================================

#[tokio::test]
async fn test_export_report_csv() {
    let service = create_test_report_service().await;
    let report = sample_custom_report();
    let result = sample_report_result();

    let export = service.export_report(&report, &result, ReportFormat::Csv).await.unwrap();

    assert!(export.file_name.ends_with(".csv"));
    assert_eq!(export.content_type, "text/csv");
    assert!(export.size_bytes > 0);

    let content = String::from_utf8(export.data).unwrap();
    assert!(content.contains("Sessions"));
    assert!(content.contains("Total Users"));
}

#[tokio::test]
async fn test_export_report_json() {
    let service = create_test_report_service().await;
    let report = sample_custom_report();
    let result = sample_report_result();

    let export = service.export_report(&report, &result, ReportFormat::Json).await.unwrap();

    assert!(export.file_name.ends_with(".json"));
    assert_eq!(export.content_type, "application/json");
    assert!(export.size_bytes > 0);

    // Verify it's valid JSON
    let content = String::from_utf8(export.data).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).unwrap();
}

#[tokio::test]
async fn test_export_report_html() {
    let service = create_test_report_service().await;
    let report = sample_custom_report();
    let result = sample_report_result();

    let export = service.export_report(&report, &result, ReportFormat::Html).await.unwrap();

    assert!(export.file_name.ends_with(".html"));
    assert_eq!(export.content_type, "text/html");

    let content = String::from_utf8(export.data).unwrap();
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains(&report.name));
    assert!(content.contains("<table>"));
}

#[tokio::test]
async fn test_export_report_excel() {
    let service = create_test_report_service().await;
    let report = sample_custom_report();
    let result = sample_report_result();

    let export = service.export_report(&report, &result, ReportFormat::Excel).await.unwrap();

    assert!(export.file_name.ends_with(".xlsx"));
    assert_eq!(export.content_type, "application/vnd.ms-excel");
}

#[tokio::test]
async fn test_export_report_pdf() {
    let service = create_test_report_service().await;
    let report = sample_custom_report();
    let result = sample_report_result();

    let export = service.export_report(&report, &result, ReportFormat::Pdf).await.unwrap();

    assert!(export.file_name.ends_with(".pdf"));
    assert_eq!(export.content_type, "application/pdf");
}

#[tokio::test]
async fn test_export_report_filename_format() {
    let service = create_test_report_service().await;
    let mut report = sample_custom_report();
    report.name = "My Test Report".to_string();
    let result = sample_report_result();

    let export = service.export_report(&report, &result, ReportFormat::Csv).await.unwrap();

    // Filename should have underscores instead of spaces
    assert!(export.file_name.contains("my_test_report"));
    assert!(export.file_name.contains("_")); // Contains timestamp
}

// ============================================================================
// Scheduled Reports Tests
// ============================================================================

#[tokio::test]
async fn test_schedule_report() {
    let service = create_test_report_service().await;
    let report_id = Uuid::new_v4();
    let schedule = sample_scheduled_report();

    let scheduled = service.schedule_report(report_id, schedule.clone()).await.unwrap();

    assert_eq!(scheduled.report_id, report_id);
    assert_eq!(scheduled.name, schedule.name);
    assert_eq!(scheduled.frequency, schedule.frequency);
}

#[tokio::test]
async fn test_get_scheduled_reports_empty() {
    let service = create_test_report_service().await;
    let reports = service.get_scheduled_reports(None).await.unwrap();
    assert!(reports.is_empty());
}

#[tokio::test]
async fn test_get_scheduled_reports_with_report_id() {
    let service = create_test_report_service().await;
    let reports = service.get_scheduled_reports(Some(Uuid::new_v4())).await.unwrap();
    assert!(reports.is_empty());
}

// ============================================================================
// CustomReport Model Tests
// ============================================================================

#[test]
fn test_custom_report_creation() {
    let report = sample_custom_report();

    assert!(!report.name.is_empty());
    assert!(report.description.is_some());
    assert!(!report.metrics.is_empty());
    assert!(!report.dimensions.is_empty());
    assert!(report.filters.is_empty());
    assert!(!report.is_public);
    assert!(!report.is_favorite);
}

#[test]
fn test_custom_report_serialization() {
    let report = sample_custom_report();

    let json = serde_json::to_string(&report).unwrap();
    let deserialized: CustomReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, report.name);
    assert_eq!(deserialized.metrics.len(), report.metrics.len());
    assert_eq!(deserialized.dimensions.len(), report.dimensions.len());
}

#[test]
fn test_custom_report_clone() {
    let report = sample_custom_report();
    let cloned = report.clone();

    assert_eq!(cloned.id, report.id);
    assert_eq!(cloned.name, report.name);
}

// ============================================================================
// ReportMetric Model Tests
// ============================================================================

#[test]
fn test_report_metric_creation() {
    let metric = ReportMetric {
        id: "sessions".to_string(),
        name: "Sessions".to_string(),
        category: MetricCategory::Session,
        data_type: MetricDataType::Integer,
        aggregation: MetricAggregation::Total,
    };

    assert_eq!(metric.id, "sessions");
    assert_eq!(metric.category, MetricCategory::Session);
    assert_eq!(metric.data_type, MetricDataType::Integer);
    assert_eq!(metric.aggregation, MetricAggregation::Total);
}

#[test]
fn test_metric_category_variants() {
    let categories = vec![
        MetricCategory::User,
        MetricCategory::Session,
        MetricCategory::TrafficSources,
        MetricCategory::PageTracking,
        MetricCategory::GoalConversions,
        MetricCategory::Ecommerce,
        MetricCategory::Custom,
    ];

    for cat in categories {
        let json = serde_json::to_string(&cat).unwrap();
        let _: MetricCategory = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_metric_data_type_variants() {
    let types = vec![
        MetricDataType::Integer,
        MetricDataType::Float,
        MetricDataType::Percent,
        MetricDataType::Time,
        MetricDataType::Currency,
    ];

    for dt in types {
        let json = serde_json::to_string(&dt).unwrap();
        let _: MetricDataType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_metric_aggregation_variants() {
    let aggregations = vec![
        MetricAggregation::Total,
        MetricAggregation::Average,
        MetricAggregation::Minimum,
        MetricAggregation::Maximum,
        MetricAggregation::Count,
        MetricAggregation::CountDistinct,
    ];

    for agg in aggregations {
        let json = serde_json::to_string(&agg).unwrap();
        let _: MetricAggregation = serde_json::from_str(&json).unwrap();
    }
}

// ============================================================================
// ReportDimension Model Tests
// ============================================================================

#[test]
fn test_report_dimension_creation() {
    let dimension = ReportDimension {
        id: "country".to_string(),
        name: "Country".to_string(),
        category: DimensionCategory::GeoNetwork,
    };

    assert_eq!(dimension.id, "country");
    assert_eq!(dimension.category, DimensionCategory::GeoNetwork);
}

#[test]
fn test_dimension_category_variants() {
    let categories = vec![
        DimensionCategory::User,
        DimensionCategory::Session,
        DimensionCategory::TrafficSources,
        DimensionCategory::Platform,
        DimensionCategory::GeoNetwork,
        DimensionCategory::Time,
        DimensionCategory::Custom,
    ];

    for cat in categories {
        let json = serde_json::to_string(&cat).unwrap();
        let _: DimensionCategory = serde_json::from_str(&json).unwrap();
    }
}

// ============================================================================
// ReportFilter Model Tests
// ============================================================================

#[test]
fn test_report_filter_creation() {
    let filter = ReportFilter {
        dimension: "country".to_string(),
        operator: FilterOperator::Equals,
        value: "United States".to_string(),
        case_sensitive: false,
    };

    assert_eq!(filter.dimension, "country");
    assert_eq!(filter.operator, FilterOperator::Equals);
    assert!(!filter.case_sensitive);
}

#[test]
fn test_filter_operator_variants() {
    let operators = vec![
        FilterOperator::Equals,
        FilterOperator::NotEquals,
        FilterOperator::Contains,
        FilterOperator::NotContains,
        FilterOperator::StartsWith,
        FilterOperator::EndsWith,
        FilterOperator::Regex,
        FilterOperator::GreaterThan,
        FilterOperator::LessThan,
        FilterOperator::Between,
        FilterOperator::InList,
    ];

    for op in operators {
        let json = serde_json::to_string(&op).unwrap();
        let _: FilterOperator = serde_json::from_str(&json).unwrap();
    }
}

// ============================================================================
// ChartType Tests
// ============================================================================

#[test]
fn test_chart_type_variants() {
    let types = vec![
        ChartType::Line,
        ChartType::Area,
        ChartType::Bar,
        ChartType::Column,
        ChartType::Pie,
        ChartType::Donut,
        ChartType::Table,
        ChartType::Scorecard,
        ChartType::Geo,
        ChartType::Scatter,
        ChartType::Funnel,
        ChartType::Timeline,
        ChartType::Heatmap,
    ];

    for ct in types {
        let json = serde_json::to_string(&ct).unwrap();
        let _: ChartType = serde_json::from_str(&json).unwrap();
    }
}

// ============================================================================
// ReportResult Model Tests
// ============================================================================

#[test]
fn test_report_result_creation() {
    let result = sample_report_result();

    assert_eq!(result.rows.len(), 2);
    assert!(result.totals.is_some());
    assert_eq!(result.row_count, 2);
}

#[test]
fn test_report_result_serialization() {
    let result = sample_report_result();

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: ReportResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.rows.len(), result.rows.len());
    assert_eq!(deserialized.row_count, result.row_count);
}

#[test]
fn test_report_row_creation() {
    let row = ReportRow {
        dimensions: vec!["2024-01-01".to_string(), "US".to_string()],
        metrics: vec![
            ReportMetricValue {
                metric_id: "sessions".to_string(),
                value: serde_json::Value::Number(1000.into()),
                formatted_value: "1,000".to_string(),
            },
        ],
    };

    assert_eq!(row.dimensions.len(), 2);
    assert_eq!(row.metrics.len(), 1);
}

// ============================================================================
// ScheduledReport Model Tests
// ============================================================================

#[test]
fn test_scheduled_report_creation() {
    let scheduled = sample_scheduled_report();

    assert!(!scheduled.name.is_empty());
    assert_eq!(scheduled.frequency, ReportFrequency::Weekly);
    assert_eq!(scheduled.format, ReportFormat::Pdf);
    assert!(!scheduled.recipients.is_empty());
    assert!(scheduled.enabled);
}

#[test]
fn test_scheduled_report_serialization() {
    let scheduled = sample_scheduled_report();

    let json = serde_json::to_string(&scheduled).unwrap();
    let deserialized: ScheduledReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, scheduled.name);
    assert_eq!(deserialized.frequency, scheduled.frequency);
}

#[test]
fn test_report_status_variants() {
    let statuses = vec![
        ReportStatus::Pending,
        ReportStatus::Running,
        ReportStatus::Completed,
        ReportStatus::Failed,
        ReportStatus::Cancelled,
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let _: ReportStatus = serde_json::from_str(&json).unwrap();
    }
}

// ============================================================================
// ReportTemplate Model Tests
// ============================================================================

#[test]
fn test_report_template_creation() {
    let template = ReportTemplate {
        id: "test_template".to_string(),
        name: "Test Template".to_string(),
        description: "A test template".to_string(),
        category: ReportTemplateCategory::Custom,
        metrics: vec!["sessions".to_string()],
        dimensions: vec!["date".to_string()],
        chart_type: ChartType::Line,
        is_premium: false,
    };

    assert_eq!(template.id, "test_template");
    assert!(!template.is_premium);
}

#[test]
fn test_report_template_category_variants() {
    let categories = vec![
        ReportTemplateCategory::Audience,
        ReportTemplateCategory::Acquisition,
        ReportTemplateCategory::Behavior,
        ReportTemplateCategory::Conversions,
        ReportTemplateCategory::Ecommerce,
        ReportTemplateCategory::RealTime,
        ReportTemplateCategory::Custom,
    ];

    for cat in categories {
        let json = serde_json::to_string(&cat).unwrap();
        let _: ReportTemplateCategory = serde_json::from_str(&json).unwrap();
    }
}

// ============================================================================
// Widget Model Tests
// ============================================================================

#[test]
fn test_dashboard_widget_creation() {
    let widget = DashboardWidget {
        id: Uuid::new_v4(),
        report_id: Some(Uuid::new_v4()),
        widget_type: WidgetType::Chart,
        title: "Test Widget".to_string(),
        position: WidgetPosition { x: 0, y: 0 },
        size: WidgetSize::Medium,
        refresh_interval: Some(60),
        settings: serde_json::json!({"theme": "dark"}),
    };

    assert_eq!(widget.title, "Test Widget");
    assert_eq!(widget.widget_type, WidgetType::Chart);
    assert_eq!(widget.size, WidgetSize::Medium);
}

#[test]
fn test_widget_type_variants() {
    let types = vec![
        WidgetType::Metric,
        WidgetType::Chart,
        WidgetType::Table,
        WidgetType::Timeline,
        WidgetType::Geo,
        WidgetType::Funnel,
        WidgetType::RealTime,
        WidgetType::Custom,
    ];

    for wt in types {
        let json = serde_json::to_string(&wt).unwrap();
        let _: WidgetType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_widget_size_variants() {
    let sizes = vec![
        WidgetSize::Small,
        WidgetSize::Medium,
        WidgetSize::Large,
        WidgetSize::Wide,
        WidgetSize::Tall,
        WidgetSize::Full,
    ];

    for size in sizes {
        let json = serde_json::to_string(&size).unwrap();
        let _: WidgetSize = serde_json::from_str(&json).unwrap();
    }
}

// ============================================================================
// Annotation Model Tests
// ============================================================================

#[test]
fn test_report_annotation_creation() {
    let annotation = ReportAnnotation {
        id: Uuid::new_v4(),
        date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        title: "Campaign Launch".to_string(),
        description: Some("Launched new marketing campaign".to_string()),
        annotation_type: AnnotationType::Campaign,
        visibility: AnnotationVisibility::Shared,
        created_by: Uuid::new_v4(),
        created_at: Utc::now(),
    };

    assert_eq!(annotation.title, "Campaign Launch");
    assert_eq!(annotation.annotation_type, AnnotationType::Campaign);
    assert_eq!(annotation.visibility, AnnotationVisibility::Shared);
}

#[test]
fn test_annotation_type_variants() {
    let types = vec![
        AnnotationType::Note,
        AnnotationType::Campaign,
        AnnotationType::Release,
        AnnotationType::Incident,
        AnnotationType::Milestone,
        AnnotationType::Custom,
    ];

    for at in types {
        let json = serde_json::to_string(&at).unwrap();
        let _: AnnotationType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_annotation_visibility_variants() {
    let visibilities = vec![
        AnnotationVisibility::Private,
        AnnotationVisibility::Shared,
        AnnotationVisibility::Public,
    ];

    for vis in visibilities {
        let json = serde_json::to_string(&vis).unwrap();
        let _: AnnotationVisibility = serde_json::from_str(&json).unwrap();
    }
}

// ============================================================================
// ReportExportResult Tests
// ============================================================================

#[test]
fn test_report_export_result_creation() {
    let export = ReportExportResult {
        file_name: "report_20240115.csv".to_string(),
        content_type: "text/csv".to_string(),
        data: vec![1, 2, 3, 4, 5],
        size_bytes: 5,
    };

    assert_eq!(export.file_name, "report_20240115.csv");
    assert_eq!(export.content_type, "text/csv");
    assert_eq!(export.size_bytes, 5);
    assert_eq!(export.data.len(), 5);
}

#[test]
fn test_report_export_request_creation() {
    let request = ReportExportRequest {
        report_id: Uuid::new_v4(),
        format: ReportFormat::Csv,
        date_range: Some(DateRange::last_n_days(7)),
        include_totals: true,
        include_chart: false,
    };

    assert_eq!(request.format, ReportFormat::Csv);
    assert!(request.include_totals);
    assert!(!request.include_chart);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_custom_report_with_empty_metrics() {
    let now = Utc::now();
    let report = CustomReport {
        id: Uuid::new_v4(),
        name: "Empty Metrics Report".to_string(),
        description: None,
        metrics: vec![],
        dimensions: vec![],
        filters: vec![],
        segments: vec![],
        date_range: DateRangePreset::Last7Days,
        chart_type: ChartType::Table,
        created_at: now,
        updated_at: now,
        created_by: Uuid::new_v4(),
        is_public: false,
        is_favorite: false,
    };

    let json = serde_json::to_string(&report).unwrap();
    let deserialized: CustomReport = serde_json::from_str(&json).unwrap();

    assert!(deserialized.metrics.is_empty());
    assert!(deserialized.dimensions.is_empty());
}

#[test]
fn test_custom_report_with_filters() {
    let report = sample_report_with_filters();

    assert_eq!(report.filters.len(), 1);
    assert_eq!(report.filters[0].dimension, "country");
    assert_eq!(report.filters[0].operator, FilterOperator::Equals);
    assert_eq!(report.filters[0].value, "United States");
    assert!(!report.filters[0].case_sensitive);
}

#[test]
fn test_report_result_with_no_totals() {
    let result = ReportResult {
        report_id: Uuid::new_v4(),
        date_range: DateRange::today(),
        rows: vec![],
        totals: None,
        row_count: 0,
        sampling_info: None,
        generated_at: Utc::now(),
    };

    assert!(result.totals.is_none());
    assert_eq!(result.row_count, 0);
}

#[test]
fn test_scheduled_report_with_no_recipients() {
    let scheduled = ScheduledReport {
        id: Uuid::new_v4(),
        report_id: Uuid::new_v4(),
        name: "No Recipients".to_string(),
        frequency: ReportFrequency::Daily,
        format: ReportFormat::Json,
        recipients: vec![],
        include_comparison: false,
        enabled: false,
        next_run: Utc::now(),
        last_run: None,
        last_status: None,
        created_at: Utc::now(),
    };

    assert!(scheduled.recipients.is_empty());
    assert!(!scheduled.enabled);
}
