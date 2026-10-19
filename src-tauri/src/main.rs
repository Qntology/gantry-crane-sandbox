#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! 데스크톱 셸 (Tauri v1). 계산은 모두 `crane_model::api` 가 하고, 여기서는 상태(모델 · 마지막 결과)만 들고 있다.

use crane_model::antisway::AntiSwayMode;
use crane_model::api::{self, CompareRow, ModelStatus, Models, PipelineConfig, PipelineReport};
use crane_model::calib::{self, CrossCheck};
use crane_model::crane_profile;
use crane_model::cyclesim::{self, Scenario};
use crane_model::decision::{self, Action, DecisionReport};
use crane_model::sds::{Scope, SdsStore};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState {
    models: Mutex<Models>,
    last: Mutex<Option<PipelineReport>>,
    data_dir: PathBuf,
}

fn mode_of(s: &str) -> Result<AntiSwayMode, String> {
    AntiSwayMode::all().into_iter().find(|m| m.key() == s).ok_or_else(|| format!("모르는 흔들림 억제 방식: {s}"))
}

#[tauri::command]
fn presets() -> serde_json::Value {
    api::presets_json()
}

#[tauri::command]
fn demo_scenario(crane: String, load: f64, mode: String) -> Result<Scenario, String> {
    Ok(cyclesim::demo_scenario(&crane, load, mode_of(&mode)?))
}

#[tauri::command]
async fn run_pipeline(scenario: Scenario, config: PipelineConfig, state: State<'_, AppState>) -> Result<PipelineReport, String> {
    let mut cfg = config;
    if cfg.sds_dir.is_none() {
        cfg.sds_dir = Some(state.data_dir.join("sds").to_string_lossy().into_owned());
    }
    let rep = {
        let models = state.models.lock().map_err(|e| e.to_string())?;
        api::run_pipeline(&scenario, &cfg, &models)?
    };
    *state.last.lock().map_err(|e| e.to_string())? = Some(rep.clone());
    Ok(rep)
}

#[tauri::command]
async fn compare(scenario: Scenario) -> Result<Vec<CompareRow>, String> {
    api::compare(&scenario, &AntiSwayMode::all())
}

#[tauri::command]
async fn crosscheck(crane: String, rope: f64, load: f64, sling: f64) -> Result<CrossCheck, String> {
    let spec = crane_profile::preset(&crane).ok_or_else(|| format!("크레인 프리셋 {crane} 이 없습니다"))?;
    Ok(calib::crosscheck(&spec, rope, load, sling))
}

#[tauri::command]
async fn models_load(root: String, state: State<'_, AppState>) -> Result<ModelStatus, String> {
    let m = Models::load(std::path::Path::new(&root));
    let st = m.status();
    *state.models.lock().map_err(|e| e.to_string())? = m;
    Ok(st)
}

#[tauri::command]
fn models_status(state: State<'_, AppState>) -> Result<ModelStatus, String> {
    Ok(state.models.lock().map_err(|e| e.to_string())?.status())
}

/// 작업자 선택을 마지막 판정에 반영하고, 작업자–규칙 · 작업자–LAYA 일치를 SDS 원장에 남긴다
#[tauri::command]
fn choose_action(action: String, state: State<'_, AppState>) -> Result<DecisionReport, String> {
    let a = Action::from_key(&action).ok_or_else(|| format!("모르는 조치: {action}"))?;
    let mut last = state.last.lock().map_err(|e| e.to_string())?;
    let rep = last.as_mut().ok_or("먼저 실행하세요")?;
    decision::apply_operator_choice(&mut rep.decision, a)?;
    let mut sds = SdsStore::open(&state.data_dir.join("sds"))?;
    let scope = Scope::new(&rep.crane.name, "all");
    sds.record_agreement(&scope, "operator_rule", rep.decision.rule_action == a);
    if let Some(adv) = rep.decision.advisor_action {
        sds.record_agreement(&scope, "operator_laya", adv == a);
    }
    sds.flush()?;
    let op = sds.agreement_rate(&scope, "operator_laya");
    let rl = sds.agreement_rate(&scope, "rule_laya");
    rep.decision.choice = Some(decision::choice_prompt(&rep.decision, op, rl));
    Ok(rep.decision.clone())
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
            app.manage(AppState { models: Mutex::new(Models::none()), last: Mutex::new(None), data_dir: dir });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
