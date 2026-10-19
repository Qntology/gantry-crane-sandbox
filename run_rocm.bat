@echo off
chcp 65001 > nul
call "%~dp0_env_rocm.bat"
if errorlevel 1 exit /b 1
cd /d "%~dp0src-tauri"
set "PATH=%CD%\dlls;%PATH%"
echo [DEV] Tauri 앱 시작 (rocm)...
call %TAURI_CLI% dev -- --no-default-features --features rocm
