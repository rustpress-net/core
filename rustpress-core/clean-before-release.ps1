# =============================================================================
# RustPress - Clean before release
# =============================================================================
# Removes build artifacts to save disk space before creating a new release
# =============================================================================

$ProjectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Write-Host "RustPress - Cleaning build artifacts..." -ForegroundColor Cyan
Write-Host "Project directory: $ProjectDir" -ForegroundColor Gray

# Remove target directory
$targetPath = Join-Path $ProjectDir "target"
if (Test-Path $targetPath) {
    Write-Host "Removing target directory..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $targetPath -ErrorAction SilentlyContinue
    Write-Host "  Target directory removed" -ForegroundColor Green
} else {
    Write-Host "  Target directory not found (already clean)" -ForegroundColor Gray
}

# Remove any temporary build directories
$tempDirs = @(
    "C:\temp\rustpress-*",
    "$env:TEMP\rustpress-*",
    "/tmp/rustpress-*"
)

foreach ($pattern in $tempDirs) {
    $dirs = Get-Item $pattern -ErrorAction SilentlyContinue
    foreach ($dir in $dirs) {
        Write-Host "Removing $($dir.FullName)..." -ForegroundColor Yellow
        Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "Cleanup complete!" -ForegroundColor Green
