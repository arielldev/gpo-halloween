@echo off
setlocal
cd /d "%~dp0"
title GPO Halloween - Build installer
set "DISCORD=https://discord.gg/unPZxXAtfb"
set "KEY=%USERPROFILE%\.tauri\gpo-halloween.key"
set "OUT=%~dp0src-tauri\target\release\bundle\nsis"

echo ========================================
echo  GPO Halloween - Build installer
echo ========================================
echo.

where node >nul 2>&1
if errorlevel 1 (
    echo [!] Node.js not found. Install Node 20+ from https://nodejs.org and run this again.
    call :fallback
    pause
    exit /b 1
)
where cargo >nul 2>&1
if errorlevel 1 (
    echo [!] Rust not found. Install it from https://rustup.rs and run this again.
    call :fallback
    pause
    exit /b 1
)

echo [1/4] Installing packages...
call npm install
if errorlevel 1 (
    echo [!] npm install failed.
    call :fallback
    pause
    exit /b 1
)

echo [2/4] Update signing key...
if exist "%KEY%" (
    echo       Using %KEY%
) else (
    echo       None found, generating one at %KEY%
    if not exist "%USERPROFILE%\.tauri" mkdir "%USERPROFILE%\.tauri"
    call npx tauri signer generate -w "%KEY%" --ci -p ""
    if errorlevel 1 (
        echo [!] Could not generate a signing key.
        call :fallback
        pause
        exit /b 1
    )
)
set /p TAURI_SIGNING_PRIVATE_KEY=<"%KEY%"
set "TAURI_SIGNING_PRIVATE_KEY_PASSWORD="

echo [3/4] Building. First build takes a few minutes...
call npm run app:build
if errorlevel 1 (
    echo [!] Build failed. See the output above.
    call :fallback
    pause
    exit /b 1
)

echo [4/4] Writing latest.json for the auto-updater...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0src-tauri\latest-json.ps1" -Out "%OUT%"
if errorlevel 1 echo       Skipped: no signed installer found.

echo.
echo ========================================
echo  Done. Installer is in:
echo  %OUT%
echo ========================================
echo.
echo To publish a release on GitHub, tag it  v[version]  and upload:
echo   - GPO Halloween_[version]_x64-setup.exe
echo   - latest.json   (what installed copies poll; its signature field already covers the exe)
echo.
echo Forks: put your own key's .pub content and your repo URL in
echo src-tauri\tauri.conf.json under plugins.updater, or installed
echo copies will keep updating from the original project.
start "" "%OUT%"
pause
exit /b 0

:fallback
echo.
echo     Don't want to build it yourself? Grab the ready-made installer
echo     from GitHub Releases or from our Discord: %DISCORD%
echo.
exit /b 0
