@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"

echo ==================================================
echo  Sekonic Studio - Local Dev Launcher
echo  Agent : ws://127.0.0.1:17441
echo  Web UI: http://localhost:5179/scan
echo ==================================================
echo.

REM Eski bir sekonic-agent.exe aciksa portu bosalt
taskkill /IM sekonic-agent.exe /F >nul 2>&1

echo [1/3] Sekonic Local Agent baslatiliyor (port 17441)...
start "Sekonic Local Agent (17441)" cmd /k "cd /d "%~dp0" && cargo run -p sekonic-agent"

echo [2/3] Sekonic Studio Web UI baslatiliyor (port 5179)...
start "Sekonic Studio UI (5179)" cmd /k "cd /d "%~dp0" && npm --prefix apps\sekonic-bridge run dev"

echo [3/3] Tarayici aciliyor (http://localhost:5179/scan)...
timeout /t 3 /nobreak >nul
start "" "http://localhost:5179/scan"

echo.
echo Baslatildi. Kapatmak icin acilan iki terminal penceresini kapatabilirsiniz.
endlocal

