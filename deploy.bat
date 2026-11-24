@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul

cd /d "%~dp0"

echo ==================================================
echo  Sekonic Studio - Version Increment ^& Deploy
echo  Target: https://sekonic.dijios.com/
echo ==================================================
echo.

echo [1/6] Incrementing version (10 patch = 1 minor, 10 minor = 1 major)...
for /f "usebackq delims=" %%V in (`powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\bump-version.ps1"`) do set "VER_CHANGE=%%V"
if errorlevel 1 (
    echo [ERROR] Version increment failed!
    exit /b 1
)
for %%A in (%VER_CHANGE%) do set "NEW_VER=%%A"
echo       Version updated: %VER_CHANGE% (v%NEW_VER%)
echo.

echo [2/6] Building packages\sekonic-web-sdk...
pushd packages\sekonic-web-sdk
call npm run build
if errorlevel 1 (
    popd
    echo [ERROR] sekonic-web-sdk build failed!
    exit /b 1
)
popd
echo.

echo [3/6] Building universal 32-bit Windows 7/8.1/10/11 sekonic-agent.exe...
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

echo [4/6] Building apps\sekonic-bridge...
pushd apps\sekonic-bridge
call npm run build
if errorlevel 1 (
    popd
    echo [ERROR] sekonic-bridge build failed!
    exit /b 1
)
popd
echo.

echo [5/6] Deploying dist to truncgil_ubuntu (https://sekonic.dijios.com/)...
set "REMOTE_DIR=/home/truncgil/web/sekonic.dijios.com/public_html"
tar -czf - -C apps\sekonic-bridge\dist . | ssh truncgil_ubuntu "mkdir -p %REMOTE_DIR% && find %REMOTE_DIR% -mindepth 1 -delete && tar -xzf - -C %REMOTE_DIR% && chown -R truncgil:www-data %REMOTE_DIR%"
if errorlevel 1 (
    echo [ERROR] Deployment over SSH failed!
    exit /b 1
)
echo.

echo [6/6] Committing release changes and tagging v%NEW_VER%...
git add -A
git diff --cached --quiet
if errorlevel 1 (
    git commit -m "chore(release): v%NEW_VER%"
    if errorlevel 1 (
        echo [ERROR] Git commit failed!
        exit /b 1
    )
) else (
    echo       No working tree changes to commit.
)
git rev-parse -q --verify "refs/tags/v%NEW_VER%" >nul 2>&1
if errorlevel 1 (
    git tag -a "v%NEW_VER%" -m "Release v%NEW_VER%"
) else (
    git tag -f -a "v%NEW_VER%" -m "Release v%NEW_VER%"
)
if errorlevel 1 (
    echo [ERROR] Git tag failed!
    exit /b 1
)
echo       Tagged: v%NEW_VER%

echo.
echo ==================================================
echo  Deploy complete! (%VER_CHANGE% / tag v%NEW_VER%)
echo  URL: https://sekonic.dijios.com/
echo ==================================================
endlocal
