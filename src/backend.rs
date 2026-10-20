//! 화면(데스크톱 · 브라우저)이 부르는 명령을 한곳에서 처리한다.
//!
//! Tauri 명령과 `crane_demo serve` 의 `/api/<명령>` 이 같은 `Backend::call` 을 쓴다. 상태(불러온 모델, 마지막 파이프라인 결과,
//! 진행 중인 현장 운영 시뮬레이션)는 여기에 있고, 현장 프리셋 · 내구도 원장 · SDS 원장은 `data_dir` 아래에 저장한다.

use crate::antisway::AntiSwayMode;
use crate::api::{self, Models, PipelineConfig, PipelineReport};
use crate::calib;
use crate::cargo;
use crate::climate_data;
use crate::crane_profile;
use crate::cyclesim::{self, Scenario};
use crate::decision::{self, Action};
use crate::era5cal;
use crate::fieldcal;
use crate::health::{self, HealthStore};
use crate::marine;
use crate::operation::{self, DecisionPolicy, OpSim, OperationConfig, StepOutcome, WeatherSource};
use crate::prognostics;
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

    /// ERA5 시간별 캐시 — 저장소에서 띄웠으면 `out/weather_cache` (weather build 와 같은 곳), 아니면 `data_dir/weather_cache`
    fn weather_cache(&self) -> PathBuf {
        let repo = Path::new(operation::DEFAULT_WEATHER_CACHE);
        if repo.is_dir() {
            repo.to_path_buf()
        } else {
            self.data_dir.join("weather_cache")
        }
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
                // 날씨 출처는 객체 {era5, from, offline} 나 CLI 와 같은 글자 "era5:<위치>[@YYYY-MM-DD]" 로 줄 수 있다
                let mut cv = args.get("config").cloned().unwrap_or(Value::Null);
                if let Some(Value::String(w)) = cv.get("weather").cloned() {
                    cv["weather"] = to_value(&WeatherSource::parse(&w)?)?;
                }
                let mut cfg: OperationConfig = serde_json::from_value(cv).map_err(|e| format!("인자 'config': {e}"))?;
                if let Some(p) = opt_arg::<String>(args, "policy")? {
                    cfg.policy = DecisionPolicy::from_key(&p).ok_or_else(|| format!("모르는 정책: {p}"))?;
                }
                if let Some(w) = cfg.weather.as_mut() {
                    if w.cache_dir.is_none() {
                        w.cache_dir = Some(self.weather_cache().to_string_lossy().into_owned());
                    }
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

            // ---- 염해 · 화물 · 예측 점수 ----
            "marine_refs" => Ok(json!({
                "protections": to_value(&marine::protection_presets())?,
                "salt_areas": marine::KR_SALT_100M.iter().map(|(a, r, v)| json!({"id": a, "region": r, "mdd_100m": v})).collect::<Vec<_>>(),
                "coast_types": ["harbor", "bay", "open", "tidal", "strait", "river"],
                "ports": to_value(&crate::ports::sites())?,
            })),
            "cargo_scenarios" => Ok(json!({"scenarios": to_value(&cargo::scenarios())?, "types": to_value(&cargo::types())?})),
            "marine_project" => {
                let preset: SitePreset = match opt_arg::<SitePreset>(args, "preset")? {
                    Some(p) => p,
                    None => self.sites().get(&arg::<String>(args, "site")?)?,
                };
                let years: u32 = opt_arg::<u32>(args, "years")?.unwrap_or(10).clamp(1, 40);
                let seed: u64 = opt_arg(args, "seed")?.unwrap_or(1);
                let maintain: bool = opt_arg(args, "maintain")?.unwrap_or(true);
                // 현장 프리셋의 방지 조치를 먼저, 비교용으로 나머지 프리셋을 뒤에
                let mine = preset.environment.protection()?;
                let mut prots = vec![mine.clone()];
                if opt_arg::<bool>(args, "compare")?.unwrap_or(true) {
                    prots.extend(marine::protection_presets().into_iter().filter(|p| p.id != mine.id));
                }
                let out: Vec<marine::Projection> = prots.iter().map(|p| marine::project(&preset, p, years, seed, maintain)).collect::<Result<_, _>>()?;
                to_value(&out)
            }
            "prognostics_saved" => to_value(&prognostics::saved_report().ok_or("저장된 예측 점수 보고서를 읽지 못했습니다")?),
            "prognostics_run" => {
                let ids: Vec<String> = opt_arg(args, "sites")?.unwrap_or_else(|| ["busan_newport_rmg", "gwangyang_coil_warehouse", "ulsan_shipyard_goliath"].iter().map(|s| s.to_string()).collect());
                let sites: Vec<SitePreset> = ids.iter().map(|i| self.sites().get(i)).collect::<Result<_, _>>()?;
                let days: u32 = opt_arg::<u32>(args, "days")?.unwrap_or(1095).clamp(400, 3650);
                let replicas: u32 = opt_arg::<u32>(args, "replicas")?.unwrap_or(4).clamp(1, 50);
                let every: u32 = opt_arg::<u32>(args, "every")?.unwrap_or(30).clamp(7, 365);
                let seed: u64 = opt_arg(args, "seed")?.unwrap_or(2026);
                to_value(&prognostics::run(&sites, days, replicas, every, seed)?)
            }
            "weather_check" => to_value(&era5cal::run()),
            // 운영 재생에 쓸 수 있는 ERA5 위치 · 시간별 기간 · 캐시에 있는가
            "weather_sources" => {
                let mut f = crate::weather_api::Fetcher::new(&self.weather_cache());
                f.offline = true;
                let list: Vec<Value> = crate::ports::sites()
                    .iter()
                    .map(|s| {
                        let (h0, h1) = crate::ports::hourly_period(s);
                        let cached = f.cache_path("hourly-era5", s.lat, s.lon, h0, h1).is_file();
                        json!({"id": s.id, "name": s.name, "kind": s.kind, "country": s.country, "from": h0, "to": h1, "cached": cached})
                    })
                    .collect();
                Ok(json!({"cache_dir": f.cache_dir.to_string_lossy(), "sites": list}))
            }
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

    #[test]
    fn op_start_takes_era5_weather_source() {
        let dir = std::env::temp_dir().join(format!("crane_backend_w_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut b = Backend::new(&dir);
        let ws = b.call("weather_sources", &json!({})).unwrap();
        assert_eq!(ws["sites"].as_array().unwrap().len(), crate::ports::sites().len());
        // 글자 꼴 · 객체 꼴 모두 받고, 빈 캐시 + 오프라인이면 분명한 오류
        let cache = dir.join("empty_cache").to_string_lossy().into_owned();
        for w in [json!({"era5": "kma_busan", "from": "2022-07-01", "offline": true, "cache_dir": cache}), json!("era5:nowhere@2023-01-01")] {
            let e = b.call("op_start", &json!({"site": "busan_newport_rmg", "config": {"asset_id": "W", "days": 2, "fresh": true, "weather": w}})).unwrap_err();
            assert!(e.contains("ERA5"), "{e}");
        }
        assert!(b.call("op_start", &json!({"site": "busan_newport_rmg", "config": {"asset_id": "W", "days": 2, "weather": "kma:busan"}})).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn marine_cargo_and_scores_commands_answer() {
        let dir = std::env::temp_dir().join(format!("crane_backend_m_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut b = Backend::new(&dir);
        let refs = b.call("marine_refs", &json!({})).unwrap();
        assert!(refs["protections"].as_array().unwrap().len() >= 4);
        assert_eq!(refs["salt_areas"].as_array().unwrap().len(), 27);
        let cargo = b.call("cargo_scenarios", &json!({})).unwrap();
        assert!(cargo["scenarios"].as_array().unwrap().len() >= 6);
        let proj = b.call("marine_project", &json!({"site": "busan_newport_rmg", "years": 2})).unwrap();
        let rows = proj.as_array().unwrap();
        assert_eq!(rows.len(), refs["protections"].as_array().unwrap().len());
        assert_eq!(rows[0]["rows"].as_array().unwrap().len(), 2);
        let saved = b.call("prognostics_saved", &json!({})).unwrap();
        let overall = saved["overall"].as_array().unwrap();
        assert!(overall.len() >= 4);
        assert!(overall.iter().any(|m| m["method"] == "v3"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
