@echo off
REM RustPress Build Script with Visual Studio Environment
REM This script sets up the correct Visual Studio environment for compiling native dependencies

set VCToolsVersion=14.44.35207
set VCToolsInstallDir=C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC\%VCToolsVersion%
set WindowsSDKDir=C:\Program Files (x86)\Windows Kits\10\
set WindowsSDKVersion=10.0.22621.0

set INCLUDE=%VCToolsInstallDir%\include;%WindowsSDKDir%include\%WindowsSDKVersion%\ucrt;%WindowsSDKDir%include\%WindowsSDKVersion%\um;%WindowsSDKDir%include\%WindowsSDKVersion%\shared
set LIB=%VCToolsInstallDir%\lib\x64;%WindowsSDKDir%lib\%WindowsSDKVersion%\ucrt\x64;%WindowsSDKDir%lib\%WindowsSDKVersion%\um\x64
set PATH=%VCToolsInstallDir%\bin\HostX64\x64;%PATH%

echo Visual Studio environment configured:
echo   VCToolsVersion: %VCToolsVersion%
echo   Target: x86_64-pc-windows-msvc
echo.

if "%1"=="" (
    echo Running: cargo check
    cargo check
) else (
    echo Running: cargo %*
    cargo %*
)

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Build completed successfully!
) else (
    echo.
    echo Build failed with exit code %ERRORLEVEL%
)
