@echo off
setlocal
cd /d "%~dp0"
title GPO Halloween - Build installer
set "OUT=%~dp0src-tauri\target\release\bundle\nsis"

echo ========================================
echo  GPO Halloween - Build installer
echo ========================================
echo.

where node >nul 2>&1
if errorlevel 1 (
    echo [!] Node.js not found. Install Node 22 from https://nodejs.org and run this again.
    pause
    exit /b 1
)
where cargo >nul 2>&1
if errorlevel 1 (
    echo [!] Rust not found. Install it from https://rustup.rs and run this again.
    pause
    exit /b 1
)

echo [1/2] Installing packages...
call npm install
if errorlevel 1 (
    echo [!] npm install failed.
    pause
    exit /b 1
)

echo [2/2] Building. First build takes a few minutes...
call npm run app:build
if errorlevel 1 (
    echo [!] Build failed. See the output above.
    pause
    exit /b 1
)

echo.
echo ========================================
echo  Done. Installer is in:
echo  %OUT%
echo ========================================
start "" "%OUT%"
pause
exit /b 0
