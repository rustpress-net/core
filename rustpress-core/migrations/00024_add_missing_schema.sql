-- Add missing columns and tables to match code expectations

-- Add missing columns to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS locale VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS timezone VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;

-- Create sessions table for user session management
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    user_agent TEXT,
    ip_address VARCHAR(45),
    last_activity_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    revoked_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash);

-- Create options table (distinct from settings, used by OptionsRepository)
CREATE TABLE IF NOT EXISTS options (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    site_id UUID,
    option_name VARCHAR(255) NOT NULL,
    option_value JSONB,
    option_group VARCHAR(100) NOT NULL DEFAULT 'general',
    autoload BOOLEAN NOT NULL DEFAULT TRUE,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    value_type VARCHAR(50),
    validation JSONB,
    display_name VARCHAR(255),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(site_id, option_name)
);

CREATE INDEX IF NOT EXISTS idx_options_site_id ON options(site_id);
CREATE INDEX IF NOT EXISTS idx_options_option_name ON options(option_name);
CREATE INDEX IF NOT EXISTS idx_options_option_group ON options(option_group);
CREATE INDEX IF NOT EXISTS idx_options_autoload ON options(autoload);

-- Migrate existing settings to options table
INSERT INTO options (option_name, option_value, option_group, autoload, is_system)
SELECT
    key as option_name,
    CASE
        WHEN type = 'boolean' THEN to_jsonb(value = 'true')
        WHEN type = 'integer' THEN to_jsonb(value::integer)
        ELSE to_jsonb(value)
    END as option_value,
    group_name as option_group,
    TRUE as autoload,
    TRUE as is_system
FROM settings
ON CONFLICT (site_id, option_name) DO NOTHING;

-- Add missing columns to media table if needed
ALTER TABLE media ADD COLUMN IF NOT EXISTS folder_id UUID;
ALTER TABLE media ADD COLUMN IF NOT EXISTS title VARCHAR(500);
ALTER TABLE media ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE media ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;

-- Create media_folders table
CREATE TABLE IF NOT EXISTS media_folders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    parent_id UUID REFERENCES media_folders(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_media_folders_parent_id ON media_folders(parent_id);

-- Add foreign key for folder_id in media (only if folder_id was just added)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'fk_media_folder' AND table_name = 'media'
    ) THEN
        ALTER TABLE media ADD CONSTRAINT fk_media_folder
            FOREIGN KEY (folder_id) REFERENCES media_folders(id) ON DELETE SET NULL;
    END IF;
END $$;

-- Add missing columns to comments table
ALTER TABLE comments ADD COLUMN IF NOT EXISTS likes_count INTEGER DEFAULT 0;
ALTER TABLE comments ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;

-- Create comment_likes table
CREATE TABLE IF NOT EXISTS comment_likes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    comment_id UUID NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    ip_address VARCHAR(45),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create unique index with COALESCE for proper null handling
CREATE UNIQUE INDEX IF NOT EXISTS idx_comment_likes_unique
    ON comment_likes(comment_id, COALESCE(user_id, '00000000-0000-0000-0000-000000000000'::uuid), COALESCE(ip_address, ''));

CREATE INDEX IF NOT EXISTS idx_comment_likes_comment_id ON comment_likes(comment_id);
