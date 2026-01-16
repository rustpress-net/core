#!/bin/bash

# RustPress Initialization Script
# Downloads all required dependencies and sets up the project

set -e

echo "=================================="
echo "RustPress Initialization"
echo "=================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Read manifest
MANIFEST="rustpress-manifest.json"

if [ ! -f "$MANIFEST" ]; then
    echo "Error: $MANIFEST not found"
    exit 1
fi

# Check for required tools
command -v git >/dev/null 2>&1 || { echo "Error: git is required"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo/Rust is required"; exit 1; }

echo ""
echo "Downloading RustPress dependencies..."
echo ""

# Clone admin-ui
if [ ! -d "admin-ui" ]; then
    echo "Cloning admin-ui..."
    git clone https://github.com/rustpress-net/admin-ui.git admin-ui
else
    echo "admin-ui already exists, pulling latest..."
    cd admin-ui && git pull && cd ..
fi

# Clone crates
if [ ! -d "crates" ]; then
    echo "Cloning crates..."
    git clone https://github.com/rustpress-net/crates.git crates
else
    echo "crates already exists, pulling latest..."
    cd crates && git pull && cd ..
fi

# Optional: Clone themes
read -p "Download themes? (y/n): " download_themes
if [ "$download_themes" = "y" ]; then
    if [ ! -d "themes" ]; then
        echo "Cloning themes..."
        git clone https://github.com/rustpress-net/themes.git themes 2>/dev/null || echo "Themes repository not available yet"
    fi
fi

# Optional: Clone plugins
read -p "Download plugins? (y/n): " download_plugins
if [ "$download_plugins" = "y" ]; then
    if [ ! -d "plugins" ]; then
        echo "Cloning plugins..."
        git clone https://github.com/rustpress-net/plugins.git plugins 2>/dev/null || echo "Plugins repository not available yet"
    fi
fi

echo ""
echo "Installing Node.js dependencies for admin-ui..."
cd admin-ui
if command -v npm >/dev/null 2>&1; then
    npm install
    npm run build
else
    echo "Warning: npm not found, skipping admin-ui build"
fi
cd ..

echo ""
echo "Building RustPress..."
cargo build --release

echo ""
echo "=================================="
echo "RustPress initialization complete!"
echo "=================================="
echo ""
echo "Next steps:"
echo "1. Copy .env.example to .env and configure your database"
echo "2. Run: cargo run --bin migrate"
echo "3. Run: cargo run --release"
echo ""
