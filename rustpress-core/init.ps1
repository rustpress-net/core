# RustPress Initialization Script for Windows
# Downloads all required dependencies and sets up the project

$ErrorActionPreference = "Stop"

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "RustPress Initialization" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

# Read manifest
$Manifest = "rustpress-manifest.json"

if (-not (Test-Path $Manifest)) {
    Write-Host "Error: $Manifest not found" -ForegroundColor Red
    exit 1
}

# Check for required tools
try {
    git --version | Out-Null
} catch {
    Write-Host "Error: git is required" -ForegroundColor Red
    exit 1
}

try {
    cargo --version | Out-Null
} catch {
    Write-Host "Error: cargo/Rust is required" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Downloading RustPress dependencies..." -ForegroundColor Yellow
Write-Host ""

# Clone admin-ui
if (-not (Test-Path "admin-ui")) {
    Write-Host "Cloning admin-ui..." -ForegroundColor Green
    git clone https://github.com/rustpress-net/admin-ui.git admin-ui
} else {
    Write-Host "admin-ui already exists, pulling latest..." -ForegroundColor Green
    Set-Location admin-ui
    git pull
    Set-Location ..
}

# Clone crates
if (-not (Test-Path "crates")) {
    Write-Host "Cloning crates..." -ForegroundColor Green
    git clone https://github.com/rustpress-net/crates.git crates
} else {
    Write-Host "crates already exists, pulling latest..." -ForegroundColor Green
    Set-Location crates
    git pull
    Set-Location ..
}

# Optional: Clone themes
$downloadThemes = Read-Host "Download themes? (y/n)"
if ($downloadThemes -eq "y") {
    if (-not (Test-Path "themes")) {
        Write-Host "Cloning themes..." -ForegroundColor Green
        git clone https://github.com/rustpress-net/themes.git themes 2>$null
        if (-not $?) {
            Write-Host "Themes repository not available yet" -ForegroundColor Yellow
        }
    }
}

# Optional: Clone plugins
$downloadPlugins = Read-Host "Download plugins? (y/n)"
if ($downloadPlugins -eq "y") {
    if (-not (Test-Path "plugins")) {
        Write-Host "Cloning plugins..." -ForegroundColor Green
        git clone https://github.com/rustpress-net/plugins.git plugins 2>$null
        if (-not $?) {
            Write-Host "Plugins repository not available yet" -ForegroundColor Yellow
        }
    }
}

Write-Host ""
Write-Host "Installing Node.js dependencies for admin-ui..." -ForegroundColor Yellow
Set-Location admin-ui
try {
    npm install
    npm run build
} catch {
    Write-Host "Warning: npm not found or failed, skipping admin-ui build" -ForegroundColor Yellow
}
Set-Location ..

Write-Host ""
Write-Host "Building RustPress..." -ForegroundColor Yellow
cargo build --release

Write-Host ""
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "RustPress initialization complete!" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor White
Write-Host "1. Copy .env.example to .env and configure your database" -ForegroundColor White
Write-Host "2. Run: cargo run --bin migrate" -ForegroundColor White
Write-Host "3. Run: cargo run --release" -ForegroundColor White
Write-Host ""
