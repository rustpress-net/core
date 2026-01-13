-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    role VARCHAR(50) NOT NULL DEFAULT 'subscriber',
    avatar_url TEXT,
    locale VARCHAR(10) DEFAULT 'en',
    timezone VARCHAR(50) DEFAULT 'UTC',
    email_verified_at TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Posts table
CREATE TYPE post_type_enum AS ENUM ('post', 'page', 'custom');
CREATE TYPE post_status_enum AS ENUM ('draft', 'pending', 'published', 'scheduled', 'private', 'trash');

CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id UUID,
    post_type post_type_enum NOT NULL DEFAULT 'post',
    author_id UUID NOT NULL REFERENCES users(id),
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL,
    content TEXT,
    excerpt TEXT,
    status post_status_enum NOT NULL DEFAULT 'draft',
    visibility VARCHAR(50) DEFAULT 'public',
    password VARCHAR(255),
    parent_id UUID REFERENCES posts(id),
    menu_order INT DEFAULT 0,
    template VARCHAR(255),
    featured_image_id UUID,
    comment_status VARCHAR(50) DEFAULT 'open',
    comment_count INT DEFAULT 0,
    ping_status VARCHAR(50) DEFAULT 'open',
    meta_title VARCHAR(500),
    meta_description TEXT,
    canonical_url TEXT,
    published_at TIMESTAMPTZ,
    scheduled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT unique_slug_per_site UNIQUE (site_id, slug)
);

-- Themes table
CREATE TABLE themes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id UUID,
    theme_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    version VARCHAR(50),
    author VARCHAR(255),
    author_url VARCHAR(500),
    license VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    is_installed BOOLEAN NOT NULL DEFAULT TRUE,
    parent_theme_id VARCHAR(100),
    screenshot_url VARCHAR(500),
    homepage_url VARCHAR(500),
    tags TEXT[],
    supports JSONB DEFAULT '{}',
    menu_locations JSONB DEFAULT '{}',
    widget_areas JSONB DEFAULT '{}',
    customizer_schema JSONB DEFAULT '{}',
    settings JSONB DEFAULT '{}',
    template_count INT DEFAULT 0,
    activated_at TIMESTAMPTZ,
    installed_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(theme_id, site_id)
);

-- Options table
CREATE TABLE options (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id UUID,
    option_name VARCHAR(255) NOT NULL,
    option_value JSONB,
    autoload BOOLEAN DEFAULT TRUE,
    CONSTRAINT unique_option_per_site UNIQUE (site_id, option_name)
);

-- Insert default admin user (password: admin123)
INSERT INTO users (email, username, password_hash, display_name, role, status, email_verified_at)
VALUES ('admin@rustpress.local', 'admin', '$argon2id$v=19$m=19456,t=2,p=1$ciLQC8dPihNq8CTLEnStlw$ilGVcVTxhIy20fr0wZtLwSfTsTtKny4DGmp0BfmMUYo', 'Administrator', 'administrator', 'active', NOW());

-- Insert default theme
INSERT INTO themes (theme_id, name, version, description, author, is_active, is_installed, activated_at)
VALUES ('rustpress-enterprise', 'RustPress Enterprise', '1.0.0', 'Default RustPress theme', 'RustPress Team', true, true, NOW());

-- Insert default options
INSERT INTO options (option_name, option_value, autoload) VALUES
    ('site_title', '"RustPress"', true),
    ('site_tagline', '"A Modern CMS Built with Rust"', true),
    ('site_url', '"http://localhost:8080"', true),
    ('posts_per_page', '10', true);
