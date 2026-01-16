# RustPress - Clean All Builds
# Run with: powershell -ExecutionPolicy Bypass -File clean-all.ps1

$ErrorActionPreference = "SilentlyContinue"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  RustPress - Clean All Builds" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

$totalFreed = 0

function Remove-BuildDir {
    param([string]$Path, [string]$Description)

    if (Test-Path $Path) {
        $size = (Get-ChildItem $Path -Recurse -ErrorAction SilentlyContinue |
                 Measure-Object -Property Length -Sum).Sum / 1MB
        Remove-Item -Path $Path -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  [REMOVED] $Description" -ForegroundColor Green
        Write-Host "            Freed: $([math]::Round($size, 2)) MB" -ForegroundColor DarkGray
        return $size
    } else {
        Write-Host "  [SKIPPED] $Description (not found)" -ForegroundColor DarkGray
        return 0
    }
}

# 1. Clean main Rust target
Write-Host "[1/4] Cleaning Rust target directory..." -ForegroundColor Yellow
$totalFreed += Remove-BuildDir "target" "target/"

# 2. Clean admin-ui
Write-Host ""
Write-Host "[2/4] Cleaning admin-ui build artifacts..." -ForegroundColor Yellow
$totalFreed += Remove-BuildDir "admin-ui\node_modules" "admin-ui/node_modules/"
$totalFreed += Remove-BuildDir "admin-ui\dist" "admin-ui/dist/"
$totalFreed += Remove-BuildDir "admin-ui\.vite" "admin-ui/.vite/"

# 3. Clean plugin targets
Write-Host ""
Write-Host "[3/4] Cleaning plugin target directories..." -ForegroundColor Yellow
Get-ChildItem -Path "plugins" -Directory -ErrorAction SilentlyContinue | ForEach-Object {
    $pluginTarget = Join-Path $_.FullName "target"
    $totalFreed += Remove-BuildDir $pluginTarget "plugins/$($_.Name)/target/"
}

# 4. Clean misc cache files
Write-Host ""
Write-Host "[4/4] Cleaning temporary files..." -ForegroundColor Yellow
$logFiles = Get-ChildItem -Path . -Filter "*.log" -Recurse -ErrorAction SilentlyContinue
if ($logFiles) {
    $logFiles | Remove-Item -Force -ErrorAction SilentlyContinue
    Write-Host "  [REMOVED] $($logFiles.Count) log file(s)" -ForegroundColor Green
}

# Summary
Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Clean complete!" -ForegroundColor Green
Write-Host "  Total space freed: $([math]::Round($totalFreed, 2)) MB" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""
