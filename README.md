# RustPress

A modern, high-performance Content Management System (CMS) built with Rust. RustPress is a WordPress alternative designed for speed, security, and scalability.

## Features

- **High Performance**: Built with Rust and Axum for blazing-fast response times
- **Multi-tenant Support**: Host multiple sites from a single installation
- **Plugin Architecture**: Extensible through a robust plugin system
- **Theme System**: Full theme support with template engine
- **RESTful API**: Complete REST API for headless CMS usage
- **Security First**: Built-in security middleware with:
  - SQL injection protection
  - XSS prevention
  - CSRF protection
  - Rate limiting
  - Bot detection
  - Request fingerprinting
- **Modern Stack**: PostgreSQL, Redis caching, async I/O

## Requirements

- PostgreSQL 15+
- Redis 7+

## Installation

### One-Line Install (Recommended)

**Linux / macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/rustpress/rustpress/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/rustpress/rustpress/main/install.ps1 | iex
```

### Using Cargo

If you have Rust installed:
```bash
cargo install rustpress
```

### Using Docker

```bash
docker run -d -p 8080:8080 ghcr.io/rustpress/rustpress:latest
```

### From Source

```bash
# Clone the repository
git clone https://github.com/rustpress-net/core.git
cd rustpress

# Set up environment variables
cp .env.example .env
# Edit .env with your database credentials

# Run migrations
cargo run --bin rustpress-migrate

# Start the server
cargo run --bin rustpress
```

## Quick Start

After installation:

```bash
# 1. Set up your database
rustpress-migrate

# 2. Start the server
rustpress

# 3. Visit http://localhost:8080
```

## Project Structure

```
rustpress/
├── crates/
│   ├── rustpress-core/       # Core types and traits
│   ├── rustpress-database/   # Database layer
│   ├── rustpress-auth/       # Authentication & authorization
│   ├── rustpress-api/        # REST API handlers
│   ├── rustpress-server/     # HTTP server & middleware
│   ├── rustpress-plugins/    # Plugin system
│   ├── rustpress-themes/     # Theme system
│   ├── rustpress-cache/      # Caching layer
│   ├── rustpress-storage/    # File storage
│   ├── rustpress-jobs/       # Background jobs
│   ├── rustpress-events/     # Event system
│   ├── rustpress-content/    # Content management
│   ├── rustpress-users/      # User management
│   ├── rustpress-media/      # Media handling
│   ├── rustpress-admin/      # Admin dashboard
│   ├── rustpress-cli/        # CLI tools
│   ├── rustpress-health/     # Health checks
│   ├── rustpress-cdn/        # CDN integration
│   └── rustpress-performance/# Performance utilities
└── Cargo.toml
```

## Configuration

RustPress uses environment variables for configuration:

```env
# Database
DATABASE_URL=postgres://user:password@localhost:5432/rustpress

# Redis
REDIS_URL=redis://localhost:6379

# Server
HOST=127.0.0.1
PORT=3000

# Security
JWT_SECRET=your-secret-key
SESSION_SECRET=your-session-secret
```

## Plugins

Plugins are distributed as separate packages. Install them by adding to your RustPress installation:

- [rustpress-plugin-rustbackup](https://github.com/rustpress-net/core-plugin-rustbackup) - Backup & restore
- [rustpress-plugin-rustseo](https://github.com/rustpress-net/core-plugin-rustseo) - SEO optimization
- [rustpress-plugin-rustanalytics](https://github.com/rustpress-net/core-plugin-rustanalytics) - Analytics
- [rustpress-plugin-rustcommerce](https://github.com/rustpress-net/core-plugin-rustcommerce) - E-commerce
- [rustpress-plugin-rustforms](https://github.com/rustpress-net/core-plugin-rustforms) - Form builder
- [rustpress-plugin-rustmail](https://github.com/rustpress-net/core-plugin-rustmail) - Email integration
- [rustpress-plugin-rustmedia](https://github.com/rustpress-net/core-plugin-rustmedia) - Media management
- [rustpress-plugin-rustsecurity](https://github.com/rustpress-net/core-plugin-rustsecurity) - Security enhancements
- [rustpress-plugin-rustusers](https://github.com/rustpress-net/core-plugin-rustusers) - User management

## Development

```bash
# Run tests
cargo test

# Run with hot reload (requires cargo-watch)
cargo watch -x run

# Check code
cargo clippy

# Format code
cargo fmt
```

## API Documentation

The REST API is available at `/api/v1/`. Key endpoints:

- `POST /api/v1/auth/login` - Authentication
- `GET /api/v1/posts` - List posts
- `GET /api/v1/pages` - List pages
- `GET /api/v1/media` - List media
- `GET /api/v1/users` - List users (admin)

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

## License

RustPress is dual-licensed under MIT and Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
