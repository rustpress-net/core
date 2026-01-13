#!/bin/bash
# RustPress Installer
# Usage: curl -sSL https://raw.githubusercontent.com/rustpress/rustpress/main/install.sh | bash

set -e

REPO="rustpress-net/core"
INSTALL_DIR="${RUSTPRESS_INSTALL_DIR:-/usr/local/bin}"
TMP_DIR=$(mktemp -d)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux*) OS="linux" ;;
        darwin*) OS="macos" ;;
        *) error "Unsupported operating system: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        *) error "Unsupported architecture: $arch" ;;
    esac

    # macOS ARM uses different naming
    if [ "$OS" = "macos" ] && [ "$ARCH" = "arm64" ]; then
        PLATFORM="macos-arm64"
    elif [ "$OS" = "macos" ]; then
        PLATFORM="macos-x86_64"
    else
        PLATFORM="linux-x86_64"
    fi

    info "Detected platform: $PLATFORM"
}

# Get latest release version
get_latest_version() {
    info "Fetching latest release..."

    if command -v curl &> /dev/null; then
        VERSION=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget &> /dev/null; then
        VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        error "curl or wget is required to download RustPress"
    fi

    if [ -z "$VERSION" ]; then
        error "Failed to get latest version"
    fi

    info "Latest version: $VERSION"
}

# Download and install
download_and_install() {
    local filename="rustpress-${PLATFORM}-${VERSION}.zip"
    local url="https://github.com/$REPO/releases/download/$VERSION/$filename"

    info "Downloading RustPress $VERSION for $PLATFORM..."

    cd "$TMP_DIR"

    if command -v curl &> /dev/null; then
        curl -sLO "$url" || error "Failed to download from $url"
    else
        wget -q "$url" || error "Failed to download from $url"
    fi

    info "Extracting archive..."
    unzip -q "$filename" || error "Failed to extract archive"

    # Make binary executable
    chmod +x rustpress

    # Check if we need sudo
    if [ -w "$INSTALL_DIR" ]; then
        mv rustpress "$INSTALL_DIR/"
    else
        warn "Installing to $INSTALL_DIR requires sudo..."
        sudo mv rustpress "$INSTALL_DIR/"
    fi

    success "RustPress installed to $INSTALL_DIR/rustpress"
}

# Verify installation
verify_installation() {
    if command -v rustpress &> /dev/null; then
        local installed_version=$(rustpress --version 2>/dev/null || echo "unknown")
        success "RustPress is ready! Version: $installed_version"
    else
        warn "Installation complete but 'rustpress' not found in PATH"
        info "Add $INSTALL_DIR to your PATH or run: $INSTALL_DIR/rustpress"
    fi
}

# Main
main() {
    echo ""
    echo "  ____            _   ____                    "
    echo " |  _ \ _   _ ___| |_|  _ \ _ __ ___  ___ ___ "
    echo " | |_) | | | / __| __| |_) | '__/ _ \/ __/ __|"
    echo " |  _ <| |_| \__ \ |_|  __/| | |  __/\__ \__ \\"
    echo " |_| \_\\\\__,_|___/\__|_|   |_|  \___||___/___/"
    echo ""
    echo "  High-Performance CMS Built with Rust"
    echo ""

    # Check for unzip
    if ! command -v unzip &> /dev/null; then
        error "unzip is required. Please install it first."
    fi

    detect_platform
    get_latest_version
    download_and_install
    verify_installation

    echo ""
    info "Quick Start:"
    echo "  1. Set up your database: rustpress-migrate"
    echo "  2. Start the server: rustpress"
    echo "  3. Visit http://localhost:8080"
    echo ""
    info "Documentation: https://github.com/$REPO"
    echo ""
}

main "$@"
