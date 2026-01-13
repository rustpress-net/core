-- RustCloudflare Plugin - Initial Schema
-- Version: 1.0.0

-- DNS Records Cache
CREATE TABLE IF NOT EXISTS cloudflare_dns_records (
    id SERIAL PRIMARY KEY,
    cloudflare_id VARCHAR(64) UNIQUE NOT NULL,
    record_type VARCHAR(16) NOT NULL,
    name VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    proxied BOOLEAN DEFAULT false,
    ttl INTEGER DEFAULT 1,
    priority INTEGER,
    synced_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_dns_records_name ON cloudflare_dns_records(name);
CREATE INDEX idx_dns_records_type ON cloudflare_dns_records(record_type);

-- Cache Purge Events Log
CREATE TABLE IF NOT EXISTS cloudflare_cache_events (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR(32) NOT NULL,
    details JSONB,
    user_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_cache_events_type ON cloudflare_cache_events(event_type);
CREATE INDEX idx_cache_events_created ON cloudflare_cache_events(created_at);

-- Security Events Log
CREATE TABLE IF NOT EXISTS cloudflare_security_events (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR(32) NOT NULL,
    ip_address INET,
    country VARCHAR(2),
    action VARCHAR(32),
    rule_id VARCHAR(64),
    details JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_security_events_type ON cloudflare_security_events(event_type);
CREATE INDEX idx_security_events_ip ON cloudflare_security_events(ip_address);
CREATE INDEX idx_security_events_created ON cloudflare_security_events(created_at);

-- Workers Deployments
CREATE TABLE IF NOT EXISTS cloudflare_workers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    script TEXT,
    routes JSONB DEFAULT '[]',
    bindings JSONB DEFAULT '{}',
    deployed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- KV Namespace Mappings
CREATE TABLE IF NOT EXISTS cloudflare_kv_namespaces (
    id SERIAL PRIMARY KEY,
    cloudflare_id VARCHAR(64) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    purpose VARCHAR(64),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- R2 Bucket Mappings
CREATE TABLE IF NOT EXISTS cloudflare_r2_buckets (
    id SERIAL PRIMARY KEY,
    name VARCHAR(63) UNIQUE NOT NULL,
    public_url TEXT,
    is_media_storage BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Analytics Snapshots (for historical data)
CREATE TABLE IF NOT EXISTS cloudflare_analytics_snapshots (
    id SERIAL PRIMARY KEY,
    period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    total_requests BIGINT DEFAULT 0,
    cached_requests BIGINT DEFAULT 0,
    uncached_requests BIGINT DEFAULT 0,
    total_bandwidth BIGINT DEFAULT 0,
    cached_bandwidth BIGINT DEFAULT 0,
    threats_blocked INTEGER DEFAULT 0,
    unique_visitors INTEGER DEFAULT 0,
    page_views INTEGER DEFAULT 0,
    data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_analytics_period ON cloudflare_analytics_snapshots(period_start, period_end);

-- Page Rules
CREATE TABLE IF NOT EXISTS cloudflare_page_rules (
    id SERIAL PRIMARY KEY,
    cloudflare_id VARCHAR(64) UNIQUE NOT NULL,
    targets JSONB NOT NULL,
    actions JSONB NOT NULL,
    priority INTEGER DEFAULT 1,
    status VARCHAR(16) DEFAULT 'active',
    synced_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Firewall Rules
CREATE TABLE IF NOT EXISTS cloudflare_firewall_rules (
    id SERIAL PRIMARY KEY,
    cloudflare_id VARCHAR(64) UNIQUE NOT NULL,
    description TEXT,
    action VARCHAR(32) NOT NULL,
    expression TEXT NOT NULL,
    priority INTEGER,
    paused BOOLEAN DEFAULT false,
    synced_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- IP Access Rules
CREATE TABLE IF NOT EXISTS cloudflare_ip_access_rules (
    id SERIAL PRIMARY KEY,
    cloudflare_id VARCHAR(64) UNIQUE NOT NULL,
    mode VARCHAR(32) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    notes TEXT,
    synced_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_ip_rules_mode ON cloudflare_ip_access_rules(mode);
CREATE INDEX idx_ip_rules_ip ON cloudflare_ip_access_rules(ip_address);

-- SSL Certificates
CREATE TABLE IF NOT EXISTS cloudflare_certificates (
    id SERIAL PRIMARY KEY,
    cloudflare_id VARCHAR(64) UNIQUE NOT NULL,
    cert_type VARCHAR(32),
    hosts JSONB,
    status VARCHAR(32),
    expires_at TIMESTAMP WITH TIME ZONE,
    synced_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Plugin Settings
CREATE TABLE IF NOT EXISTS cloudflare_settings (
    id SERIAL PRIMARY KEY,
    key VARCHAR(255) UNIQUE NOT NULL,
    value JSONB,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert default settings
INSERT INTO cloudflare_settings (key, value) VALUES
    ('cdn_enabled', 'true'),
    ('cache_level', '"aggressive"'),
    ('security_level', '"medium"'),
    ('ssl_mode', '"strict"'),
    ('auto_purge_on_update', 'true')
ON CONFLICT (key) DO NOTHING;
