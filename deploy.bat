@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul

cd /d "%~dp0"

echo ==================================================
echo  Sekonic Studio - Version Increment ^& Deploy
echo  Target: https://seconic.dijios.com/
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

echo [2/4] Building packages\sekonic-web-sdk...
pushd packages\sekonic-web-sdk
call npm run build
if errorlevel 1 (
    popd
    echo [ERROR] sekonic-web-sdk build failed!
    exit /b 1
)
popd
echo.

echo [3/4] Building apps\sekonic-bridge...
pushd apps\sekonic-bridge
call npm run build
if errorlevel 1 (
    popd
    echo [ERROR] sekonic-bridge build failed!
    exit /b 1
)
popd
echo.

echo [4/4] Deploying dist to truncgil_ubuntu (https://seconic.dijios.com/)...
set "REMOTE_DIR=/home/truncgil/web/seconic.dijios.com/public_html"
tar -czf - -C apps\sekonic-bridge\dist . | ssh truncgil_ubuntu "mkdir -p %REMOTE_DIR% && find %REMOTE_DIR% -mindepth 1 -delete && tar -xzf - -C %REMOTE_DIR% && chown -R truncgil:www-data %REMOTE_DIR%"
if errorlevel 1 (
    echo [ERROR] Deployment over SSH failed!
    exit /b 1
)

echo.
echo ==================================================
echo  Deploy complete! (%VER_CHANGE%)
echo  URL: https://seconic.dijios.com/
echo ==================================================
endlocal
