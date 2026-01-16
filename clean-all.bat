@echo off
echo ============================================
echo  RustPress - Clean All Builds
echo ============================================
echo.

cd /d "%~dp0"

echo [1/4] Cleaning Rust target directory...
if exist "target" (
    rd /s /q "target"
    echo       Removed: target/
) else (
    echo       Skipped: target/ (not found)
)

echo.
echo [2/4] Cleaning admin-ui build artifacts...
if exist "admin-ui\node_modules" (
    rd /s /q "admin-ui\node_modules"
    echo       Removed: admin-ui/node_modules/
) else (
    echo       Skipped: admin-ui/node_modules/ (not found)
)

if exist "admin-ui\dist" (
    rd /s /q "admin-ui\dist"
    echo       Removed: admin-ui/dist/
) else (
    echo       Skipped: admin-ui/dist/ (not found)
)

if exist "admin-ui\.vite" (
    rd /s /q "admin-ui\.vite"
    echo       Removed: admin-ui/.vite/
) else (
    echo       Skipped: admin-ui/.vite/ (not found)
)

echo.
echo [3/4] Cleaning plugin target directories...
for /d %%p in (plugins\*) do (
    if exist "%%p\target" (
        rd /s /q "%%p\target"
        echo       Removed: %%p\target/
    )
)

echo.
echo [4/4] Cleaning temporary/cache files...
if exist "Cargo.lock" (
    echo       Keeping: Cargo.lock (for reproducible builds)
)

for /r %%f in (*.log) do (
    del /q "%%f" 2>nul
)
echo       Cleaned: *.log files

echo.
echo ============================================
echo  Clean complete!
echo ============================================
pause
