//! CLI · 데스크톱 셸이 함께 쓰는 상위 API.
//!
//! `run_pipeline`: 사이클 시뮬레이션 → 듀티(근무) → 시계열 예측(물리 사전값 + 잔차, TTM-R3 또는 Holt) →
//! SDS 원장 · 드리프트 → 결정 게이트 + LAYA 조언 → 작업자 선택 카드 → 정밀 해석기 교차검증.
//! 엔드밀 `pipeline.rs` 의 '1차 시뮬레이션 → 시계열 예측 → 규칙·LAYA 판정 → 2차 보정' 흐름과 같다.

use crate::antisway::AntiSwayMode;
use crate::calib::{self, CrossCheck};
use crate::crane_profile::{self, Axis, CraneSpec};
use crate::cyclesim::{self, CycleReport, Scenario};
use crate::decision::{self, Advisor, DecisionInput, DecisionReport};
use crate::duty::{self, DutyConfig, DutyReport};
use crate::sds::{Scope, SdsStore};
use crate::timeseries::{self, ForecastResult, SeriesModel};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 불러온 학습 모델
#[derive(Default)]
pub struct Models {
    pub ttm: Option<Box<dyn SeriesModel + Send + Sync>>,
    pub advisor: Option<Box<dyn Advisor + Send + Sync>>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub ttm: bool,
    pub advisor: bool,
    pub ml_feature: bool,
    pub notes: Vec<String>,
}

/// 화면 · 설정이 쓰는 모델 키 (modelhub 목록과 같다)
pub const MODEL_KEYS: [&str; 2] = ["ttm", "laya"];

/// 잠금 밖에서 따로 불러온 모델 하나 — `Models::insert` 로 넣는다
pub enum LoadedModel {
    Ttm(Box<dyn SeriesModel + Send + Sync>),
    Laya(Box<dyn Advisor + Send + Sync>),
}

/// 모델 폴더 표지 — TTM 은 `config.json`, LAYA 는 Hugging Face 배치 그대로 `encoder/config.json`
fn has_marker(dir: &Path, keyword: &str) -> bool {
    match keyword {
        "laya" => dir.join("encoder").join("config.json").exists(),
        _ => dir.join("config.json").exists(),
    }
}

/// root 아래(root 자신 포함, 깊이 4)에서 경로에 `keyword` 가 들어가고 표지 파일과 safetensors 를 가진 폴더를 찾는다.
/// 예: `granite-timeseries-ttm-r3/52-16-dec-52-r3/` 는 "ttm", `laya-typed-decisions/` 는 "laya" 로 찾는다.
/// TTM 이 여러 판이면 문맥 길이가 짧은 판을 앞에 둔다 (근무 이력이 짧아도 쓸 수 있게).
pub fn find_model_dirs(root: &Path, keyword: &str) -> Vec<PathBuf> {
    fn walk(base: &Path, dir: &Path, kw: &str, depth: usize, out: &mut Vec<PathBuf>) {
        if depth > 4 {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        entries.sort();
        let rel = dir.strip_prefix(base).unwrap_or(dir).to_string_lossy().to_ascii_lowercase();
        let has_st = entries.iter().any(|p| p.extension().map(|x| x == "safetensors").unwrap_or(false));
        if rel.contains(kw) && has_st && has_marker(dir, kw) {
            out.push(dir.to_path_buf());
        }
        for e in entries.iter().filter(|p| p.is_dir()) {
            walk(base, e, kw, depth + 1, out);
        }
    }
    let kw = keyword.to_ascii_lowercase();
    let mut out = Vec::new();
    // root 의 이름도 경로에 넣어 비교한다 (root 가 곧 모델 폴더일 때)
    walk(root.parent().unwrap_or(root), root, &kw, 0, &mut out);
    if kw == "ttm" {
        let ctx = |d: &PathBuf| {
            std::fs::read_to_string(d.join("config.json"))
                .ok()
                .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                .and_then(|v| v.get("context_length").and_then(|x| x.as_u64()))
                .unwrap_or(u64::MAX)
        };
        out.sort_by(|a, b| ctx(a).cmp(&ctx(b)).then(a.cmp(b)));
    }
    out
}

pub fn find_model_dir(root: &Path, keyword: &str) -> Option<PathBuf> {
    find_model_dirs(root, keyword).into_iter().next()
}

/// 모델 하나를 불러온다. 돌려주는 글은 화면에 보일 요약이다.
#[cfg(feature = "ml")]
pub fn load_model(key: &str, dir: &Path) -> Result<(LoadedModel, String), String> {
    let device = crate::ml::device();
    let dev = crate::ml::device_label(&device);
    match key {
        "ttm" => {
            let t = crate::ml::ttm::TtmForecaster::load(dir, &device)?;
            let detail = format!(
                "context {} → 예측 {} · 분위 {}개 · {} · {dev}",
                t.context_length(),
                t.prediction_length(),
                t.config.quantile_levels.len(),
                if t.config.decompose { "추세/잔차 분해형" } else { "단일형" }
            );
            Ok((LoadedModel::Ttm(Box::new(t)), detail))
        }
        "laya" => {
            let a = crate::ml::laya::LayaAdvisor::load(dir, &device)?;
            Ok((LoadedModel::Laya(Box::new(a)), format!("ModernBERT-large 인코더 · 판정 조언 (상향만) · {dev}")))
        }
        other => Err(format!("알 수 없는 모델: {other}")),
    }
}

#[cfg(not(feature = "ml"))]
pub fn load_model(_key: &str, _dir: &Path) -> Result<(LoadedModel, String), String> {
    Err("ml 기능 없이 빌드되어 모델을 불러오지 않습니다".into())
}

impl Models {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn is_loaded(&self, key: &str) -> bool {
        match key {
            "ttm" => self.ttm.is_some(),
            "laya" => self.advisor.is_some(),
            _ => false,
        }
    }

    pub fn insert(&mut self, m: LoadedModel) {
        match m {
            LoadedModel::Ttm(t) => self.ttm = Some(t),
            LoadedModel::Laya(a) => self.advisor = Some(a),
        }
    }

    pub fn unload(&mut self, key: &str) -> bool {
        let was = self.is_loaded(key);
        match key {
            "ttm" => self.ttm = None,
            "laya" => self.advisor = None,
            _ => {}
        }
        was
    }

    /// root 아래의 TTM · LAYA 를 모두 불러온다 (CLI `--models`)
    pub fn load(root: &Path) -> Self {
        let mut m = Models::default();
        if !cfg!(feature = "ml") {
            m.notes.push("ml 기능 없이 빌드되어 모델을 불러오지 않습니다".into());
            return m;
        }
        #[cfg(feature = "ml")]
        m.notes.push(format!("장치: {}", crate::ml::device_label(&crate::ml::device())));
        for (key, name, hint) in [("ttm", "TTM", "이름에 'ttm' 포함, config.json + safetensors"), ("laya", "LAYA", "이름에 'laya' 포함, encoder/config.json + safetensors")] {
            let dirs = find_model_dirs(root, key);
            if dirs.is_empty() {
                m.notes.push(format!("{name} 폴더를 찾지 못했습니다 ({hint})"));
                continue;
            }
            let mut last = String::new();
            for dir in dirs {
                match load_model(key, &dir) {
                    Ok((lm, _)) => {
                        m.notes.push(format!("{name}: {}", dir.display()));
                        m.insert(lm);
                        last.clear();
                        break;
                    }
                    Err(e) => last = format!("{name} 로드 실패: {e}"),
                }
            }
            if !last.is_empty() {
                m.notes.push(last);
            }
        }
        m
    }

    pub fn status(&self) -> ModelStatus {
        ModelStatus { ttm: self.ttm.is_some(), advisor: self.advisor.is_some(), ml_feature: cfg!(feature = "ml"), notes: self.notes.clone() }
    }
}

fn axis_of_key(key: &str) -> Option<Axis> {
    Axis::all().into_iter().find(|a| key.contains(a.key()))
}

/// 듀티 시계열을 예측한다. 권선 온도는 물리 사전값(평균 손실의 정상상태로 다가가는 지수 곡선) + 잔차.
pub fn forecast_duty(spec: &CraneSpec, d: &DutyReport, horizon_h: f64, model: Option<&dyn SeriesModel>) -> Vec<ForecastResult> {
    let lim = &spec.limits;
    let mut out = Vec::new();
    for s in d.series.iter() {
        if s.len() < 3 {
            continue;
        }
        let step = s.median_step().unwrap_or(1.0).max(1e-9);
        let horizon = ((horizon_h / step).ceil() as usize).clamp(1, 2000);
        let key = s.key.as_str();
        let r = if key.ends_with("_winding_peak_c") {
            let ax = axis_of_key(key).unwrap_or(Axis::Hoist);
            let prior = d.thermal_priors.iter().find(|p| p.axis == ax).cloned();
            let limit = spec.drive(ax).motor.insulation_limit_c() - lim.winding_margin_k;
            match prior {
                Some(p) => {
                    let f = move |t: f64| p.at(t);
                    timeseries::forecast(s, horizon, model, Some(&f), Some(limit))
                }
                None => timeseries::forecast(s, horizon, model, None, Some(limit)),
            }
        } else if key == "rope_damage" {
            timeseries::forecast(s, horizon, model, None, Some(lim.rope_damage_warn))
        } else if key.starts_with("brake_wear_") {
            let ax = axis_of_key(key).unwrap_or(Axis::Hoist);
            let b = &spec.drive(ax).brake;
            let usable = (b.lining_new_mm - b.lining_min_mm).max(1e-9);
            timeseries::forecast(s, horizon, model, None, Some(usable * (1.0 - lim.brake_lining_inspect)))
        } else {
            timeseries::forecast(s, horizon, model, None, None)
        };
        out.push(r);
    }
    out
}

fn default_true() -> bool {
    true
}
fn default_forecast_h() -> f64 {
    8.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// 근무 시뮬레이션 (없으면 사이클만)
    #[serde(default)]
    pub duty: Option<DutyConfig>,
    /// 예측 수평선 (h)
    #[serde(default = "default_forecast_h")]
    pub forecast_hours: f64,
    /// 측정 · 예보 풍속 (m/s)
    #[serde(default)]
    pub wind_mps: Option<f64>,
    /// SDS 원장 폴더
    #[serde(default)]
    pub sds_dir: Option<String>,
    /// LAYA 조언의 상향 허용
    #[serde(default = "default_true")]
    pub allow_advisor_escalation: bool,
    /// 정밀 해석기 교차검증
    #[serde(default = "default_true")]
    pub crosscheck: bool,
    /// 교차검증에 쓸 슬링 길이 (훅 → 하중 중심, m)
    #[serde(default)]
    pub sling_length_m: f64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self { duty: Some(DutyConfig::default()), forecast_hours: 8.0, wind_mps: None, sds_dir: None, allow_advisor_escalation: true, crosscheck: true, sling_length_m: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineReport {
    pub crane: CraneSpec,
    pub cycle: CycleReport,
    pub duty: Option<DutyReport>,
    pub forecasts: Vec<ForecastResult>,
    pub decision: DecisionReport,
    pub crosscheck: Option<CrossCheck>,
    pub models: ModelStatus,
    pub notes: Vec<String>,
}

pub fn run_pipeline(sc: &Scenario, cfg: &PipelineConfig, models: &Models) -> Result<PipelineReport, String> {
    let spec = sc.crane.resolve()?;
    let mut notes = Vec::new();
    let cycle = cyclesim::simulate(sc)?;
    let duty = match &cfg.duty {
        Some(dc) => Some(duty::run(sc, dc)?),
        None => None,
    };
    let model_ref: Option<&dyn SeriesModel> = models.ttm.as_deref().map(|m| m as &dyn SeriesModel);
    let forecasts = duty.as_ref().map(|d| forecast_duty(&spec, d, cfg.forecast_hours, model_ref)).unwrap_or_default();

    // 원장
    let mut sds = match &cfg.sds_dir {
        Some(dir) => Some(SdsStore::open(Path::new(dir))?),
        None => None,
    };
    let drift: Vec<(String, f64)> = match sds.as_mut() {
        Some(s) => {
            // 이번 실행을 먼저 넣어야 이번 이상이 이번 판정에 바로 잡힌다
            s.ingest_cycle(&spec.name, &cycle);
            let items = s.drift_report(&spec.name);
            items.into_iter().map(|d| (d.key, d.z)).collect()
        }
        None => Vec::new(),
    };

    let mut decision = decision::evaluate(&DecisionInput {
        spec: &spec,
        cycle: Some(&cycle),
        duty: duty.as_ref(),
        forecasts: &forecasts,
        wind_mps: cfg.wind_mps,
        antisway_mode: sc.antisway.mode,
        drift: &drift,
    });
    if let Some(adv) = models.advisor.as_deref() {
        decision::apply_advisor(&mut decision, adv as &dyn Advisor, cfg.allow_advisor_escalation);
    }
    let scope = Scope::new(&spec.name, "all");
    let (rule_rate, op_rate) = match sds.as_mut() {
        Some(s) => {
            s.record_gates(&scope, &decision.gates);
            if let Some(a) = decision.agreement {
                s.record_agreement(&scope, "rule_laya", a);
            }
            let r = (s.agreement_rate(&scope, "rule_laya"), s.agreement_rate(&scope, "operator_laya"));
            if let Err(e) = s.flush() {
                notes.push(format!("SDS 저장 실패: {e}"));
            }
            r
        }
        None => (None, None),
    };
    decision.choice = Some(decision::choice_prompt(&decision, op_rate, rule_rate));

    let crosscheck = if cfg.crosscheck {
        // 횡이동(주행 · 횡행 속도 > 0.01 m/s) 중 로프 길이의 중앙값 기준. 횡이동이 없으면 전체 표본
        let sm = &cycle.samples;
        let mut ls: Vec<f64> = (0..sm.l.len())
            .filter(|&i| {
                sm.axes.first().and_then(|a| a.v.get(i)).map_or(false, |v| v.abs() > 0.01)
                    || sm.axes.get(1).and_then(|a| a.v.get(i)).map_or(false, |v| v.abs() > 0.01)
            })
            .map(|i| sm.l[i])
            .collect();
        if ls.is_empty() {
            ls = sm.l.clone();
        }
        ls.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let l_mid = ls.get(ls.len() / 2).cloned().unwrap_or(spec.structure.rope_min_m);
        Some(calib::crosscheck(&spec, l_mid, cycle.load_peak_kg.max(1.0), cfg.sling_length_m))
    } else {
        None
    };
    if models.ttm.is_none() {
        notes.push("TTM 모델 없이 Holt 감쇠 추세로 예측했습니다".into());
    }
    Ok(PipelineReport { crane: spec, cycle, duty, forecasts, decision, crosscheck, models: models.status(), notes })
}

/// 흔들림 억제 방식 비교 요약
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareRow {
    pub mode: String,
    pub cycle_time_s: f64,
    pub max_residual_sway_mm: f64,
    pub max_final_sway_mm: f64,
    pub max_settle_time_s: f64,
    pub energy_kwh: f64,
    pub hoist_peak_current_a: f64,
    pub travel_peak_torque_ratio: f64,
}

pub fn compare(sc: &Scenario, modes: &[AntiSwayMode]) -> Result<Vec<CompareRow>, String> {
    let reps = cyclesim::compare_antisway(sc, modes)?;
    Ok(reps
        .iter()
        .map(|r| CompareRow {
            mode: r.antisway.clone(),
            cycle_time_s: r.cycle_time_s,
            max_residual_sway_mm: r.moves.iter().map(|m| m.residual_sway_mm).fold(0.0, f64::max),
            max_final_sway_mm: r.moves.iter().map(|m| m.final_sway_mm).fold(0.0, f64::max),
            max_settle_time_s: r.moves.iter().map(|m| m.settle_time_s).fold(0.0, f64::max),
            energy_kwh: r.axes.iter().map(|a| a.energy_in_kwh).sum(),
            hoist_peak_current_a: r.axes[2].peak_current_a,
            travel_peak_torque_ratio: r.axes[0].peak_torque_ratio,
        })
        .collect())
}

/// 프리셋 목록 (크레인, 모터)
pub fn presets_json() -> serde_json::Value {
    serde_json::json!({
        "cranes": crane_profile::presets(),
        "motors": crate::motor::presets(),
        "antisway_modes": AntiSwayMode::all().iter().map(|m| m.key()).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_end_to_end_without_models() {
        let sc = cyclesim::demo_scenario("overhead_5t_16m", 3000.0, AntiSwayMode::Zvd);
        let cfg = PipelineConfig { duty: Some(DutyConfig { cycles_per_hour: 20.0, hours: 2.0, ..Default::default() }), forecast_hours: 4.0, ..Default::default() };
        let dir = std::env::temp_dir().join(format!("crane_pipe_{}", std::process::id()));
        let cfg = PipelineConfig { sds_dir: Some(dir.to_string_lossy().into()), ..cfg };
        let r = run_pipeline(&sc, &cfg, &Models::none()).unwrap();
        assert!(r.duty.is_some());
        assert_eq!(r.forecasts.len(), 8);
        let hoist = r.forecasts.iter().find(|f| f.key == "hoist_winding_peak_c").unwrap();
        assert!(hoist.physics_residual);
        assert!(r.decision.choice.is_some());
        assert!(r.crosscheck.as_ref().unwrap().freq_ratio.is_finite());
        // 원장이 만들어졌다
        assert!(dir.join("motor.json").exists());
        let _ = std::fs::remove_dir_all(&dir);
        let js = serde_json::to_string(&r).unwrap();
        assert!(js.len() > 1000);
    }

    #[test]
    fn compare_rows() {
        let sc = cyclesim::demo_scenario("overhead_5t_16m", 3000.0, AntiSwayMode::Off);
        let rows = compare(&sc, &[AntiSwayMode::Off, AntiSwayMode::Zvd]).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows[1].max_residual_sway_mm < rows[0].max_residual_sway_mm);
    }

    #[test]
    fn find_model_dir_requires_config_and_weights() {
        let root = std::env::temp_dir().join(format!("crane_models_{}", std::process::id()));
        let d = root.join("granite-timeseries-ttm-r3").join("52-16");
        std::fs::create_dir_all(&d).unwrap();
        assert!(find_model_dir(&root, "ttm").is_none());
        std::fs::write(d.join("config.json"), "{}").unwrap();
        std::fs::write(d.join("model.safetensors"), "").unwrap();
        // 가중치가 있는 하위 폴더 이름에 ttm 이 없어도 경로로 찾는다
        assert_eq!(find_model_dir(&root, "ttm").as_deref(), Some(d.as_path()));
        assert!(find_model_dir(&root, "laya").is_none());
        // LAYA 는 Hugging Face 배치 그대로 (루트에 config.json 없이 encoder/config.json)
        let l = root.join("laya-typed-decisions");
        std::fs::create_dir_all(l.join("encoder")).unwrap();
        std::fs::write(l.join("model.safetensors"), "").unwrap();
        assert!(find_model_dir(&root, "laya").is_none());
        std::fs::write(l.join("encoder").join("config.json"), "{}").unwrap();
        assert_eq!(find_model_dir(&root, "laya").as_deref(), Some(l.as_path()));
        // root 가 곧 모델 폴더여도 찾는다
        assert_eq!(find_model_dir(&l, "laya").as_deref(), Some(l.as_path()));
        let _ = std::fs::remove_dir_all(&root);
    }
}
