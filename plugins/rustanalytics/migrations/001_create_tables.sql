-- RustAnalytics Database Migrations
-- Migration 001: Create initial tables for Google Analytics integration

-- Settings table for plugin configuration
CREATE TABLE IF NOT EXISTS rustanalytics_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    setting_key VARCHAR(100) NOT NULL UNIQUE,
    setting_value JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index for settings lookup
CREATE INDEX idx_rustanalytics_settings_key ON rustanalytics_settings(setting_key);

-- Cache table for analytics data
CREATE TABLE IF NOT EXISTS rustanalytics_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_key VARCHAR(500) NOT NULL UNIQUE,
    cache_data JSONB NOT NULL,
    data_type VARCHAR(100) NOT NULL, -- overview, realtime, audience, etc.
    date_range_start DATE,
    date_range_end DATE,
    property_id VARCHAR(100),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for cache table
CREATE INDEX idx_rustanalytics_cache_key ON rustanalytics_cache(cache_key);
CREATE INDEX idx_rustanalytics_cache_type ON rustanalytics_cache(data_type);
CREATE INDEX idx_rustanalytics_cache_expires ON rustanalytics_cache(expires_at);
CREATE INDEX idx_rustanalytics_cache_date_range ON rustanalytics_cache(date_range_start, date_range_end);
CREATE INDEX idx_rustanalytics_cache_property ON rustanalytics_cache(property_id);

-- Reports table for custom saved reports
CREATE TABLE IF NOT EXISTS rustanalytics_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    report_type VARCHAR(50) NOT NULL, -- custom, template, scheduled

    -- Report configuration
    date_range_type VARCHAR(50), -- relative (last7days, etc.) or absolute
    date_range_start DATE,
    date_range_end DATE,
    dimensions TEXT[], -- Array of dimension names
    metrics TEXT[], -- Array of metric names
    filters JSONB, -- Filter configuration
    sort_by VARCHAR(100),
    sort_order VARCHAR(10) DEFAULT 'desc',
    row_limit INTEGER DEFAULT 100,

    -- Schedule configuration
    is_scheduled BOOLEAN DEFAULT false,
    schedule_frequency VARCHAR(20), -- daily, weekly, monthly
    schedule_day INTEGER, -- day of week (0-6) or day of month (1-31)
    schedule_hour INTEGER DEFAULT 8, -- Hour of day (0-23)
    schedule_timezone VARCHAR(50) DEFAULT 'UTC',
    schedule_recipients TEXT[], -- Email addresses
    last_scheduled_run TIMESTAMP WITH TIME ZONE,
    next_scheduled_run TIMESTAMP WITH TIME ZONE,

    -- Metadata
    created_by UUID,
    is_public BOOLEAN DEFAULT false,
    tags TEXT[],

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_run_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for reports table
CREATE INDEX idx_rustanalytics_reports_name ON rustanalytics_reports(name);
CREATE INDEX idx_rustanalytics_reports_type ON rustanalytics_reports(report_type);
CREATE INDEX idx_rustanalytics_reports_scheduled ON rustanalytics_reports(is_scheduled) WHERE is_scheduled = true;
CREATE INDEX idx_rustanalytics_reports_next_run ON rustanalytics_reports(next_scheduled_run) WHERE is_scheduled = true;
CREATE INDEX idx_rustanalytics_reports_created_by ON rustanalytics_reports(created_by);

-- Report runs/history table
CREATE TABLE IF NOT EXISTS rustanalytics_report_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL REFERENCES rustanalytics_reports(id) ON DELETE CASCADE,

    -- Run details
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, running, completed, failed
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    duration_ms INTEGER,

    -- Results
    row_count INTEGER,
    result_data JSONB,
    totals JSONB,

    -- Metadata
    run_type VARCHAR(20) DEFAULT 'manual', -- manual, scheduled
    triggered_by UUID,
    error_message TEXT,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for report runs
CREATE INDEX idx_rustanalytics_report_runs_report ON rustanalytics_report_runs(report_id);
CREATE INDEX idx_rustanalytics_report_runs_status ON rustanalytics_report_runs(status);
CREATE INDEX idx_rustanalytics_report_runs_date ON rustanalytics_report_runs(created_at);

-- Sync history table
CREATE TABLE IF NOT EXISTS rustanalytics_sync_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sync_type VARCHAR(50) NOT NULL, -- full, essential, realtime
    status VARCHAR(20) NOT NULL, -- success, partial, failed

    -- Results
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    duration_ms INTEGER,
    records_synced BIGINT DEFAULT 0,

    -- Details
    date_ranges_synced JSONB, -- Which date ranges were synced
    errors JSONB, -- Array of error messages if any

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for sync history
CREATE INDEX idx_rustanalytics_sync_history_type ON rustanalytics_sync_history(sync_type);
CREATE INDEX idx_rustanalytics_sync_history_status ON rustanalytics_sync_history(status);
CREATE INDEX idx_rustanalytics_sync_history_date ON rustanalytics_sync_history(started_at DESC);

-- Goals configuration table
CREATE TABLE IF NOT EXISTS rustanalytics_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ga_goal_id VARCHAR(50), -- ID from Google Analytics
    name VARCHAR(255) NOT NULL,
    description TEXT,
    goal_type VARCHAR(50) NOT NULL, -- destination, duration, pages, event

    -- Goal configuration
    config JSONB NOT NULL, -- Type-specific configuration
    value DECIMAL(10, 2) DEFAULT 0, -- Goal value

    -- Status
    is_active BOOLEAN DEFAULT true,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for goals
CREATE INDEX idx_rustanalytics_goals_ga_id ON rustanalytics_goals(ga_goal_id);
CREATE INDEX idx_rustanalytics_goals_type ON rustanalytics_goals(goal_type);
CREATE INDEX idx_rustanalytics_goals_active ON rustanalytics_goals(is_active);

-- Daily aggregated data table (for faster historical queries)
CREATE TABLE IF NOT EXISTS rustanalytics_daily_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data_date DATE NOT NULL,
    property_id VARCHAR(100) NOT NULL,

    -- Overview metrics
    sessions INTEGER DEFAULT 0,
    users INTEGER DEFAULT 0,
    new_users INTEGER DEFAULT 0,
    pageviews INTEGER DEFAULT 0,
    pages_per_session DECIMAL(10, 4) DEFAULT 0,
    avg_session_duration DECIMAL(10, 2) DEFAULT 0,
    bounce_rate DECIMAL(5, 2) DEFAULT 0,

    -- Conversions
    goal_completions INTEGER DEFAULT 0,
    goal_value DECIMAL(12, 2) DEFAULT 0,

    -- E-commerce
    transactions INTEGER DEFAULT 0,
    revenue DECIMAL(12, 2) DEFAULT 0,

    -- Detailed breakdowns stored as JSONB
    traffic_sources JSONB, -- By source/medium
    channels JSONB, -- By channel
    devices JSONB, -- By device category
    countries JSONB, -- By country
    pages JSONB, -- Top pages

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Unique constraint
    UNIQUE(data_date, property_id)
);

-- Indexes for daily data
CREATE INDEX idx_rustanalytics_daily_date ON rustanalytics_daily_data(data_date);
CREATE INDEX idx_rustanalytics_daily_property ON rustanalytics_daily_data(property_id);
CREATE INDEX idx_rustanalytics_daily_date_range ON rustanalytics_daily_data(data_date, property_id);

-- Alerts configuration table
CREATE TABLE IF NOT EXISTS rustanalytics_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_enabled BOOLEAN DEFAULT true,

    -- Alert configuration
    metric VARCHAR(100) NOT NULL, -- sessions, users, bounceRate, etc.
    condition VARCHAR(20) NOT NULL, -- gt, lt, eq, change_up, change_down
    threshold DECIMAL(15, 4) NOT NULL,
    comparison_period VARCHAR(50), -- For change comparisons (e.g., previous_day)

    -- Notification settings
    notification_channels TEXT[], -- email, slack, webhook
    notification_recipients JSONB,
    cooldown_minutes INTEGER DEFAULT 60, -- Minimum time between alerts

    -- Status
    last_triggered_at TIMESTAMP WITH TIME ZONE,
    trigger_count INTEGER DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for alerts
CREATE INDEX idx_rustanalytics_alerts_enabled ON rustanalytics_alerts(is_enabled);
CREATE INDEX idx_rustanalytics_alerts_metric ON rustanalytics_alerts(metric);

-- Alert history table
CREATE TABLE IF NOT EXISTS rustanalytics_alert_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID NOT NULL REFERENCES rustanalytics_alerts(id) ON DELETE CASCADE,

    -- Trigger details
    triggered_at TIMESTAMP WITH TIME ZONE NOT NULL,
    metric_value DECIMAL(15, 4) NOT NULL,
    threshold_value DECIMAL(15, 4) NOT NULL,
    comparison_value DECIMAL(15, 4),

    -- Notification status
    notifications_sent JSONB, -- Which channels were notified

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for alert history
CREATE INDEX idx_rustanalytics_alert_history_alert ON rustanalytics_alert_history(alert_id);
CREATE INDEX idx_rustanalytics_alert_history_date ON rustanalytics_alert_history(triggered_at DESC);

-- Annotations table (for marking events on charts)
CREATE TABLE IF NOT EXISTS rustanalytics_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    annotation_date DATE NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50), -- marketing, technical, business, other
    color VARCHAR(20) DEFAULT '#3B82F6',
    is_visible BOOLEAN DEFAULT true,
    created_by UUID,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for annotations
CREATE INDEX idx_rustanalytics_annotations_date ON rustanalytics_annotations(annotation_date);
CREATE INDEX idx_rustanalytics_annotations_category ON rustanalytics_annotations(category);
CREATE INDEX idx_rustanalytics_annotations_visible ON rustanalytics_annotations(is_visible);

-- Dashboards table (for custom dashboard layouts)
CREATE TABLE IF NOT EXISTS rustanalytics_dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_default BOOLEAN DEFAULT false,

    -- Layout configuration
    layout JSONB NOT NULL, -- Widget positions and sizes
    widgets JSONB NOT NULL, -- Widget configurations

    -- Permissions
    created_by UUID,
    is_public BOOLEAN DEFAULT false,
    shared_with UUID[], -- User IDs who can view

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for dashboards
CREATE INDEX idx_rustanalytics_dashboards_default ON rustanalytics_dashboards(is_default);
CREATE INDEX idx_rustanalytics_dashboards_created_by ON rustanalytics_dashboards(created_by);

-- Create triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION update_rustanalytics_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers to tables with updated_at
CREATE TRIGGER update_rustanalytics_settings_updated_at
    BEFORE UPDATE ON rustanalytics_settings
    FOR EACH ROW EXECUTE FUNCTION update_rustanalytics_updated_at();

CREATE TRIGGER update_rustanalytics_cache_updated_at
    BEFORE UPDATE ON rustanalytics_cache
    FOR EACH ROW EXECUTE FUNCTION update_rustanalytics_updated_at();

CREATE TRIGGER update_rustanalytics_reports_updated_at
    BEFORE UPDATE ON rustanalytics_reports
    FOR EACH ROW EXECUTE FUNCTION update_rustanalytics_updated_at();

CREATE TRIGGER update_rustanalytics_goals_updated_at
    BEFORE UPDATE ON rustanalytics_goals
    FOR EACH ROW EXECUTE FUNCTION update_rustanalytics_updated_at();

CREATE TRIGGER update_rustanalytics_daily_data_updated_at
    BEFORE UPDATE ON rustanalytics_daily_data
    FOR EACH ROW EXECUTE FUNCTION update_rustanalytics_updated_at();

CREATE TRIGGER update_rustanalytics_alerts_updated_at
    BEFORE UPDATE ON rustanalytics_alerts
    FOR EACH ROW EXECUTE FUNCTION update_rustanalytics_updated_at();

CREATE TRIGGER update_rustanalytics_annotations_updated_at
    BEFORE UPDATE ON rustanalytics_annotations
    FOR EACH ROW EXECUTE FUNCTION update_rustanalytics_updated_at();

CREATE TRIGGER update_rustanalytics_dashboards_updated_at
    BEFORE UPDATE ON rustanalytics_dashboards
    FOR EACH ROW EXECUTE FUNCTION update_rustanalytics_updated_at();

-- Insert default settings
INSERT INTO rustanalytics_settings (setting_key, setting_value)
VALUES
    ('ga_config', '{"property_id": "", "measurement_id": "", "service_account_json": null}'::jsonb),
    ('tracking_options', '{"enable_tracking": true, "track_logged_in_users": true, "track_admin_users": false, "anonymize_ip": true, "respect_dnt": true, "cookie_consent_required": true, "enhanced_link_attribution": false, "enhanced_ecommerce": false}'::jsonb),
    ('dashboard_preferences', '{"default_date_range": "last7days", "show_realtime_widget": true, "show_traffic_widget": true, "show_toppages_widget": true, "show_acquisition_widget": true}'::jsonb),
    ('report_settings', '{"email_enabled": false, "email_recipients": [], "frequency": "weekly"}'::jsonb),
    ('privacy_settings', '{"gdpr_compliant": true, "ccpa_compliant": false}'::jsonb),
    ('cache_settings', '{"cache_duration_minutes": 15, "max_memory_entries": 1000}'::jsonb)
ON CONFLICT (setting_key) DO NOTHING;

-- Create a cleanup job entry (handled by application cron)
-- This comment serves as documentation for the cleanup process
-- Cleanup should:
-- 1. Delete expired cache entries: DELETE FROM rustanalytics_cache WHERE expires_at < NOW()
-- 2. Delete old sync history: DELETE FROM rustanalytics_sync_history WHERE created_at < NOW() - INTERVAL '30 days'
-- 3. Delete old report runs: DELETE FROM rustanalytics_report_runs WHERE created_at < NOW() - INTERVAL '90 days'
-- 4. Delete old alert history: DELETE FROM rustanalytics_alert_history WHERE created_at < NOW() - INTERVAL '90 days'
