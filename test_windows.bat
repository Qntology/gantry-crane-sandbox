@echo off
chcp 65001 > nul
rem 라이브러리 · 통합 테스트 + 정밀 로프 해석기 테스트 (CPU). 인자: vulkan | rocm 이면 그 GPU 기능으로 빌드
set "BACKEND=%~1"
if "%BACKEND%"=="" set "BACKEND=cpu"
call "%~dp0_env_%BACKEND%.bat"
if errorlevel 1 exit /b 1
cd /d "%~dp0"
if /i "%BACKEND%"=="cpu" (
    cargo test
) else (
    set "CRANE_DEVICE=%BACKEND%"
    cargo test --no-default-features --features "%BACKEND% store"
)
if errorlevel 1 exit /b 1
pushd crates\crane_mbd
cargo test --release
popd
