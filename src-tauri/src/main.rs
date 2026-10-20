#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! 데스크톱 셸 (Tauri v1). 계산과 상태는 모두 `crane_model::backend::Backend` 가 갖고, 여기서는 명령을 넘겨 준다.
//! 브라우저용 `crane_demo serve` 와 같은 명령 이름 · 인자를 쓴다 (`api_call`).
//! 앱을 켜면 `ModelService` 가 없는 모델(TTM-R3 · LAYA)을 바로 받기 시작하고, 받은 모델은 백그라운드에서 불러온다.

use crane_model::api::{CompareRow, ModelStatus, PipelineConfig, PipelineReport};
use crane_model::backend::Backend;
use crane_model::calib::CrossCheck;
use crane_model::cyclesim::Scenario;
use crane_model::decision::DecisionReport;
use crane_model::model_service::ModelService;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

struct AppState {
    backend: Mutex<Backend>,
    svc: Arc<ModelService>,
}

fn call(state: &State<'_, AppState>, cmd: &str, args: Value) -> Result<Value, String> {
    // 모델 · 설정 명령은 긴 계산이 Backend 를 잡고 있어도 바로 답한다 (내려받기 진행률)
    if let Some(r) = state.svc.call(cmd, &args) {
        return r;
    }
    let mut b = state.backend.lock().map_err(|e| e.to_string())?;
    b.call(cmd, &args)
}

fn typed<T: for<'de> serde::Deserialize<'de>>(v: Value) -> Result<T, String> {
    serde_json::from_value(v).map_err(|e| e.to_string())
}

/// 모든 명령의 공통 입구 — 현장 프리셋 · 현장 운영 · 내구도 원장 · 실데이터 보정 · 모델 · 설정
///
/// 인자 이름을 `cmd` 로 두면 안 된다: Tauri v1 의 `invoke(cmd, args)` 는 `{cmd, callback, error, ...args}` 로 펼쳐 보내므로
/// `args.cmd` 가 명령 이름을 덮어써 "command <이름> not found" 가 된다 (화면은 `{name, args}` 로 부른다).
#[tauri::command]
async fn api_call(name: String, args: Value, state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, &name, args)
}

// ---- 모델 · 설정 — endmill-sandbox 와 같은 이름의 개별 명령 (인자는 화면과 crane_demo serve 처럼 snake_case) ----

#[tauri::command]
async fn models_catalog(state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "models_catalog", json!({}))
}

#[tauri::command(rename_all = "snake_case")]
async fn models_download(key: String, state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "models_download", json!({"key": key}))
}

#[tauri::command(rename_all = "snake_case")]
async fn models_cancel(key: String, state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "models_cancel", json!({"key": key}))
}

#[tauri::command(rename_all = "snake_case")]
async fn models_unload(key: String, state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "models_unload", json!({"key": key}))
}

#[tauri::command(rename_all = "snake_case")]
async fn models_delete(key: String, state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "models_delete", json!({"key": key}))
}

/// 작업이 모델을 쓰기 전: 있으면 불러오고, 없으면 (자동 받기면) 받기 시작한다
#[tauri::command(rename_all = "snake_case")]
async fn models_ensure(key: String, download: Option<bool>, state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "models_ensure", json!({"key": key, "download": download}))
}

/// 없는 모델 모두 받기
#[tauri::command]
async fn models_prepare(state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "models_prepare", json!({}))
}

#[tauri::command(rename_all = "snake_case")]
async fn models_set_root(root: String, state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "models_set_root", json!({"root": root}))
}

#[tauri::command]
async fn settings_get(state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "settings_get", json!({}))
}

#[tauri::command(rename_all = "snake_case")]
async fn settings_set_models(
    auto_download: Option<bool>,
    startup_download: Option<bool>,
    auto_load: Option<bool>,
    endpoint: Option<String>,
    token: Option<String>,
    clear_token: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    call(
        &state,
        "settings_set_models",
        json!({"auto_download": auto_download, "startup_download": startup_download, "auto_load": auto_load, "endpoint": endpoint, "token": token, "clear_token": clear_token}),
    )
}

#[tauri::command]
fn presets(state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "presets", json!({}))
}

#[tauri::command]
fn demo_scenario(crane: String, load: f64, mode: String, state: State<'_, AppState>) -> Result<Scenario, String> {
    typed(call(&state, "demo_scenario", json!({"crane": crane, "load": load, "mode": mode}))?)
}

#[tauri::command]
async fn run_pipeline(scenario: Scenario, config: PipelineConfig, state: State<'_, AppState>) -> Result<PipelineReport, String> {
    typed(call(&state, "run_pipeline", json!({"scenario": scenario, "config": config}))?)
}

#[tauri::command]
async fn compare(scenario: Scenario, state: State<'_, AppState>) -> Result<Vec<CompareRow>, String> {
    typed(call(&state, "compare", json!({"scenario": scenario}))?)
}

#[tauri::command]
async fn crosscheck(crane: String, rope: f64, load: f64, sling: f64, state: State<'_, AppState>) -> Result<CrossCheck, String> {
    typed(call(&state, "crosscheck", json!({"crane": crane, "rope": rope, "load": load, "sling": sling}))?)
}

/// `key` 하나를 불러오거나, 예전 화면처럼 `root` 를 주면 그 폴더로 바꾸고 모두 불러온다
#[tauri::command(rename_all = "snake_case")]
async fn models_load(key: Option<String>, root: Option<String>, state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, "models_load", json!({"key": key, "root": root}))
}

#[tauri::command]
fn models_status(state: State<'_, AppState>) -> Result<ModelStatus, String> {
    typed(call(&state, "models_status", json!({}))?)
}

/// 작업자 선택을 마지막 판정에 반영하고, 작업자–규칙 · 작업자–LAYA 일치를 SDS 원장에 남긴다
#[tauri::command]
fn choose_action(action: String, state: State<'_, AppState>) -> Result<DecisionReport, String> {
    typed(call(&state, "choose_action", json!({"action": action}))?)
}

#[tauri::command]
fn save_text(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("{path}: {e}"))
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path_resolver().app_data_dir().unwrap_or_else(|| std::env::temp_dir().join("gantry-crane-sandbox"));
            let _ = std::fs::create_dir_all(&dir);
            let backend = Backend::new(&dir);
            let svc = backend.svc.clone();
            svc.start(true);
            app.manage(AppState { backend: Mutex::new(backend), svc });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api_call,
            presets,
            demo_scenario,
            run_pipeline,
            compare,
            crosscheck,
            models_load,
            models_status,
            models_catalog,
            models_download,
            models_cancel,
            models_unload,
            models_delete,
            models_ensure,
            models_prepare,
            models_set_root,
            settings_get,
            settings_set_models,
            choose_action,
            save_text
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = &event {
                // 종료 정리가 어디선가 멈춰도 30초 뒤에는 프로세스를 끝낸다
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_secs(30));
                    eprintln!("[exit] 종료 정리가 30초 안에 끝나지 않아 프로세스를 끝냅니다");
                    hard_exit(1);
                });
                if let Some(state) = app_handle.try_state::<AppState>() {
                    // 내려받기를 멈춘다 (받은 부분은 이어받기용으로 남는다). 원장 · SDS 는 명령마다 이미 저장했다.
                    state.svc.shutdown();
                }
                // GPU 빌드에서 런타임 · 드라이버 정리 단계에 창 없는 프로세스가 남아 exe 를 잠그는 일이 있어 (endmill-sandbox 와 같은 처리)
                // 여기서 바로 끝낸다.
                hard_exit(0);
            }
        });
}

/// 프로세스를 즉시 끝낸다. Windows 에서는 ExitProcess 의 DLL 정리 단계를 거치지 않는다.
fn hard_exit(code: i32) -> ! {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn TerminateProcess(process: *mut std::ffi::c_void, exit_code: u32) -> i32;
        }
        unsafe {
            TerminateProcess(GetCurrentProcess(), code as u32);
        }
    }
    std::process::exit(code)
}
