@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul

cd /d "%~dp0"

echo ==================================================
echo  Sekonic Studio - Version Increment ^& Deploy
echo  Target: https://sekonic.dijios.com/
echo ==================================================
echo.

echo [1/4] Incrementing version (10 patch = 1 minor, 10 minor = 1 major)...
for /f "usebackq delims=" %%V in (`powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\bump-version.ps1"`) do set "VER_CHANGE=%%V"
if errorlevel 1 (
    echo [ERROR] Version increment failed!
    exit /b 1
)
echo       Version updated: %VER_CHANGE%
echo.

echo [2/5] Building packages\sekonic-web-sdk...
pushd packages\sekonic-web-sdk
call npm run build
if errorlevel 1 (
    popd
    echo [ERROR] sekonic-web-sdk build failed!
    exit /b 1
)
popd
echo.

echo [3/5] Building universal 32-bit Windows 7/8.1/10/11 sekonic-agent.exe...
set "RUSTFLAGS=-C target-feature=+crt-static"
cargo build --release --target i686-pc-windows-msvc -p sekonic-agent
set "AGENT_ERR=!errorlevel!"
set "RUSTFLAGS="
if not "!AGENT_ERR!"=="0" (
    echo [ERROR] sekonic-agent build failed!
    exit /b 1
)
if not exist "apps\sekonic-bridge\public\downloads" mkdir "apps\sekonic-bridge\public\downloads"
copy /Y "target\i686-pc-windows-msvc\release\sekonic-agent.exe" "apps\sekonic-bridge\public\downloads\sekonic-agent.exe" >nul
echo.

echo [4/5] Building apps\sekonic-bridge...
pushd apps\sekonic-bridge
call npm run build
if errorlevel 1 (
    popd
    echo [ERROR] sekonic-bridge build failed!
    exit /b 1
)
popd
echo.

echo [5/5] Deploying dist to truncgil_ubuntu (https://sekonic.dijios.com/)...
set "REMOTE_DIR=/home/truncgil/web/sekonic.dijios.com/public_html"
tar -czf - -C apps\sekonic-bridge\dist . | ssh truncgil_ubuntu "mkdir -p %REMOTE_DIR% && find %REMOTE_DIR% -mindepth 1 -delete && tar -xzf - -C %REMOTE_DIR% && chown -R truncgil:www-data %REMOTE_DIR%"
if errorlevel 1 (
    echo [ERROR] Deployment over SSH failed!
    exit /b 1
)

echo.
echo ==================================================
echo  Deploy complete! (%VER_CHANGE%)
echo  URL: https://sekonic.dijios.com/
echo ==================================================
endlocal
