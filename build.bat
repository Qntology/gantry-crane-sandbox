@echo off
chcp 65001 > nul
rem Gantry Crane Sandbox - 릴리스 빌드 (CPU)
call "%~dp0_env_cpu.bat"
if errorlevel 1 exit /b 1
if not exist "%~dp0node_modules" (
    call npm install
    if errorlevel 1 exit /b 1
)
set "TAURI_CLI=cargo tauri"
if exist "%~dp0node_modules\.bin\tauri.cmd" set TAURI_CLI="%~dp0node_modules\.bin\tauri.cmd"
cd /d "%~dp0src-tauri"
call %TAURI_CLI% build
