#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! 데스크톱 셸 (Tauri v1). 계산과 상태는 모두 `crane_model::backend::Backend` 가 갖고, 여기서는 명령을 넘겨 준다.
//! 브라우저용 `crane_demo serve` 와 같은 명령 이름 · 인자를 쓴다 (`api_call`).

use crane_model::api::{CompareRow, ModelStatus, PipelineConfig, PipelineReport};
use crane_model::backend::Backend;
use crane_model::calib::CrossCheck;
use crane_model::cyclesim::Scenario;
use crane_model::decision::DecisionReport;
use serde_json::{json, Value};
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState {
    backend: Mutex<Backend>,
}

fn call(state: &State<'_, AppState>, cmd: &str, args: Value) -> Result<Value, String> {
    let mut b = state.backend.lock().map_err(|e| e.to_string())?;
    b.call(cmd, &args)
}

fn typed<T: for<'de> serde::Deserialize<'de>>(v: Value) -> Result<T, String> {
    serde_json::from_value(v).map_err(|e| e.to_string())
}

/// 모든 명령의 공통 입구 — 현장 프리셋 · 현장 운영 · 내구도 원장 · 실데이터 보정
#[tauri::command]
async fn api_call(cmd: String, args: Value, state: State<'_, AppState>) -> Result<Value, String> {
    call(&state, &cmd, args)
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

#[tauri::command]
async fn models_load(root: String, state: State<'_, AppState>) -> Result<ModelStatus, String> {
    typed(call(&state, "models_load", json!({"root": root}))?)
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
            app.manage(AppState { backend: Mutex::new(Backend::new(&dir)) });
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
            choose_action,
            save_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
