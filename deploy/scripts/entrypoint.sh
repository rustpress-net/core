#!/bin/bash
set -e

# RustPress Docker Entrypoint Script
# Handles database migrations and server startup

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Wait for PostgreSQL to be ready
wait_for_postgres() {
    log_info "Waiting for PostgreSQL to be ready..."

    max_attempts=30
    attempt=1

    while [ $attempt -le $max_attempts ]; do
        if pg_isready -h "${PGHOST:-postgres}" -p "${PGPORT:-5432}" -U "${PGUSER:-rustpress}" > /dev/null 2>&1; then
            log_info "PostgreSQL is ready!"
            return 0
        fi

        log_info "Attempt $attempt/$max_attempts - PostgreSQL not ready, waiting..."
        sleep 2
        attempt=$((attempt + 1))
    done

    log_error "PostgreSQL did not become ready in time"
    return 1
}

# Run database migrations
run_migrations() {
    log_info "Running database migrations..."

    MIGRATIONS_DIR="${MIGRATIONS_DIR:-/app/migrations}"

    if [ ! -d "$MIGRATIONS_DIR" ]; then
        log_warn "Migrations directory not found: $MIGRATIONS_DIR"
        return 0
    fi

    # Create migrations tracking table if it doesn't exist
    psql "$DATABASE_URL" -c "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
    " 2>/dev/null || true

    # Get list of applied migrations
    applied=$(psql "$DATABASE_URL" -t -c "SELECT version FROM schema_migrations ORDER BY version;" 2>/dev/null | tr -d ' ')

    # Run pending migrations
    migration_count=0
    for migration_file in $(ls -1 "$MIGRATIONS_DIR"/*.sql 2>/dev/null | sort); do
        migration_name=$(basename "$migration_file")

        # Check if already applied
        if echo "$applied" | grep -q "^${migration_name}$"; then
            log_info "Migration already applied: $migration_name"
            continue
        fi

        log_info "Applying migration: $migration_name"

        if psql "$DATABASE_URL" -f "$migration_file"; then
            # Record migration as applied
            psql "$DATABASE_URL" -c "INSERT INTO schema_migrations (version) VALUES ('$migration_name');" 2>/dev/null
            log_info "Migration applied successfully: $migration_name"
            migration_count=$((migration_count + 1))
        else
            log_error "Migration failed: $migration_name"
            return 1
        fi
    done

    if [ $migration_count -gt 0 ]; then
        log_info "Applied $migration_count migration(s)"
    else
        log_info "No new migrations to apply"
    fi

    return 0
}

# Seed initial data if database is empty
seed_initial_data() {
    log_info "Checking for initial data..."

    # Check if any users exist
    user_count=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM users WHERE deleted_at IS NULL;" 2>/dev/null | tr -d ' ')

    if [ "${user_count:-0}" -eq 0 ]; then
        log_info "No users found, seeding initial admin user..."

        # Generate random password if not provided
        ADMIN_PASSWORD="${ADMIN_PASSWORD:-$(openssl rand -base64 12)}"
        ADMIN_EMAIL="${ADMIN_EMAIL:-admin@localhost}"
        ADMIN_USERNAME="${ADMIN_USERNAME:-admin}"

        # Hash password using bcrypt (via Python if available)
        if command -v python3 &> /dev/null; then
            HASHED_PASSWORD=$(python3 -c "import bcrypt; print(bcrypt.hashpw(b'${ADMIN_PASSWORD}', bcrypt.gensalt()).decode())" 2>/dev/null || echo "")
        fi

        if [ -n "$HASHED_PASSWORD" ]; then
            psql "$DATABASE_URL" -c "
                INSERT INTO users (id, email, username, password_hash, display_name, role, status, created_at, updated_at)
                VALUES (
                    gen_random_uuid(),
                    '${ADMIN_EMAIL}',
                    '${ADMIN_USERNAME}',
                    '${HASHED_PASSWORD}',
                    'Administrator',
                    'administrator',
                    'active',
                    NOW(),
                    NOW()
                )
                ON CONFLICT (email) DO NOTHING;
            " 2>/dev/null

            log_info "Initial admin user created:"
            log_info "  Email: ${ADMIN_EMAIL}"
            log_info "  Username: ${ADMIN_USERNAME}"
            log_info "  Password: ${ADMIN_PASSWORD}"
            log_warn "Please change the admin password after first login!"
        else
            log_warn "Could not create admin user (bcrypt not available)"
        fi
    else
        log_info "Users already exist, skipping seed"
    fi

    # Check if site options exist
    option_count=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM options WHERE option_name = 'site_name';" 2>/dev/null | tr -d ' ')

    if [ "${option_count:-0}" -eq 0 ]; then
        log_info "Seeding default site options..."

        psql "$DATABASE_URL" -c "
            INSERT INTO options (option_name, option_value, autoload) VALUES
            ('site_name', '\"RustPress\"', true),
            ('site_description', '\"A high-performance content management system\"', true),
            ('site_url', '\"${RUSTPRESS_SITE_URL:-http://localhost:8080}\"', true),
            ('admin_email', '\"${ADMIN_EMAIL:-admin@localhost}\"', true),
            ('posts_per_page', '10', true),
            ('date_format', '\"Y-m-d\"', true),
            ('time_format', '\"H:i\"', true),
            ('timezone', '\"UTC\"', true),
            ('default_role', '\"subscriber\"', true)
            ON CONFLICT (option_name) DO NOTHING;
        " 2>/dev/null

        log_info "Default site options created"
    fi
}

# Main entrypoint logic
main() {
    log_info "Starting RustPress..."
    log_info "Version: ${RUSTPRESS_VERSION:-unknown}"

    # Parse DATABASE_URL for pg_isready
    if [ -n "$DATABASE_URL" ]; then
        # Extract host from DATABASE_URL
        export PGHOST=$(echo "$DATABASE_URL" | sed -n 's/.*@\([^:\/]*\).*/\1/p')
        export PGPORT=$(echo "$DATABASE_URL" | sed -n 's/.*:\([0-9]*\)\/.*/\1/p')
        export PGUSER=$(echo "$DATABASE_URL" | sed -n 's/.*\/\/\([^:]*\):.*/\1/p')
    fi

    # Wait for database
    if ! wait_for_postgres; then
        log_error "Database not available, exiting"
        exit 1
    fi

    # Run migrations unless explicitly disabled
    if [ "${SKIP_MIGRATIONS:-false}" != "true" ]; then
        if ! run_migrations; then
            log_error "Migrations failed, exiting"
            exit 1
        fi
    else
        log_warn "Skipping migrations (SKIP_MIGRATIONS=true)"
    fi

    # Seed initial data unless explicitly disabled
    if [ "${SKIP_SEED:-false}" != "true" ]; then
        seed_initial_data
    else
        log_warn "Skipping seed (SKIP_SEED=true)"
    fi

    log_info "Starting RustPress server..."

    # Execute the main command
    exec "$@"
}

# Run main with all arguments
main "$@"
