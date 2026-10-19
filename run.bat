@echo off
chcp 65001 > nul
rem Gantry Crane Sandbox - 개발 실행 (CPU, ML 포함). GPU: run_vulkan.bat / run_rocm.bat
call "%~dp0_env_cpu.bat"
if errorlevel 1 exit /b 1
if not exist "%~dp0node_modules" (
    echo [SETUP] npm install...
    call npm install
    if errorlevel 1 exit /b 1
)
set "TAURI_CLI=cargo tauri"
if exist "%~dp0node_modules\.bin\tauri.cmd" set TAURI_CLI="%~dp0node_modules\.bin\tauri.cmd"
cd /d "%~dp0src-tauri"
echo [DEV] Tauri 앱 시작 (CPU)...
call %TAURI_CLI% dev
