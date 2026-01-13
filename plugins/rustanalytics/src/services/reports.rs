//! Report Service
//!
//! Service for creating, managing, and running custom analytics reports.

use std::sync::Arc;

use chrono::Utc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::reports::*;
use crate::models::api::*;
use crate::models::{DateRange, DateRangePreset, ReportFormat};
use crate::services::cache::CacheService;
use crate::services::client::{ClientError, GoogleAnalyticsClient};

/// Database pool type alias
type DbPool = Arc<dyn std::any::Any + Send + Sync>;

/// Report Service for custom report management
pub struct ReportService {
    /// GA API client
    client: Arc<GoogleAnalyticsClient>,
    /// Cache service
    cache: Arc<CacheService>,
    /// Database pool (reserved for future database integration)
    #[allow(dead_code)]
    db: DbPool,
}

impl ReportService {
    /// Create a new report service
    pub fn new(client: Arc<GoogleAnalyticsClient>, cache: Arc<CacheService>, db: DbPool) -> Self {
        Self { client, cache, db }
    }

    /// List all custom reports
    pub async fn list_reports(&self, _user_id: Option<Uuid>) -> Result<Vec<CustomReport>, ClientError> {
        // In a real implementation, this would query the database
        // For now, return empty list
        Ok(Vec::new())
    }

    /// Get a custom report by ID
    pub async fn get_report(&self, _report_id: Uuid) -> Result<Option<CustomReport>, ClientError> {
        // Database query would go here
        Ok(None)
    }

    /// Create a new custom report
    pub async fn create_report(&self, report: CustomReport) -> Result<CustomReport, ClientError> {
        let now = Utc::now();
        let new_report = CustomReport {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            ..report
        };

        // Save to database
        info!("Created new report: {}", new_report.id);

        Ok(new_report)
    }

    /// Update an existing report
    pub async fn update_report(
        &self,
        report_id: Uuid,
        updates: CustomReport,
    ) -> Result<CustomReport, ClientError> {
        let updated_report = CustomReport {
            id: report_id,
            updated_at: Utc::now(),
            ..updates
        };

        // Update in database
        info!("Updated report: {}", report_id);

        Ok(updated_report)
    }

    /// Delete a custom report
    pub async fn delete_report(&self, report_id: Uuid) -> Result<bool, ClientError> {
        // Delete from database
        info!("Deleted report: {}", report_id);
        Ok(true)
    }

    /// Run a custom report
    pub async fn run_report(
        &self,
        report: &CustomReport,
        date_range: Option<DateRange>,
    ) -> Result<ReportResult, ClientError> {
        let date_range = date_range.unwrap_or_else(|| self.date_range_from_preset(&report.date_range));

        // Check cache first
        let cache_key = format!(
            "report:{}:{}:{}",
            report.id, date_range.start_date, date_range.end_date
        );

        if let Some(cached) = self.cache.get::<ReportResult>(&cache_key).await {
            debug!("Returning cached report result");
            return Ok(cached);
        }

        // Build the GA4 report request
        let request = self.build_report_request(report, &date_range)?;

        // Execute the report
        let response = self.client.run_report(request).await?;

        // Process the response
        let result = self.process_report_response(report, response, date_range)?;

        // Cache the result
        self.cache.set(&cache_key, &result).await;

        Ok(result)
    }

    /// Build a GA4 report request from a custom report definition
    fn build_report_request(
        &self,
        report: &CustomReport,
        date_range: &DateRange,
    ) -> Result<RunReportRequest, ClientError> {
        // Build dimensions
        let dimensions: Option<Vec<Dimension>> = if report.dimensions.is_empty() {
            None
        } else {
            Some(
                report
                    .dimensions
                    .iter()
                    .map(|d| Dimension {
                        name: d.id.clone(),
                        dimension_expression: None,
                    })
                    .collect(),
            )
        };

        // Build metrics
        let metrics: Vec<Metric> = report
            .metrics
            .iter()
            .map(|m| Metric {
                name: m.id.clone(),
                expression: None,
                invisible: None,
            })
            .collect();

        // Build filters
        let dimension_filter = if report.filters.is_empty() {
            None
        } else {
            Some(self.build_filter_expression(&report.filters)?)
        };

        Ok(RunReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            date_ranges: vec![GoogleAnalyticsClient::build_date_range(date_range)],
            dimensions,
            metrics,
            dimension_filter,
            metric_filter: None,
            order_bys: None,
            offset: None,
            limit: Some(10000),
            metric_aggregations: Some(vec!["TOTAL".to_string()]),
            keep_empty_rows: None,
            return_property_quota: None,
        })
    }

    /// Build a filter expression from report filters
    fn build_filter_expression(
        &self,
        filters: &[ReportFilter],
    ) -> Result<FilterExpression, ClientError> {
        if filters.is_empty() {
            return Err(ClientError::InvalidResponse("No filters provided".to_string()));
        }

        if filters.len() == 1 {
            return Ok(self.build_single_filter(&filters[0])?);
        }

        // Multiple filters - combine with AND
        let expressions: Vec<FilterExpression> = filters
            .iter()
            .map(|f| self.build_single_filter(f))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(FilterExpression {
            and_group: Some(FilterExpressionList { expressions }),
            or_group: None,
            not_expression: None,
            filter: None,
        })
    }

    /// Build a single filter
    fn build_single_filter(&self, filter: &ReportFilter) -> Result<FilterExpression, ClientError> {
        let match_type = match filter.operator {
            FilterOperator::Equals => StringFilterMatchType::Exact,
            FilterOperator::Contains => StringFilterMatchType::Contains,
            FilterOperator::StartsWith => StringFilterMatchType::BeginsWith,
            FilterOperator::EndsWith => StringFilterMatchType::EndsWith,
            FilterOperator::Regex => StringFilterMatchType::FullRegexp,
            _ => StringFilterMatchType::Exact,
        };

        Ok(FilterExpression {
            and_group: None,
            or_group: None,
            not_expression: None,
            filter: Some(Filter {
                field_name: filter.dimension.clone(),
                string_filter: Some(StringFilter {
                    match_type,
                    value: filter.value.clone(),
                    case_sensitive: Some(filter.case_sensitive),
                }),
                in_list_filter: None,
                numeric_filter: None,
                between_filter: None,
            }),
        })
    }

    /// Process report response into ReportResult
    fn process_report_response(
        &self,
        report: &CustomReport,
        response: RunReportResponse,
        date_range: DateRange,
    ) -> Result<ReportResult, ClientError> {
        let mut rows = Vec::new();

        if let Some(response_rows) = response.rows {
            for row in response_rows {
                let dimensions: Vec<String> = row
                    .dimension_values
                    .map(|dims| {
                        dims.iter()
                            .map(|d| d.value.clone().unwrap_or_default())
                            .collect()
                    })
                    .unwrap_or_default();

                let metrics: Vec<ReportMetricValue> = row
                    .metric_values
                    .map(|vals| {
                        vals.iter()
                            .enumerate()
                            .map(|(i, v)| {
                                let value = v.value.clone().unwrap_or_default();
                                let metric_id = report
                                    .metrics
                                    .get(i)
                                    .map(|m| m.id.clone())
                                    .unwrap_or_default();

                                ReportMetricValue {
                                    metric_id,
                                    value: serde_json::Value::String(value.clone()),
                                    formatted_value: value,
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                rows.push(ReportRow { dimensions, metrics });
            }
        }

        // Process totals
        let totals = response.totals.and_then(|t| t.into_iter().next()).map(|row| {
            let dimensions = Vec::new();
            let metrics: Vec<ReportMetricValue> = row
                .metric_values
                .map(|vals| {
                    vals.iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let value = v.value.clone().unwrap_or_default();
                            let metric_id = report
                                .metrics
                                .get(i)
                                .map(|m| m.id.clone())
                                .unwrap_or_default();

                            ReportMetricValue {
                                metric_id,
                                value: serde_json::Value::String(value.clone()),
                                formatted_value: value,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            ReportRow { dimensions, metrics }
        });

        let row_count = response.row_count.map(|c| c as u64).unwrap_or(rows.len() as u64);

        Ok(ReportResult {
            report_id: report.id,
            date_range,
            rows,
            totals,
            row_count,
            sampling_info: None,
            generated_at: Utc::now(),
        })
    }

    /// Export a report to the specified format
    pub async fn export_report(
        &self,
        report: &CustomReport,
        result: &ReportResult,
        format: ReportFormat,
    ) -> Result<ReportExportResult, ClientError> {
        let (data, content_type, extension) = match format {
            ReportFormat::Csv => {
                let csv = self.generate_csv(report, result)?;
                (csv.into_bytes(), "text/csv", "csv")
            }
            ReportFormat::Json => {
                let json = serde_json::to_string_pretty(result)
                    .map_err(|e| ClientError::InvalidResponse(e.to_string()))?;
                (json.into_bytes(), "application/json", "json")
            }
            ReportFormat::Excel => {
                // Simplified - would use actual Excel library
                let csv = self.generate_csv(report, result)?;
                (csv.into_bytes(), "application/vnd.ms-excel", "xlsx")
            }
            ReportFormat::Pdf => {
                // Simplified - would use actual PDF library
                let html = self.generate_html(report, result)?;
                (html.into_bytes(), "application/pdf", "pdf")
            }
            ReportFormat::Html => {
                let html = self.generate_html(report, result)?;
                (html.into_bytes(), "text/html", "html")
            }
        };

        let file_name = format!(
            "{}_{}.{}",
            report.name.replace(" ", "_").to_lowercase(),
            Utc::now().format("%Y%m%d_%H%M%S"),
            extension
        );

        Ok(ReportExportResult {
            file_name,
            content_type: content_type.to_string(),
            size_bytes: data.len() as u64,
            data,
        })
    }

    /// Generate CSV from report result
    fn generate_csv(
        &self,
        report: &CustomReport,
        result: &ReportResult,
    ) -> Result<String, ClientError> {
        let mut csv = String::new();

        // Header row
        let headers: Vec<String> = report
            .dimensions
            .iter()
            .map(|d| d.name.clone())
            .chain(report.metrics.iter().map(|m| m.name.clone()))
            .collect();
        csv.push_str(&headers.join(","));
        csv.push('\n');

        // Data rows
        for row in &result.rows {
            let values: Vec<String> = row
                .dimensions
                .iter()
                .cloned()
                .chain(row.metrics.iter().map(|m| m.formatted_value.clone()))
                .collect();
            csv.push_str(&values.join(","));
            csv.push('\n');
        }

        Ok(csv)
    }

    /// Generate HTML from report result
    fn generate_html(
        &self,
        report: &CustomReport,
        result: &ReportResult,
    ) -> Result<String, ClientError> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html><html><head>");
        html.push_str("<meta charset=\"UTF-8\">");
        html.push_str(&format!("<title>{}</title>", report.name));
        html.push_str("<style>");
        html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }");
        html.push_str("table { border-collapse: collapse; width: 100%; }");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }");
        html.push_str("th { background-color: #4CAF50; color: white; }");
        html.push_str("tr:nth-child(even) { background-color: #f2f2f2; }");
        html.push_str("</style></head><body>");

        html.push_str(&format!("<h1>{}</h1>", report.name));
        if let Some(ref desc) = report.description {
            html.push_str(&format!("<p>{}</p>", desc));
        }
        html.push_str(&format!(
            "<p>Date Range: {} to {}</p>",
            result.date_range.start_date, result.date_range.end_date
        ));

        html.push_str("<table><thead><tr>");

        // Headers
        for dim in &report.dimensions {
            html.push_str(&format!("<th>{}</th>", dim.name));
        }
        for metric in &report.metrics {
            html.push_str(&format!("<th>{}</th>", metric.name));
        }
        html.push_str("</tr></thead><tbody>");

        // Data rows
        for row in &result.rows {
            html.push_str("<tr>");
            for dim in &row.dimensions {
                html.push_str(&format!("<td>{}</td>", dim));
            }
            for metric in &row.metrics {
                html.push_str(&format!("<td>{}</td>", metric.formatted_value));
            }
            html.push_str("</tr>");
        }

        html.push_str("</tbody></table>");
        html.push_str(&format!(
            "<p>Generated: {}</p>",
            result.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        html.push_str("</body></html>");

        Ok(html)
    }

    /// Get date range from preset
    fn date_range_from_preset(&self, preset: &DateRangePreset) -> DateRange {
        match preset {
            DateRangePreset::Today => DateRange::today(),
            DateRangePreset::Yesterday => DateRange::yesterday(),
            DateRangePreset::Last7Days => DateRange::last_n_days(7),
            DateRangePreset::Last14Days => DateRange::last_n_days(14),
            DateRangePreset::Last28Days => DateRange::last_n_days(28),
            DateRangePreset::Last30Days => DateRange::last_n_days(30),
            DateRangePreset::Last90Days => DateRange::last_n_days(90),
            DateRangePreset::Last365Days => DateRange::last_n_days(365),
            DateRangePreset::ThisMonth => DateRange::this_month(),
            DateRangePreset::LastMonth => DateRange::last_month(),
            _ => DateRange::last_n_days(30),
        }
    }

    /// Get available report templates
    pub fn get_templates(&self) -> Vec<ReportTemplate> {
        vec![
            ReportTemplate {
                id: "audience_overview".to_string(),
                name: "Audience Overview".to_string(),
                description: "Overview of your audience including users, sessions, and demographics".to_string(),
                category: ReportTemplateCategory::Audience,
                metrics: vec!["totalUsers".to_string(), "sessions".to_string(), "bounceRate".to_string()],
                dimensions: vec!["date".to_string()],
                chart_type: ChartType::Line,
                is_premium: false,
            },
            ReportTemplate {
                id: "traffic_sources".to_string(),
                name: "Traffic Sources".to_string(),
                description: "Where your visitors come from".to_string(),
                category: ReportTemplateCategory::Acquisition,
                metrics: vec!["sessions".to_string(), "totalUsers".to_string()],
                dimensions: vec!["sessionSource".to_string(), "sessionMedium".to_string()],
                chart_type: ChartType::Pie,
                is_premium: false,
            },
            ReportTemplate {
                id: "top_pages".to_string(),
                name: "Top Pages".to_string(),
                description: "Most viewed pages on your site".to_string(),
                category: ReportTemplateCategory::Behavior,
                metrics: vec!["screenPageViews".to_string(), "averageSessionDuration".to_string()],
                dimensions: vec!["pagePath".to_string(), "pageTitle".to_string()],
                chart_type: ChartType::Bar,
                is_premium: false,
            },
            ReportTemplate {
                id: "ecommerce_overview".to_string(),
                name: "E-commerce Overview".to_string(),
                description: "Revenue, transactions, and product performance".to_string(),
                category: ReportTemplateCategory::Ecommerce,
                metrics: vec!["totalRevenue".to_string(), "ecommercePurchases".to_string()],
                dimensions: vec!["date".to_string()],
                chart_type: ChartType::Area,
                is_premium: true,
            },
            ReportTemplate {
                id: "conversions".to_string(),
                name: "Conversions".to_string(),
                description: "Goal completions and conversion rates".to_string(),
                category: ReportTemplateCategory::Conversions,
                metrics: vec!["conversions".to_string()],
                dimensions: vec!["eventName".to_string()],
                chart_type: ChartType::Funnel,
                is_premium: false,
            },
        ]
    }

    /// Create a report from a template
    pub fn create_from_template(
        &self,
        template_id: &str,
        user_id: Uuid,
    ) -> Result<CustomReport, ClientError> {
        let template = self
            .get_templates()
            .into_iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| ClientError::InvalidResponse("Template not found".to_string()))?;

        let now = Utc::now();

        Ok(CustomReport {
            id: Uuid::new_v4(),
            name: template.name,
            description: Some(template.description),
            metrics: template
                .metrics
                .into_iter()
                .map(|m| ReportMetric {
                    id: m.clone(),
                    name: m,
                    category: MetricCategory::Session,
                    data_type: MetricDataType::Integer,
                    aggregation: MetricAggregation::Total,
                })
                .collect(),
            dimensions: template
                .dimensions
                .into_iter()
                .map(|d| ReportDimension {
                    id: d.clone(),
                    name: d,
                    category: DimensionCategory::Session,
                })
                .collect(),
            filters: Vec::new(),
            segments: Vec::new(),
            date_range: DateRangePreset::Last30Days,
            chart_type: template.chart_type,
            created_at: now,
            updated_at: now,
            created_by: user_id,
            is_public: false,
            is_favorite: false,
        })
    }

    /// Schedule a report
    pub async fn schedule_report(
        &self,
        report_id: Uuid,
        schedule: ScheduledReport,
    ) -> Result<ScheduledReport, ClientError> {
        let scheduled = ScheduledReport {
            id: Uuid::new_v4(),
            report_id,
            created_at: Utc::now(),
            ..schedule
        };

        // Save to database
        info!("Scheduled report {} with ID {}", report_id, scheduled.id);

        Ok(scheduled)
    }

    /// Get scheduled reports
    pub async fn get_scheduled_reports(
        &self,
        _report_id: Option<Uuid>,
    ) -> Result<Vec<ScheduledReport>, ClientError> {
        // Query database
        Ok(Vec::new())
    }
}

impl std::fmt::Debug for ReportService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReportService")
            .field("client", &self.client)
            .finish()
    }
}
