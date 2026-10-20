//! 화면(데스크톱 · 브라우저)이 부르는 명령을 한곳에서 처리한다.
//!
//! Tauri 명령과 `crane_demo serve` 의 `/api/<명령>` 이 같은 `Backend::call` 을 쓴다. 상태(불러온 모델, 마지막 파이프라인 결과,
//! 진행 중인 현장 운영 시뮬레이션)는 여기에 있고, 현장 프리셋 · 내구도 원장 · SDS 원장은 `data_dir` 아래에 저장한다.

use crate::antisway::AntiSwayMode;
use crate::api::{self, Models, PipelineConfig, PipelineReport};
use crate::calib;
use crate::climate_data;
use crate::crane_profile;
use crate::cyclesim::{self, Scenario};
use crate::decision::{self, Action};
use crate::fieldcal;
use crate::health::{self, HealthStore};
use crate::operation::{DecisionPolicy, OpSim, OperationConfig, StepOutcome};
use crate::sds::{Scope, SdsStore};
use crate::site::{self, SitePreset, SiteStore};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub struct Backend {
    pub models: Models,
    pub last: Option<PipelineReport>,
    pub op: Option<OpSim>,
    pub data_dir: PathBuf,
}

fn mode_of(s: &str) -> Result<AntiSwayMode, String> {
    AntiSwayMode::all().into_iter().find(|m| m.key() == s).ok_or_else(|| format!("모르는 흔들림 억제 방식: {s}"))
}

fn arg<T: for<'de> Deserialize<'de>>(args: &Value, key: &str) -> Result<T, String> {
    let v = args.get(key).cloned().unwrap_or(Value::Null);
    serde_json::from_value(v).map_err(|e| format!("인자 '{key}': {e}"))
}

fn opt_arg<T: for<'de> Deserialize<'de>>(args: &Value, key: &str) -> Result<Option<T>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => serde_json::from_value(v.clone()).map(Some).map_err(|e| format!("인자 '{key}': {e}")),
    }
}

fn to_value<T: serde::Serialize>(v: &T) -> Result<Value, String> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

impl Backend {
    pub fn new(data_dir: &Path) -> Self {
        let _ = std::fs::create_dir_all(data_dir);
        Self { models: Models::none(), last: None, op: None, data_dir: data_dir.to_path_buf() }
    }

    pub fn sites(&self) -> SiteStore {
        SiteStore::new(&self.data_dir.join("presets"))
    }

    pub fn health_store(&self) -> HealthStore {
        HealthStore::new(&self.data_dir.join("health"))
    }

    fn sds_dir(&self) -> PathBuf {
        self.data_dir.join("sds")
    }

    /// 진행 중인 운영 시뮬레이션의 상태 (진행 · 기다리는 결정 · 보고서)
    fn op_state(&self, outcome: Option<&StepOutcome>) -> Result<Value, String> {
        let sim = self.op.as_ref().ok_or("진행 중인 현장 운영이 없습니다 (op_start 먼저)")?;
        Ok(json!({
            "outcome": outcome,
            "pending": sim.pending(),
            "hour_index": sim.hour_index(),
            "finished": sim.finished(),
            "report": sim.report(),
        }))
    }

    fn save_ledger(&self) -> Result<Option<String>, String> {
        match &self.op {
            Some(sim) => Ok(Some(self.health_store().save(sim.ledger())?.to_string_lossy().into_owned())),
            None => Ok(None),
        }
    }

    pub fn call(&mut self, cmd: &str, args: &Value) -> Result<Value, String> {
        match cmd {
            "ping" => Ok(json!({"ok": true, "ml_feature": cfg!(feature = "ml")})),
            "presets" => Ok(api::presets_json()),
            "demo_scenario" => {
                let crane: String = arg(args, "crane")?;
                let load: f64 = arg(args, "load")?;
                let mode: String = arg(args, "mode")?;
                to_value(&cyclesim::demo_scenario(&crane, load, mode_of(&mode)?))
            }
            "run_pipeline" => {
                let scenario: Scenario = arg(args, "scenario")?;
                let mut cfg: PipelineConfig = arg(args, "config")?;
                if cfg.sds_dir.is_none() {
                    cfg.sds_dir = Some(self.sds_dir().to_string_lossy().into_owned());
                }
                let rep = api::run_pipeline(&scenario, &cfg, &self.models)?;
                let v = to_value(&rep)?;
                self.last = Some(rep);
                Ok(v)
            }
            "compare" => {
                let scenario: Scenario = arg(args, "scenario")?;
                to_value(&api::compare(&scenario, &AntiSwayMode::all())?)
            }
            "crosscheck" => {
                let crane: String = arg(args, "crane")?;
                let spec = crane_profile::preset(&crane).ok_or_else(|| format!("크레인 프리셋 {crane} 이 없습니다"))?;
                to_value(&calib::crosscheck(&spec, arg(args, "rope")?, arg(args, "load")?, arg(args, "sling")?))
            }
            "models_load" => {
                let root: String = arg(args, "root")?;
                self.models = Models::load(Path::new(&root));
                to_value(&self.models.status())
            }
            "models_status" => to_value(&self.models.status()),
            "choose_action" => {
                let action: String = arg(args, "action")?;
                let a = Action::from_key(&action).ok_or_else(|| format!("모르는 조치: {action}"))?;
                let sds_dir = self.sds_dir();
                let rep = self.last.as_mut().ok_or("먼저 실행하세요")?;
                decision::apply_operator_choice(&mut rep.decision, a)?;
                let mut sds = SdsStore::open(&sds_dir)?;
                let scope = Scope::new(&rep.crane.name, "all");
                sds.record_agreement(&scope, "operator_rule", rep.decision.rule_action == a);
                if let Some(adv) = rep.decision.advisor_action {
                    sds.record_agreement(&scope, "operator_laya", adv == a);
                }
                sds.flush()?;
                let op = sds.agreement_rate(&scope, "operator_laya");
                let rl = sds.agreement_rate(&scope, "rule_laya");
                rep.decision.choice = Some(decision::choice_prompt(&rep.decision, op, rl));
                to_value(&rep.decision)
            }

            // ---- 현장 프리셋 ----
            "sites_list" => to_value(&self.sites().list()),
            "site_get" => {
                let id: String = arg(args, "id")?;
                to_value(&self.sites().get(&id)?)
            }
            "site_save" => {
                let preset: SitePreset = arg(args, "preset")?;
                let p = self.sites().save(&preset)?;
                Ok(json!({"path": p.to_string_lossy(), "id": preset.id}))
            }
            "site_delete" => {
                let id: String = arg(args, "id")?;
                self.sites().delete(&id)?;
                Ok(json!({"deleted": id}))
            }
            "site_scenario" => {
                let id: String = arg(args, "id")?;
                let p = self.sites().get(&id)?;
                let spec = p.spec()?;
                let frac: f64 = opt_arg(args, "fraction")?.unwrap_or(0.6);
                to_value(&p.scenario(frac * spec.structure.rated_load_kg, 1.0, crate::environment::Environment::default())?)
            }
            "climates" => to_value(&climate_data::presets()),
            "cranes" => to_value(&crane_profile::presets()),

            // ---- 현장 운영 ----
            "op_start" => {
                let preset: SitePreset = match opt_arg::<SitePreset>(args, "preset")? {
                    Some(p) => p,
                    None => self.sites().get(&arg::<String>(args, "site")?)?,
                };
                let mut cfg: OperationConfig = arg(args, "config")?;
                if let Some(p) = opt_arg::<String>(args, "policy")? {
                    cfg.policy = DecisionPolicy::from_key(&p).ok_or_else(|| format!("모르는 정책: {p}"))?;
                }
                let ledger = if cfg.fresh { None } else { self.health_store().load(&cfg.asset_id)? };
                self.op = Some(OpSim::new(preset, cfg, ledger)?);
                self.op_state(None)
            }
            "op_run" => {
                let hours: usize = opt_arg(args, "hours")?.unwrap_or(usize::MAX / 4);
                let sim = self.op.as_mut().ok_or("진행 중인 현장 운영이 없습니다 (op_start 먼저)")?;
                let out = sim.run(hours);
                self.save_ledger()?;
                self.op_state(Some(&out))
            }
            "op_decide" => {
                let id: String = arg(args, "id")?;
                let key: String = arg(args, "key")?;
                let sim = self.op.as_mut().ok_or("진행 중인 현장 운영이 없습니다")?;
                sim.decide(&id, &key)?;
                let hours: usize = opt_arg(args, "then_hours")?.unwrap_or(usize::MAX / 4);
                let out = sim.run(hours);
                self.save_ledger()?;
                self.op_state(Some(&out))
            }
            "op_report" => self.op_state(None),

            // ---- 내구도 원장 ----
            "health_list" => to_value(&self.health_store().list()),
            "health_get" => {
                let asset: String = arg(args, "asset")?;
                let l = self.health_store().load(&asset)?.ok_or_else(|| format!("자산 '{asset}' 원장이 없습니다"))?;
                Ok(json!({"ledger": l, "durability": health::evaluate(&l)}))
            }
            "health_replace" => {
                let asset: String = arg(args, "asset")?;
                let comp: String = arg(args, "component")?;
                let st = self.health_store();
                let mut l = st.load(&asset)?.ok_or_else(|| format!("자산 '{asset}' 원장이 없습니다"))?;
                let spec = crane_profile::preset(&l.crane).or_else(|| self.sites().get(&l.site).ok().and_then(|s| s.spec().ok())).ok_or("원장의 크레인 제원을 찾지 못했습니다")?;
                let day = l.day;
                l.replace(&comp, &spec, day, 0, "수동 교체")?;
                st.save(&l)?;
                Ok(json!({"ledger": l, "durability": health::evaluate(&l)}))
            }
            "health_delete" => {
                let asset: String = arg(args, "asset")?;
                self.health_store().delete(&asset)?;
                Ok(json!({"deleted": asset}))
            }

            // ---- 실데이터 보정 ----
            "fieldcal" => {
                let quick: bool = opt_arg(args, "quick")?.unwrap_or(true);
                let o = if quick { fieldcal::CalOptions::quick() } else { fieldcal::CalOptions::default() };
                to_value(&fieldcal::run(&o))
            }
            "fieldcal_saved" => to_value(&fieldcal::saved_report().ok_or("저장된 보정 보고서를 읽지 못했습니다")?),
            "site_defaults" => to_value(&site::builtin()),
            other => Err(format!("모르는 명령: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_flow_through_backend_persists_ledger() {
        let dir = std::env::temp_dir().join(format!("crane_backend_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut b = Backend::new(&dir);
        let sites = b.call("sites_list", &json!({})).unwrap();
        let id = sites[0]["id"].as_str().unwrap().to_string();
        let st = b.call("op_start", &json!({"site": id, "config": {"asset_id": "B-1", "days": 3, "seed": 2, "policy": "ask"}})).unwrap();
        assert_eq!(st["finished"], false);
        let mut v = b.call("op_run", &json!({})).unwrap();
        for _ in 0..200 {
            if v["finished"].as_bool().unwrap_or(false) {
                break;
            }
            let p = &v["pending"];
            assert!(!p.is_null(), "paused without a decision");
            let rec = p["prompt"]["recommended"].as_str().unwrap().to_string();
            v = b.call("op_decide", &json!({"id": p["id"], "key": rec})).unwrap();
        }
        assert_eq!(v["finished"], true);
        let h = b.call("health_get", &json!({"asset": "B-1"})).unwrap();
        assert_eq!(h["ledger"]["day"].as_f64().unwrap(), 3.0);
        assert!(b.call("nope", &json!({})).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
