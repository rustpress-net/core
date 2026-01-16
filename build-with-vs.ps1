# RustPress Build Script with Visual Studio Environment
# This script sets up the correct Visual Studio environment for compiling native dependencies

param(
    [string]$Command = "check",
    [string]$Package = ""
)

# Visual Studio paths
$VCToolsVersion = '14.44.35207'
$VCToolsInstallDir = "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC\$VCToolsVersion"
$WindowsSDKDir = 'C:\Program Files (x86)\Windows Kits\10\'
$WindowsSDKVersion = '10.0.22621.0'

# Set environment variables for C compilation
$env:VCToolsVersion = $VCToolsVersion
$env:VCToolsInstallDir = "$VCToolsInstallDir\"
$env:INCLUDE = "$VCToolsInstallDir\include;$WindowsSDKDir\include\$WindowsSDKVersion\ucrt;$WindowsSDKDir\include\$WindowsSDKVersion\um;$WindowsSDKDir\include\$WindowsSDKVersion\shared"
$env:LIB = "$VCToolsInstallDir\lib\x64;$WindowsSDKDir\lib\$WindowsSDKVersion\ucrt\x64;$WindowsSDKDir\lib\$WindowsSDKVersion\um\x64"
$env:PATH = "$VCToolsInstallDir\bin\HostX64\x64;$env:PATH"

Write-Host "Visual Studio environment configured:" -ForegroundColor Green
Write-Host "  VCToolsVersion: $VCToolsVersion"
Write-Host "  Target: x86_64-pc-windows-msvc"
Write-Host ""

# Build command
$cargoArgs = @($Command)
if ($Package) {
    $cargoArgs += "-p"
    $cargoArgs += $Package
}

Write-Host "Running: cargo $($cargoArgs -join ' ')" -ForegroundColor Cyan
& cargo @cargoArgs

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nBuild completed successfully!" -ForegroundColor Green
} else {
    Write-Host "`nBuild failed with exit code $LASTEXITCODE" -ForegroundColor Red
}
