# RustPress Installer for Windows
# Usage: irm https://raw.githubusercontent.com/rustpress/rustpress/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$REPO = "rustpress-net/core"
$INSTALL_DIR = if ($env:RUSTPRESS_INSTALL_DIR) { $env:RUSTPRESS_INSTALL_DIR } else { "$env:LOCALAPPDATA\RustPress\bin" }

function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Success { param($msg) Write-Host "[SUCCESS] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-Error { param($msg) Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }

function Show-Banner {
    Write-Host ""
    Write-Host "  ____            _   ____                    " -ForegroundColor Blue
    Write-Host " |  _ \ _   _ ___| |_|  _ \ _ __ ___  ___ ___ " -ForegroundColor Blue
    Write-Host " | |_) | | | / __| __| |_) | '__/ _ \/ __/ __|" -ForegroundColor Blue
    Write-Host " |  _ <| |_| \__ \ |_|  __/| | |  __/\__ \__ \" -ForegroundColor Blue
    Write-Host " |_| \_\\__,_|___/\__|_|   |_|  \___||___/___/" -ForegroundColor Blue
    Write-Host ""
    Write-Host "  High-Performance CMS Built with Rust" -ForegroundColor White
    Write-Host ""
}

function Get-LatestVersion {
    Write-Info "Fetching latest release..."

    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest" -UseBasicParsing
        $version = $response.tag_name
        Write-Info "Latest version: $version"
        return $version
    }
    catch {
        Write-Error "Failed to fetch latest version: $_"
    }
}

function Install-RustPress {
    param($Version)

    $platform = "windows-x86_64"
    $filename = "rustpress-$platform-$Version.zip"
    $url = "https://github.com/$REPO/releases/download/$Version/$filename"
    $tmpDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
    $zipPath = Join-Path $tmpDir $filename

    try {
        Write-Info "Downloading RustPress $Version for Windows..."
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

        Write-Info "Extracting archive..."
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        # Create install directory if it doesn't exist
        if (!(Test-Path $INSTALL_DIR)) {
            New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
        }

        # Copy binary
        $binaryPath = Join-Path $tmpDir "rustpress.exe"
        Copy-Item $binaryPath -Destination $INSTALL_DIR -Force

        Write-Success "RustPress installed to $INSTALL_DIR\rustpress.exe"
    }
    catch {
        Write-Error "Installation failed: $_"
    }
    finally {
        # Cleanup
        if (Test-Path $tmpDir) {
            Remove-Item -Recurse -Force $tmpDir
        }
    }
}

function Add-ToPath {
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

    if ($currentPath -notlike "*$INSTALL_DIR*") {
        Write-Info "Adding $INSTALL_DIR to PATH..."
        [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$INSTALL_DIR", "User")
        $env:PATH = "$env:PATH;$INSTALL_DIR"
        Write-Success "Added to PATH. You may need to restart your terminal."
    }
}

function Verify-Installation {
    $rustpressPath = Join-Path $INSTALL_DIR "rustpress.exe"

    if (Test-Path $rustpressPath) {
        try {
            $version = & $rustpressPath --version 2>$null
            Write-Success "RustPress is ready! Version: $version"
        }
        catch {
            Write-Success "RustPress installed successfully!"
        }
    }
    else {
        Write-Warn "Installation file not found at expected location"
    }
}

# Main
Show-Banner

$version = Get-LatestVersion
Install-RustPress -Version $version
Add-ToPath
Verify-Installation

Write-Host ""
Write-Info "Quick Start:"
Write-Host "  1. Set up your database: rustpress-migrate"
Write-Host "  2. Start the server: rustpress"
Write-Host "  3. Visit http://localhost:8080"
Write-Host ""
Write-Info "Documentation: https://github.com/$REPO"
Write-Host ""
