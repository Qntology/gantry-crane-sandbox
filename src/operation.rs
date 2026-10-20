//! 현장 운영 시뮬레이션 — 현장 프리셋(물량 · 하중 · 환경 · 기후 · 운영 조건)으로 며칠 · 몇 달을 한 시간 단위로 돌린다.
//!
//! 1. 하중 스펙트럼의 구간마다 대표 운반 사이클을 동역학으로 한 번씩 돌려(정상 · 감속 속도, 바람 기준 하나)
//!    사이클당 증분표를 만든다: 사이클 시간 · 에너지 · 로프 손상 · 브레이크 체결 · 전부하 환산 시간 · 모터 손실 · 주행 거리.
//! 2. 시간마다 날씨(생성기 + 예보)를 보고 알림을 띄운다. 사람이 판단할 일이면 결정 카드(선택지 · 권장 · 막힌 선택지)를
//!    열고, 정책이 `Ask` 면 멈춰서 선택을 기다린다 (Claude Code 의 선택 질문과 같은 흐름).
//! 3. 운전 가능한 시간 · 속도로 처리량을 정하고, 처리한 이동만큼 부품 사용량과 위험(사고율)을 쌓는다.
//! 4. 하루가 끝나면 내구도 원장에 더하고 저장한다. 부품 상태가 바뀌면 정비 결정 카드가 열린다.
//!
//! 사고는 위험 모델의 시간당 사고율로 시드가 있는 난수에서 뽑는다(확률 모델). 폭풍 일주(밀림)는 바람 하중이 제동 ·
//! 클램프 · 계류 저항을 넘으면 결정론적으로 일어난다.

use crate::alerts::{Alert, AlertBook, AlertLevel, AlertPolicy, RiskInputs, RiskModel};
use crate::choice::{ChoiceOption, ChoicePrompt};
use crate::crane_profile::{Axis, CraneSpec, G};
use crate::cyclesim::{self, CycleReport};
use crate::environment::Environment;
use crate::field_constants;
use crate::health::{self, DayUsage, DurabilityModel, DurabilityReport, Health, HealthLedger, Margins, StatusChange};
use crate::rng::Rng;
use crate::site::{self, SitePreset};
use crate::weather::{self, Forecaster, HourWeather, MonthStats, WeatherOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionPolicy {
    /// 결정 카드마다 멈춰서 사람의 선택을 기다린다
    Ask,
    /// 시스템 권장을 고른다
    Recommended,
    /// 고를 수 있는 것 중 가장 보수적인 것
    Conservative,
    /// 고를 수 있는 것 중 가장 덜 보수적인 것 (알림을 무시하면 어떻게 되는지 보기)
    Risky,
}

impl DecisionPolicy {
    pub fn key(&self) -> &'static str {
        match self {
            DecisionPolicy::Ask => "ask",
            DecisionPolicy::Recommended => "recommended",
            DecisionPolicy::Conservative => "conservative",
            DecisionPolicy::Risky => "risky",
        }
    }
    pub fn from_key(k: &str) -> Option<Self> {
        [DecisionPolicy::Ask, DecisionPolicy::Recommended, DecisionPolicy::Conservative, DecisionPolicy::Risky].into_iter().find(|p| p.key() == k.trim())
    }
}

fn default_policy() -> DecisionPolicy {
    DecisionPolicy::Recommended
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationConfig {
    /// 자산(크레인 한 대) 이름 — 내구도 원장 파일 이름
    pub asset_id: String,
    pub days: u32,
    /// 시작 날짜 (월 · 일). 원장을 이어 쓰면 원장의 다음 날짜에서 시작한다
    #[serde(default)]
    pub start_month: Option<u32>,
    #[serde(default)]
    pub start_day: Option<u32>,
    #[serde(default)]
    pub seed: u64,
    #[serde(default = "default_policy")]
    pub policy: DecisionPolicy,
    /// 확률 사고를 뽑는가 (끄면 기대 사고 수만 센다)
    #[serde(default = "default_true")]
    pub incidents: bool,
    /// 이어서 쓸 원장이 있어도 새로 시작
    #[serde(default)]
    pub fresh: bool,
}

impl Default for OperationConfig {
    fn default() -> Self {
        Self { asset_id: "crane-01".into(), days: 30, start_month: None, start_day: None, seed: 1, policy: DecisionPolicy::Recommended, incidents: true, fresh: false }
    }
}

/// 하중 구간 하나의 사이클당 증분
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CycleRow {
    pub fraction: f64,
    pub load_kg: f64,
    pub share: f64,
    /// 동역학 사이클 시간 (취급 시간 제외, s)
    pub cycle_time_s: f64,
    pub energy_kwh: f64,
    pub regen_kwh: f64,
    pub rope_damage: f64,
    /// 로프 안전율 / 요구 안전율
    pub rope_sf_ratio: f64,
    /// 로프 최소 안전율 (동적 포함)
    #[serde(default)]
    pub rope_sf: f64,
    pub brake_sets: [f64; 3],
    pub brake_dyn_wear_mm: [f64; 3],
    pub run_s: [f64; 3],
    /// ∫(|τ|/τ_ref)³ dt (s)
    pub flh_s: [f64; 3],
    /// 사이클 평균 동손 · 철손+기계손 (모터 1대, W) · 냉각 계수 · 속도 비율
    pub copper_w: [f64; 3],
    pub other_w: [f64; 3],
    pub cooling: [f64; 3],
    pub speed_frac: [f64; 3],
    /// 권선–프레임 맥동 (평균 위 최대, K)
    pub ripple_k: [f64; 3],
    pub travel_km_eq: f64,
    pub trolley_km_eq: f64,
    pub residual_sway_mm: f64,
    pub peak_torque_ratio: [f64; 3],
    pub hoist_brake_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTable {
    pub normal: Vec<CycleRow>,
    pub reduced: Vec<CycleRow>,
    pub reduced_speed: f64,
    /// 바람 기준(평균 · 돌풍)에서 정격 60% 하중의 잔류 흔들림 (mm)
    pub wind_ref_mps: f64,
    pub wind_sway_mm: f64,
    pub calm_sway_mm: f64,
    /// 흔들림 주기 (s) — 바람 흔들림이 허용값을 넘으면 그만큼 기다린다
    pub sway_period_s: f64,
    pub sway_tolerance_mm: f64,
}

fn torque_ref(spec: &CraneSpec, a: Axis) -> f64 {
    match a {
        Axis::Hoist => spec.hoist_static_torque_nm(spec.structure.rated_load_kg).max(1e-6),
        _ => spec.drive(a).motor.rated_torque_nm().max(1e-6),
    }
}

/// 사이클 결과 → 증분 한 줄
pub fn cycle_row(spec: &CraneSpec, r: &CycleReport, fraction: f64, share: f64) -> CycleRow {
    let s = &r.samples;
    let dt = if s.t.len() > 1 { (s.t[s.t.len() - 1] - s.t[0]) / (s.t.len() - 1) as f64 } else { 0.05 };
    let st = &spec.structure;
    let mut row = CycleRow {
        fraction,
        load_kg: r.load_peak_kg,
        share,
        cycle_time_s: r.cycle_time_s,
        energy_kwh: r.axes.iter().map(|a| a.energy_in_kwh).sum(),
        regen_kwh: r.axes.iter().map(|a| a.energy_regen_kwh).sum(),
        rope_damage: r.rope.damage,
        rope_sf_ratio: if r.rope.required_safety_factor > 0.0 { r.rope.min_safety_factor / r.rope.required_safety_factor } else { 10.0 },
        rope_sf: r.rope.min_safety_factor.min(99.0),
        residual_sway_mm: r.moves.iter().map(|m| m.residual_sway_mm).fold(0.0, f64::max),
        hoist_brake_factor: r.hoist_brake_factor,
        ..Default::default()
    };
    for a in Axis::all() {
        let k = a.index();
        let d = spec.drive(a);
        let ax = &r.axes[k];
        row.brake_sets[k] = ax.brake_operations as f64;
        row.brake_dyn_wear_mm[k] = (d.brake.lining_new_mm - ax.brake_lining_mm).max(0.0);
        row.run_s[k] = ax.run_time_s;
        row.peak_torque_ratio[k] = ax.peak_torque_ratio;
        let tr = torque_ref(spec, a);
        row.flh_s[k] = s.axes[k].torque_nm.iter().map(|t| (t.abs() / tr).powi(3) * dt).sum();
        let lp = &r.loss_profiles[k];
        let n = lp.copper_w.len().max(1) as f64;
        row.copper_w[k] = lp.copper_w.iter().sum::<f64>() / n;
        row.other_w[k] = lp.other_w.iter().sum::<f64>() / n;
        let wn = d.motor.rated_speed_rad_s().max(1e-9);
        row.cooling[k] = lp.omega.iter().map(|w| d.motor.cooling_factor(*w)).sum::<f64>() / n;
        row.speed_frac[k] = lp.omega.iter().map(|w| (w.abs() / wn).min(2.0)).sum::<f64>() / n;
        // 권선–프레임 상승의 1차 응답 최대 − 평균
        let net = d.motor.thermal_network();
        let tau = d.motor.thermal.tau_winding_s.max(1.0);
        let mean = row.copper_w[k] * net.r_wf;
        let mut x = mean;
        let mut peak = mean;
        let a1 = 1.0 - (-lp.dt.max(1e-6) / tau).exp();
        for p in lp.copper_w.iter() {
            x += (p * net.r_wf - x) * a1;
            peak = peak.max(x);
        }
        row.ripple_k[k] = (peak - mean).max(0.0);
    }
    // 바퀴 하중비 가중 주행 거리 (마모 ∝ 하중 × 거리)
    let w_rated_t = st.bridge_mass_kg + st.trolley_mass_kg + st.hook_mass_kg + st.rated_load_kg;
    let w_rated_c = st.trolley_mass_kg + st.hook_mass_kg + st.rated_load_kg;
    for i in 0..s.t.len() {
        let load = s.load_kg.get(i).cloned().unwrap_or(0.0);
        let vt = s.axes[0].v.get(i).cloned().unwrap_or(0.0).abs();
        let vc = s.axes[1].v.get(i).cloned().unwrap_or(0.0).abs();
        row.travel_km_eq += vt * dt / 1000.0 * (st.bridge_mass_kg + st.trolley_mass_kg + st.hook_mass_kg + load) / w_rated_t;
        row.trolley_km_eq += vc * dt / 1000.0 * (st.trolley_mass_kg + st.hook_mass_kg + load) / w_rated_c;
    }
    row
}

/// 현장의 사이클 증분표를 만든다
pub fn build_table(site: &SitePreset) -> Result<CycleTable, String> {
    let spec = site.spec()?;
    let rated = spec.structure.rated_load_kg;
    let env = Environment { ambient_c: 20.0, ..Default::default() };
    let reduced = field_constants::REDUCED_SPEED;
    let mut normal = Vec::new();
    let mut slow = Vec::new();
    let total: f64 = site.loads.spectrum.iter().map(|b| b.share.max(0.0)).sum::<f64>().max(1e-12);
    for b in site.loads.spectrum.iter() {
        let load = (b.fraction.max(0.0) * rated).max(1.0);
        let share = b.share.max(0.0) / total;
        let r1 = cyclesim::simulate(&site.scenario(load, 1.0, env.clone())?)?;
        normal.push(cycle_row(&spec, &r1, b.fraction, share));
        let r2 = cyclesim::simulate(&site.scenario(load, reduced, env.clone())?)?;
        slow.push(cycle_row(&spec, &r2, b.fraction, share));
    }
    // 바람 기준 흔들림 (옥외만)
    let w_ref = field_constants::WIND_REF_MPS;
    let ref_load = 0.6 * rated;
    let calm = cyclesim::simulate(&site.scenario(ref_load, 1.0, env.clone())?)?;
    let calm_sway = calm.moves.iter().filter(|m| m.load_kg > 0.0).map(|m| m.residual_sway_mm).fold(0.0, f64::max);
    let wind_sway = if site.environment.indoor {
        calm_sway
    } else {
        let wenv = Environment { ambient_c: 20.0, wind_mean_mps: [0.0, w_ref / 1.5], gust_amp_mps: w_ref - w_ref / 1.5, ..Default::default() };
        let w = cyclesim::simulate(&site.scenario(ref_load, 1.0, wenv)?)?;
        w.moves.iter().filter(|m| m.load_kg > 0.0).map(|m| m.residual_sway_mm).fold(0.0, f64::max)
    };
    let l_mid = spec.structure.rope_min_m + 0.5 + 0.5 * site.throughput.lift_m;
    Ok(CycleTable {
        normal,
        reduced: slow,
        reduced_speed: reduced,
        wind_ref_mps: w_ref,
        wind_sway_mm: wind_sway,
        calm_sway_mm: calm_sway,
        sway_period_s: 2.0 * std::f64::consts::PI * (l_mid / G).sqrt(),
        sway_tolerance_mm: spec.limits.sway_tolerance_mm,
    })
}

impl CycleTable {
    pub fn rows(&self, reduced: bool) -> &Vec<CycleRow> {
        if reduced {
            &self.reduced
        } else {
            &self.normal
        }
    }

    /// 바람 흔들림 때문에 이동마다 더 기다리는 시간 (s): 허용값을 넘는 배수만큼 흔들림 주기
    pub fn wind_penalty_s(&self, gust_mps: f64, speed_scale: f64) -> f64 {
        if self.wind_ref_mps <= 0.0 || self.sway_tolerance_mm <= 0.0 {
            return 0.0;
        }
        let extra = (self.wind_sway_mm - self.calm_sway_mm).max(0.0) * (gust_mps / self.wind_ref_mps).powi(2) * speed_scale.clamp(0.3, 1.0);
        let sway = self.calm_sway_mm + extra;
        ((sway - self.sway_tolerance_mm) / self.sway_tolerance_mm).clamp(0.0, 3.0) * self.sway_period_s
    }

    /// 하중 비중으로 평균한 이동 시간 (s)
    pub fn mean_cycle_s(&self, reduced: bool) -> f64 {
        self.rows(reduced).iter().map(|r| r.share * r.cycle_time_s).sum()
    }
}

/// 결정 카드 하나
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpDecision {
    pub id: String,
    pub day: u32,
    pub hour: u8,
    pub at: String,
    /// "wind" · "storm" · "visibility" · "heat" · "cold" · "component" · "inspection" · "legal" · "critical_lift" · "overtime"
    pub topic: String,
    #[serde(default)]
    pub component: Option<String>,
    pub prompt: ChoicePrompt,
    pub chosen: Option<String>,
    pub decided_by: Option<String>,
    pub outcome: Option<String>,
    /// 결정 시점 위험 지수
    pub risk_index: f64,
    /// 가장 덜 보수적인 선택 대비 줄인 기대 사고 수 (고른 뒤 계산)
    #[serde(default)]
    pub risk_avoided: f64,
    /// (선택지, 예상 영향 시간 동안의 기대 사고 수)
    #[serde(default)]
    pub expected_by_option: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub day: u32,
    pub hour: u8,
    pub at: String,
    pub kind: String,
    /// "아차사고" · "재산 피해" · "중대"
    pub severity: String,
    pub causes: Vec<(String, f64)>,
    pub downtime_h: f64,
    pub detail: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DayRow {
    pub day: u32,
    pub date: String,
    pub scheduled_h: f64,
    pub op_h: f64,
    pub moves: f64,
    pub demand: f64,
    pub backlog: f64,
    pub downtime_h: f64,
    pub max_gust_mps: f64,
    pub temp_max_c: f64,
    pub temp_min_c: f64,
    pub feels_like_max_c: f64,
    pub rain_mm: f64,
    pub thunder: bool,
    pub min_visibility_m: f64,
    pub risk_max: f64,
    pub expected_incidents: f64,
    pub alerts: u32,
    pub incidents: u32,
    pub energy_kwh: f64,
    pub worst_component: String,
    pub worst_life: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Kpis {
    pub days: u32,
    pub scheduled_h: f64,
    pub operating_h: f64,
    /// 운전 가능 시간 / 계획 시간
    pub availability: f64,
    pub downtime_by_reason: Vec<(String, f64)>,
    pub demand_moves: f64,
    pub moves: f64,
    pub backlog_moves: f64,
    /// 운전 시간당 이동 (회/h)
    pub moves_per_op_hour: f64,
    pub energy_kwh: f64,
    pub regen_kwh: f64,
    pub expected_incidents: f64,
    pub incidents: u32,
    pub risk_mean: f64,
    pub risk_max: f64,
    pub alerts_by_level: Vec<(String, u32)>,
    pub decisions: u32,
    pub decisions_by_operator: u32,
    pub risk_avoided: f64,
    pub replacements: u32,
    pub maintenance_h: f64,
    pub refused_overweight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationReport {
    pub site: String,
    pub site_name: String,
    pub asset_id: String,
    pub crane: String,
    pub start_date: String,
    pub policy: String,
    pub seed: u64,
    pub kpi: Kpis,
    pub daily: Vec<DayRow>,
    pub alerts: Vec<Alert>,
    pub decisions: Vec<OpDecision>,
    pub incidents: Vec<Incident>,
    pub durability: DurabilityReport,
    pub ledger: HealthLedger,
    pub table: CycleTable,
    pub weather_months: Vec<MonthStats>,
    pub finished: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum StepOutcome {
    Finished,
    Decision { decision: Box<OpDecision> },
    Paused { hour_index: usize },
}

/// 정비 작업 (교체 · 점검 · 검사 · 조사)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    kind: String,
    component: Option<String>,
    hours_left: f64,
    /// true 면 비번 시간에 한다 (생산 손실 없음)
    off_shift: bool,
    reason: String,
    started: bool,
    /// 이 수명 사용 비율에 닿기 전에는 시작하지 않는다 (계획 교체)
    #[serde(default)]
    min_life: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Modifiers {
    /// 운전 중지 · 계류 구간 (시작, 끝, 이유) — 바람 · 시정 · 낙뢰 · 태풍
    parks: Vec<(usize, usize, String)>,
    /// 감속 운전 끝 시간
    slow_until: Option<usize>,
    slow_scale: f64,
    /// 폭염 휴식 적용 (끝 시간), 14–17시 옥외 중지 포함 여부
    heat_rest_until: Option<usize>,
    heat_midday_stop: bool,
    /// 폭풍 대비 계류(타이다운) 구간 (시작, 끝)
    ties: Vec<(usize, usize)>,
    /// 중량물 작업계획서가 유효한 마지막 날 (그날 포함)
    lift_plan_until: Option<u32>,
    /// 오늘 연장 근무 (h) · 교대 인력인가
    overtime_day: Option<u32>,
    overtime_h: f64,
    overtime_relief: bool,
    /// 마지막으로 연장 근무를 물은 날
    overtime_asked: Option<u32>,
    /// 정기 점검 연기 끝 날
    periodic_postponed_to: Option<f64>,
    /// 폭풍 일주 사고가 난 폭풍 (중복 방지 끝 시간)
    runaway_until: Option<usize>,
}

pub struct OpSim {
    pub site: SitePreset,
    pub spec: CraneSpec,
    pub model: DurabilityModel,
    pub risk: RiskModel,
    pub policy: AlertPolicy,
    pub cfg: OperationConfig,
    pub weather: Vec<HourWeather>,
    pub forecaster: Forecaster,
    pub table: CycleTable,
    pub ledger: HealthLedger,
    start_doy: u32,
    day0: f64,
    /// 크레인 높이 풍속 / 10 m 풍속
    hfac: f64,
    idx: usize,
    rng_demand: Rng,
    rng_incident: Rng,
    book: AlertBook,
    mods: Modifiers,
    tasks: Vec<Task>,
    backlog: f64,
    demand_factor: f64,
    usage: DayUsage,
    day: DayRow,
    pending: Option<OpDecision>,
    alerts: Vec<Alert>,
    decisions: Vec<OpDecision>,
    incidents: Vec<Incident>,
    daily: Vec<DayRow>,
    kpi: Kpis,
    downtime: Vec<(String, f64)>,
    risk_sum: f64,
    risk_n: f64,
    next_id: u32,
    /// 원장 상태 변화로 열 결정 (다음 시간에)
    queued_changes: Vec<StatusChange>,
    notes: Vec<String>,
    finished: bool,
}

const NIGHT: [u8; 8] = [22, 23, 0, 1, 2, 3, 4, 5];

fn level_from(v: f64, steps: &[(f64, AlertLevel)]) -> Option<AlertLevel> {
    let mut out = None;
    for (th, l) in steps {
        if v >= *th {
            out = Some(*l);
        }
    }
    out
}

impl OpSim {
    pub fn new(site: SitePreset, cfg: OperationConfig, ledger: Option<HealthLedger>) -> Result<Self, String> {
        site.validate()?;
        let table = build_table(&site)?;
        Self::with_table(site, cfg, ledger, table)
    }

    /// 미리 만든 사이클 증분표로 시작한다 (보정처럼 같은 현장을 여러 번 돌릴 때)
    pub fn with_table(site: SitePreset, cfg: OperationConfig, ledger: Option<HealthLedger>, table: CycleTable) -> Result<Self, String> {
        site.validate()?;
        let spec = site.spec()?;
        let climate = site.climate.resolve()?;
        if cfg.days == 0 || cfg.days > 3660 {
            return Err("days 는 1–3660 이어야 합니다".into());
        }
        let mut ledger = match ledger {
            Some(l) if !cfg.fresh => {
                if l.crane != spec.name {
                    return Err(format!("원장 '{}' 의 크레인({})과 현장 크레인({})이 다릅니다", l.asset_id, l.crane, spec.name));
                }
                l
            }
            _ => HealthLedger::new(&cfg.asset_id, &spec, &site),
        };
        let start_doy = match (cfg.start_month, cfg.start_day) {
            (Some(m), d) => weather::doy_of(m, d.unwrap_or(1)),
            _ => ledger.next_doy,
        };
        let seed = cfg.seed ^ (ledger.runs as u64).wrapping_mul(0x9E37_79B9) ^ (ledger.day as u64).wrapping_mul(0x85EB_CA6B);
        let w = weather::generate(&climate, &WeatherOptions { start_doy, days: cfg.days + 2, height_m: site.environment.crane_height_m, seed });
        let weather: Vec<HourWeather> = if site.environment.indoor {
            w.iter().map(|h| h.indoor(site.environment.indoor_base_c, site.environment.indoor_coupling, site.environment.heat_source_c)).collect()
        } else {
            w
        };
        let policy = site.alerts.for_wind_limit(spec.limits.wind_in_service_mps);
        let root = Rng::new(seed);
        let day0 = ledger.day;
        let hfac = if site.environment.indoor { 1.0 } else { (site.environment.crane_height_m.max(2.0) / 10.0).powf(climate.alpha.clamp(0.0, 0.4)) };
        ledger.runs += 1;
        let mut s = Self {
            policy,
            model: DurabilityModel::default(),
            risk: RiskModel::default(),
            forecaster: Forecaster { seed: seed ^ 0xF0CA, ..Default::default() },
            weather,
            table,
            ledger,
            start_doy,
            day0,
            hfac,
            idx: 0,
            rng_demand: root.fork(1),
            rng_incident: root.fork(2),
            book: AlertBook::default(),
            mods: Modifiers { slow_scale: 1.0, ..Default::default() },
            tasks: Vec::new(),
            backlog: 0.0,
            demand_factor: 1.0,
            usage: DayUsage::default(),
            day: DayRow::default(),
            pending: None,
            alerts: Vec::new(),
            decisions: Vec::new(),
            incidents: Vec::new(),
            daily: Vec::new(),
            kpi: Kpis::default(),
            downtime: Vec::new(),
            risk_sum: 0.0,
            risk_n: 0.0,
            next_id: 1,
            queued_changes: Vec::new(),
            notes: Vec::new(),
            finished: false,
            spec,
            site,
            cfg,
        };
        s.start_day();
        Ok(s)
    }

    pub fn pending(&self) -> Option<&OpDecision> {
        self.pending.as_ref()
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn hour_index(&self) -> usize {
        self.idx
    }

    fn total_hours(&self) -> usize {
        self.cfg.days as usize * 24
    }

    fn day_num(&self, idx: usize) -> u32 {
        (idx / 24) as u32
    }

    fn abs_day(&self, idx: usize) -> f64 {
        self.day0 + (idx / 24) as f64
    }

    fn at(&self, idx: usize) -> String {
        format!("D{} {:02}:00", self.day_num(idx) + 1, idx % 24)
    }

    // ---------- 근무 일정 ----------

    fn work_hours(&self) -> f64 {
        (self.site.throughput.shifts_per_day as f64 * self.site.throughput.shift_hours).min(24.0)
    }

    /// (계획 근무 시간인가, 교대 안에서 몇 번째 시간인가, 연장 근무인가)
    fn schedule(&self, idx: usize) -> (bool, f64, bool) {
        let t = &self.site.throughput;
        let total = self.work_hours();
        let h_abs = idx as f64;
        for back in 0..2usize {
            let d = (idx / 24) as i64 - back as i64;
            if d < 0 {
                continue;
            }
            let weekday = ((self.ledger.day as i64 + d) % 7) as u32;
            if weekday >= t.days_per_week {
                continue;
            }
            let start = d as f64 * 24.0 + t.first_shift_start_h as f64;
            let ot = if self.mods.overtime_day == Some(d as u32) { self.mods.overtime_h } else { 0.0 };
            if h_abs >= start && h_abs < start + total + ot {
                let into = h_abs - start;
                let overtime = into >= total;
                let on_shift = if overtime {
                    (into - total) + 1.0 + if self.mods.overtime_relief { 0.0 } else { t.shift_hours }
                } else {
                    into % t.shift_hours.max(1.0) + 1.0
                };
                return (true, on_shift, overtime);
            }
        }
        (false, 0.0, false)
    }

    /// 법정 안전검사 기한 (일): 최초는 설치 후 `legal_first_days`, 그 뒤 `legal_inspection_days` 마다
    fn legal_interval(&self) -> f64 {
        let ops = &self.site.operations;
        let first = !self.ledger.events.iter().any(|e| e.kind == "legal_inspection");
        if first {
            ops.legal_first_days as f64
        } else {
            ops.legal_inspection_days as f64
        }
    }

    // ---------- 결정 카드 ----------

    fn new_id(&mut self) -> String {
        let id = format!("d{:04}", self.next_id);
        self.next_id += 1;
        id
    }

    #[allow(clippy::too_many_arguments)]
    fn option(key: &str, label: &str, detail: String, recommended: bool, enabled: bool, blocked_by: &[&str]) -> ChoiceOption {
        ChoiceOption {
            key: key.into(),
            label: label.into(),
            detail,
            recommended,
            rule: recommended,
            advisor_probability: None,
            advisor_top: false,
            enabled,
            blocked_by: blocked_by.iter().map(|s| s.to_string()).collect(),
            chosen: false,
        }
    }

    fn rank(key: &str) -> u8 {
        match key {
            "continue" | "defer" | "standard" | "overtime" | "postpone" | "lift_anyway" => 0,
            "reduce_speed" | "relief" | "replace_offshift" | "schedule" => 1,
            "signalman_slow" | "rest" | "inspect_now" | "inspect" | "lift_plan" | "carry" | "park_at_stop" => 2,
            "rest_and_midday_stop" | "park" | "stop" | "replace_now" | "legal_inspection" | "refuse" => 3,
            "storm_secure" => 4,
            _ => 2,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn open_decision(&mut self, topic: &str, component: Option<&str>, header: &str, question: String, mut options: Vec<ChoiceOption>, attention: Vec<String>, expected: Vec<(String, f64)>) -> Option<StepOutcome> {
        let id = self.new_id();
        let rec = options.iter().find(|o| o.recommended).map(|o| o.key.clone()).unwrap_or_default();
        let mut prompt = ChoicePrompt {
            kind: if matches!(topic, "component" | "inspection" | "legal") { "maintenance".into() } else { "operation".into() },
            header: header.into(),
            question,
            options: std::mem::take(&mut options),
            recommended: rec.clone(),
            decided_by: "rule".into(),
            needs_attention: false,
            attention,
            operator_choice: None,
            advisor_note: "규칙 판단 (현장 운영 알림)".into(),
            agreement: Vec::new(),
        };
        // 권장 다음에는 더 안전한 선택지부터, 위험한 선택지는 뒤로
        prompt.sort_options(|k| 9 - Self::rank(k));
        prompt.settle_attention();
        let risk_index = self.current_risk(self.idx).index;
        let d = OpDecision {
            id,
            day: self.day_num(self.idx),
            hour: (self.idx % 24) as u8,
            at: self.at(self.idx),
            topic: topic.into(),
            component: component.map(String::from),
            prompt,
            chosen: None,
            decided_by: None,
            outcome: None,
            risk_index,
            risk_avoided: 0.0,
            expected_by_option: expected,
        };
        if let Some(a) = self.alerts.last_mut() {
            if a.decision_id.is_none() && a.day == d.day && a.hour == d.hour {
                a.decision_id = Some(d.id.clone());
            }
        }
        let enabled = d.prompt.options.iter().filter(|o| o.enabled).count();
        if enabled <= 1 || self.cfg.policy != DecisionPolicy::Ask {
            let key = match self.cfg.policy {
                DecisionPolicy::Conservative => d.prompt.options.iter().filter(|o| o.enabled).max_by_key(|o| Self::rank(&o.key)).map(|o| o.key.clone()),
                DecisionPolicy::Risky => d.prompt.options.iter().filter(|o| o.enabled).min_by_key(|o| Self::rank(&o.key)).map(|o| o.key.clone()),
                _ => Some(rec.clone()).filter(|k| !k.is_empty()).or_else(|| d.prompt.options.iter().find(|o| o.enabled).map(|o| o.key.clone())),
            }
            .unwrap_or_default();
            let who = if enabled <= 1 { "rule".to_string() } else { format!("policy:{}", self.cfg.policy.key()) };
            self.pending = Some(d);
            let _ = self.decide_inner(&key, &who);
            None
        } else {
            self.pending = Some(d.clone());
            Some(StepOutcome::Decision { decision: Box::new(d) })
        }
    }

    /// 사람이 고른 선택을 반영한다
    pub fn decide(&mut self, decision_id: &str, key: &str) -> Result<OpDecision, String> {
        match &self.pending {
            Some(p) if p.id == decision_id => {}
            Some(p) => return Err(format!("기다리는 결정은 '{}' 입니다 ('{}' 아님)", p.id, decision_id)),
            None => return Err("기다리는 결정이 없습니다".into()),
        }
        self.decide_inner(key, "operator")
    }

    fn decide_inner(&mut self, key: &str, who: &str) -> Result<OpDecision, String> {
        let mut d = self.pending.take().ok_or("기다리는 결정이 없습니다")?;
        let Some(opt) = d.prompt.options.iter().find(|o| o.key == key).cloned() else {
            let keys: Vec<String> = d.prompt.options.iter().map(|o| o.key.clone()).collect();
            self.pending = Some(d);
            return Err(format!("'{key}' 는 선택지가 아닙니다 ({})", keys.join(", ")));
        };
        if !opt.enabled {
            let why = opt.blocked_by.join(", ");
            self.pending = Some(d);
            return Err(format!("'{}' 은 안전 기준으로 막혀 있습니다 ({why})", opt.label));
        }
        for o in d.prompt.options.iter_mut() {
            o.chosen = o.key == key;
        }
        d.prompt.operator_choice = if who == "operator" { Some(key.into()) } else { None };
        d.prompt.decided_by = if who == "operator" { "operator".into() } else { who.into() };
        d.prompt.settle_attention();
        let outcome = self.apply_effect(&d, key);
        d.chosen = Some(key.into());
        d.decided_by = Some(who.into());
        d.outcome = Some(outcome.clone());
        let allowed = |k: &str| d.prompt.options.iter().any(|o| o.key == k && o.enabled);
        let worst = d.expected_by_option.iter().filter(|(k, _)| allowed(k)).map(|(_, e)| *e).fold(0.0, f64::max);
        let mine = d.expected_by_option.iter().find(|(k, _)| k == key).map(|(_, e)| *e).unwrap_or(worst);
        d.risk_avoided = (worst - mine).max(0.0);
        self.kpi.decisions += 1;
        if who == "operator" {
            self.kpi.decisions_by_operator += 1;
        }
        self.kpi.risk_avoided += d.risk_avoided;
        let day = self.abs_day(self.idx);
        self.ledger.note(day, (self.idx % 24) as u8, "decision", d.component.as_deref(), format!("{} — {} 선택 ({}): {}", d.prompt.header, opt.label, d.prompt.decided_by, outcome));
        self.decisions.push(d.clone());
        Ok(d)
    }

    fn push_task(&mut self, kind: &str, component: Option<&str>, hours: f64, off_shift: bool, reason: &str) {
        if self.tasks.iter().any(|t| t.kind == kind && t.component.as_deref() == component) {
            return;
        }
        self.tasks.push(Task { kind: kind.into(), component: component.map(String::from), hours_left: hours, off_shift, reason: reason.into(), started: false, min_life: None });
    }

    /// 바람이 `level` 아래로 내려가는 시간 (예보, 최대 48 h 뒤)
    fn hours_until_below(&self, from: usize, level: f64) -> usize {
        let mut calm_run = 0;
        for k in 1..=48 {
            let o = self.forecaster.outlook(&self.weather, from + k - 1, 1);
            if o.max_gust_mps < level {
                calm_run += 1;
                if calm_run >= 2 {
                    return k - 1;
                }
            } else {
                calm_run = 0;
            }
        }
        48
    }

    fn apply_effect(&mut self, d: &OpDecision, key: &str) -> String {
        let i = self.idx;
        let comp = d.component.clone();
        match (d.topic.as_str(), key) {
            ("wind", "park") | ("visibility", "stop") | ("cold", "stop") => {
                let lvl = if d.topic == "wind" { self.policy.wind_warn_mps } else { 0.0 };
                let h = if d.topic == "wind" { self.hours_until_below(i, lvl).max(1) } else { 2 };
                let reason = match d.topic.as_str() {
                    "wind" => "바람",
                    "visibility" => "시정",
                    _ => "한파 · 결빙",
                };
                self.park_now(i + h, reason);
                format!("{h} h 운전 중지 · 계류")
            }
            ("wind", "reduce_speed") | ("cold", "reduce_speed") => {
                self.mods.slow_until = Some(i + 4);
                self.mods.slow_scale = self.table.reduced_speed;
                format!("4 h 동안 속도 ×{:.1}", self.table.reduced_speed)
            }
            ("visibility", "signalman_slow") => {
                self.mods.slow_until = Some(i + 3);
                self.mods.slow_scale = self.table.reduced_speed;
                "신호수 배치 · 3 h 감속".into()
            }
            ("storm", "storm_secure") => {
                let ol = self.forecaster.outlook(&self.weather, i, self.policy.storm_lookahead_h);
                let peak = i + ol.hour_of_max.max(1) as usize;
                let from = peak.saturating_sub(field_constants::STORM_PREP_H).max(i);
                let until = peak + self.hours_until_below(peak, self.policy.wind_warn_mps) + 2;
                self.mods.parks.push((from, until, "태풍 · 폭풍".into()));
                self.mods.ties.push((from, until));
                self.usage.clamp_ops += 1.0;
                format!("{} 부터 작업 종료 · 레일 클램프 · 계류(타이다운), {} 까지 ({} h)", self.at(from), self.at(until), until - from)
            }
            ("storm", "park_at_stop") => "정지 기준에 닿으면 클램프만 체결".into(),
            ("heat", "rest") => {
                self.mods.heat_rest_until = Some((i / 24 + 1) * 24);
                self.mods.heat_midday_stop = false;
                "오늘 매시간 휴식 · 물 · 그늘".into()
            }
            ("heat", "rest_and_midday_stop") => {
                self.mods.heat_rest_until = Some((i / 24 + 1) * 24);
                self.mods.heat_midday_stop = true;
                "오늘 휴식 + 14–17시 옥외 작업 중지".into()
            }
            ("component", "replace_now") => {
                let c = comp.clone().unwrap_or_default();
                let h = field_constants::replace_hours(&c);
                self.push_task("replace", Some(&c), h, false, "교체 기준");
                format!("지금 교체 ({h:.0} h 정지)")
            }
            ("component", "replace_offshift") => {
                let c = comp.clone().unwrap_or_default();
                let h = field_constants::replace_hours(&c);
                self.push_task("replace", Some(&c), h, true, "비번 시간 정비");
                format!("다음 비번 시간에 교체 ({h:.0} h)")
            }
            ("component", "schedule") => {
                let c = comp.clone().unwrap_or_default();
                let h = field_constants::replace_hours(&c);
                self.push_task("replace", Some(&c), h, true, "계획 교체");
                let at = 0.95 * self.model.replace_at;
                if let Some(t) = self.tasks.last_mut() {
                    t.min_life = Some(at);
                }
                format!("수명 사용 {:.0}% 에 닿으면 비번 시간 교체 ({h:.0} h), 부품 준비", at * 100.0)
            }
            ("component", "inspect_now") => {
                let c = comp.clone().unwrap_or_default();
                self.push_task("inspect", Some(&c), 2.0, false, "상태 점검");
                "지금 점검 (2 h)".into()
            }
            ("inspection", "inspect") => {
                self.push_task("periodic_inspection", None, field_constants::PERIODIC_INSPECTION_H, true, "정기 점검");
                format!("비번 시간 정기 점검 ({:.0} h)", field_constants::PERIODIC_INSPECTION_H)
            }
            ("inspection", "postpone") => {
                self.mods.periodic_postponed_to = Some(self.abs_day(i) + 7.0);
                "정기 점검 1주 연기".into()
            }
            ("legal", "legal_inspection") => {
                self.push_task("legal_inspection", None, field_constants::LEGAL_INSPECTION_H, false, "법정 안전검사");
                format!("안전검사 수검 ({:.0} h)", field_constants::LEGAL_INSPECTION_H)
            }
            ("critical_lift", "lift_plan") => {
                let d = self.day_num(i);
                self.mods.lift_plan_until = Some(d + field_constants::LIFT_PLAN_VALID_DAYS - 1);
                format!("{} 일간 중량물 작업계획서 · 신호수 · 유도 로프 (반복 작업 표준 계획)", field_constants::LIFT_PLAN_VALID_DAYS)
            }
            ("overtime", "overtime") | ("overtime", "relief") => {
                let h = self.site.operations.max_overtime_h.max(0.0);
                self.mods.overtime_day = Some(self.day_num(i).saturating_sub(if (i % 24) as u32 >= self.site.throughput.first_shift_start_h { 0 } else { 1 }));
                self.mods.overtime_h = h;
                self.mods.overtime_relief = key == "relief";
                if key == "relief" {
                    format!("교대 인력 투입 · {h:.0} h 연장")
                } else {
                    format!("같은 운전원 {h:.0} h 연장")
                }
            }
            ("incident", _) => {
                self.push_task("investigate", comp.as_deref(), field_constants::INVESTIGATION_H, false, "사고 조사 · 점검");
                format!("작업 중지 · 사고 조사 ({:.0} h)", field_constants::INVESTIGATION_H)
            }
            (_, "continue") | (_, "defer") | (_, "standard") | (_, "carry") => "그대로 진행".into(),
            (t, k) => format!("{t}/{k}"),
        }
    }

    // ---------- 알림 ----------

    #[allow(clippy::too_many_arguments)]
    fn raise(&mut self, id: &str, level: AlertLevel, title: &str, message: String, action: &str, basis: &str, value: f64, limit: f64, unit: &str, lead_h: Option<u32>) {
        let i = self.idx;
        self.alerts.push(Alert {
            id: id.into(),
            day: self.day_num(i),
            hour: (i % 24) as u8,
            at: self.at(i),
            level,
            level_label: level.label().into(),
            title: title.into(),
            message,
            action: action.into(),
            basis: basis.into(),
            value,
            limit,
            unit: unit.into(),
            lead_h,
            decision_id: None,
        });
        self.day.alerts += 1;
    }

    fn outdoor(&self) -> bool {
        !self.site.environment.indoor
    }

    fn heat_exposed(&self) -> bool {
        self.site.operations.ground_crew || !self.site.operations.cab_air_conditioned || (self.site.environment.indoor && self.site.environment.heat_source_c > 0.0)
    }

    /// 이 시간의 알림과 결정 카드. 사람의 선택을 기다려야 하면 Some
    fn check_alerts(&mut self) -> Option<StepOutcome> {
        let i = self.idx;
        let w = self.weather[i].clone();
        let p = self.policy.clone();
        let (sched, _, _) = self.schedule(i);
        let soon = (1..=2).any(|k| self.schedule(i + k).0);
        let relevant = sched || soon;

        if self.outdoor() {
            // 태풍 · 폭풍 (근무와 상관없이)
            let ol = self.forecaster.outlook(&self.weather, i, p.storm_lookahead_h);
            // 기상청 강풍경보 · 제140조 는 지점(10 m) 순간풍속 기준 — 크레인 높이 예보를 10 m 로 되돌려 비교
            let g10 = ol.max_gust_mps / self.hfac;
            let storm_lvl = if ol.typhoon || g10 >= p.storm_mps { Some(AlertLevel::Danger) } else { None };
            if self.book.update_held("storm", storm_lvl, 6) {
                let lead = ol.hour_of_max;
                self.raise(
                    "storm",
                    AlertLevel::Danger,
                    "태풍 · 폭풍 예보",
                    format!("{} h 안에 순간풍속 {:.0} m/s (10 m 환산 {:.0} m/s) 예보 — 계류 기준 {:.0} m/s (10 m)", lead, ol.max_gust_mps, g10, p.storm_mps),
                    "작업 조기 종료, 레일 클램프 체결, 계류(타이다운) · 앵커 고정, 붐 · 트롤리 고정 위치",
                    "EN 13001-2 비운전 폭풍 상태, 기상청 태풍 특보",
                    g10,
                    p.storm_mps,
                    "m/s",
                    Some(lead),
                );
                let exp_secure = 0.0;
                let margin_clamp = self.runaway_margin(ol.max_gust_mps, false, true);
                let exp_clamp = if margin_clamp < 1.0 { 1.0 } else { 0.05 / margin_clamp };
                let opts = vec![
                    Self::option("storm_secure", "태풍 대비 고정", format!("조기 종료 · 클램프 · 타이다운 — 예상 일주 여유 {:.2}", self.runaway_margin(ol.max_gust_mps, true, true)), true, true, &[]),
                    Self::option("park_at_stop", "정지 기준까지 운전", format!("클램프만 — 예상 일주 여유 {margin_clamp:.2}"), false, margin_clamp >= 1.0, if margin_clamp >= 1.0 { &[] } else { &["클램프만으로는 폭풍 일주 여유 1 미만"] }),
                    Self::option("continue", "그대로 운전", "폭풍 예보 중 계속 운전".into(), false, false, &["폭풍 예보 (운전 허용 풍속 초과 예상)"]),
                ];
                if let Some(o) = self.open_decision(
                    "storm",
                    None,
                    "태풍 대비",
                    format!("{} — 태풍 · 폭풍이 다가옵니다. 크레인을 어떻게 대비할까요?", self.at(i)),
                    opts,
                    vec![format!("최대 순간풍속 {:.0} m/s 예보", ol.max_gust_mps)],
                    vec![("storm_secure".into(), exp_secure), ("park_at_stop".into(), exp_clamp), ("continue".into(), 1.0)],
                ) {
                    return Some(o);
                }
            }

            // 운전 중 바람 (실측 + 예보)
            if relevant {
                let ol = self.forecaster.outlook(&self.weather, i, p.wind_lookahead_h);
                let g_now = w.gust_mps;
                let g = g_now.max(ol.max_gust_mps);
                let lvl = if g_now >= p.wind_stop_mps {
                    Some(AlertLevel::Stop)
                } else {
                    level_from(g, &[(p.wind_caution_mps, AlertLevel::Caution), (p.wind_warn_mps, AlertLevel::Warning), (p.wind_stop_mps, AlertLevel::Danger)])
                };
                if self.book.update_held("wind", lvl, 2) {
                    let l = lvl.unwrap_or(AlertLevel::Info);
                    let (title, action) = match l {
                        AlertLevel::Stop => ("순간풍속 운전 정지 기준 초과", "운전 정지 · 하중 내려놓기 · 레일 클램프 체결"),
                        AlertLevel::Danger => ("정지 기준 풍속 예보", "작업 마무리 · 계류 준비"),
                        AlertLevel::Warning => ("강풍 경고", "감속 · 흔들림 억제 확인 · 계류 준비"),
                        _ => ("바람 주의", "흔들림 · 풍속계 확인"),
                    };
                    let lead = if g_now >= g - 1e-9 { None } else { Some(ol.hour_of_max) };
                    self.raise(
                        "wind",
                        l,
                        title,
                        format!("순간풍속 지금 {:.1} m/s, {} h 안 최대 {:.1} m/s 예보 (경고 {:.0} · 정지 {:.0} m/s)", g_now, p.wind_lookahead_h, ol.max_gust_mps, p.wind_warn_mps, p.wind_stop_mps),
                        action,
                        "EN 13001-2 운전 풍 상태 (3초 돌풍), 크레인 운전 허용 풍속",
                        g,
                        p.wind_stop_mps,
                        "m/s",
                        lead,
                    );
                    if l == AlertLevel::Stop {
                        let h = self.hours_until_below(i, p.wind_warn_mps).max(1);
                        self.park_now(i + h, "바람");
                        self.usage.clamp_ops += 1.0;
                    } else if l >= AlertLevel::Warning {
                        let hours = self.hours_until_below(i, p.wind_caution_mps).clamp(1, 12) as f64;
                        let cap = self.capacity_per_hour(1.0, g);
                        let e_cont = self.expected_incidents_for(i, 1.0, hours);
                        let e_slow = self.expected_incidents_for(i, self.table.reduced_speed, hours);
                        let rec_park = l == AlertLevel::Danger;
                        let opts = vec![
                            Self::option("reduce_speed", "속도 낮춰 계속", format!("속도 ×{:.1} · 흔들림 억제 유지, 정지 기준이면 자동 정지 — 예상 사고 {:.4}", self.table.reduced_speed, e_slow), !rec_park, true, &[]),
                            Self::option("park", "운전 중지 · 계류", format!("예상 손실 약 {hours:.0} h ≈ {:.0} 이동", hours * cap), rec_park, true, &[]),
                            Self::option("continue", "그대로 계속", format!("예상 사고 {e_cont:.4} (감속 대비 ×{:.1})", e_cont / e_slow.max(1e-12)), false, l < AlertLevel::Danger, if l < AlertLevel::Danger { &[] } else { &["정지 기준 풍속 예보"] }),
                        ];
                        if let Some(o) = self.open_decision(
                            "wind",
                            None,
                            "강풍",
                            format!("{} — 순간풍속 {:.1} m/s ({} h 안 {:.1} m/s 예보). 어떻게 운전할까요?", self.at(i), g_now, p.wind_lookahead_h, ol.max_gust_mps),
                            opts,
                            vec![format!("운전 정지 기준 {:.0} m/s", p.wind_stop_mps)],
                            vec![("reduce_speed".into(), e_slow), ("park".into(), 0.0), ("continue".into(), e_cont)],
                        ) {
                            return Some(o);
                        }
                    }
                }
            }

            // 낙뢰
            let th = w.thunder_km.filter(|d| *d <= p.lightning_km);
            if self.book.update("lightning", th.map(|_| AlertLevel::Stop)) && relevant {
                let d = th.unwrap_or(0.0);
                self.raise(
                    "lightning",
                    AlertLevel::Stop,
                    "낙뢰 근접",
                    format!("{d:.1} km 안 낙뢰 (중지 기준 {:.0} km)", p.lightning_km),
                    "작업 중지 · 하중 내려놓기 · 운전실 · 지상 작업자 대피, 마지막 낙뢰 뒤 30분 대기",
                    "낙뢰 30/30 규칙 · 항만 크레인 운전 기준",
                    d,
                    p.lightning_km,
                    "km",
                    None,
                );
            }
            if th.is_some() {
                let resume = i + 1 + (p.lightning_clear_min / 60.0).ceil() as usize - 1;
                self.park_now(resume + 1, "낙뢰");
            }

            // 시정
            if relevant {
                let v = w.visibility_m;
                let lvl = if v < p.visibility_stop_m {
                    Some(AlertLevel::Stop)
                } else if v < p.visibility_caution_m {
                    Some(AlertLevel::Warning)
                } else {
                    None
                };
                if self.book.update_held("visibility", lvl, 2) {
                    let l = lvl.unwrap_or(AlertLevel::Info);
                    self.raise(
                        "visibility",
                        l,
                        "시정 저하",
                        format!("시정 {v:.0} m (주의 {:.0} · 중지 {:.0} m)", p.visibility_caution_m, p.visibility_stop_m),
                        if l == AlertLevel::Stop { "작업 중지 · 시정 회복 대기" } else { "신호수 배치 · 감속 · 조명" },
                        "안개 · 강수 시정, 신호수 배치 기준",
                        v,
                        p.visibility_caution_m,
                        "m",
                        None,
                    );
                    if l == AlertLevel::Stop {
                        self.park_now(i + 1, "시정");
                    } else {
                        let e_c = self.expected_incidents_for(i, 1.0, 3.0);
                        let e_s = self.expected_incidents_for(i, 0.5, 3.0);
                        let opts = vec![
                            Self::option("signalman_slow", "신호수 · 감속", format!("신호수 배치 · 속도 ×{:.1} — 예상 사고 {:.4}", self.table.reduced_speed, e_s), true, true, &[]),
                            Self::option("stop", "시정 회복까지 중지", "운전 중지 (2 h)".into(), false, true, &[]),
                            Self::option("continue", "그대로 계속", format!("예상 사고 {e_c:.4}"), false, true, &[]),
                        ];
                        if let Some(o) = self.open_decision("visibility", None, "시정 저하", format!("{} — 시정 {v:.0} m. 어떻게 할까요?", self.at(i)), opts, vec!["안개 · 강수로 시야가 좁습니다".into()], vec![("signalman_slow".into(), e_s), ("stop".into(), 0.0), ("continue".into(), e_c)]) {
                            return Some(o);
                        }
                    }
                }
            }

            // 한파 · 결빙
            if relevant {
                let lvl = if w.temp_c <= p.cold_c {
                    Some(AlertLevel::Warning)
                } else if w.icy {
                    Some(AlertLevel::Caution)
                } else {
                    None
                };
                if self.book.update_held("cold", lvl, 3) {
                    let l = lvl.unwrap_or(AlertLevel::Info);
                    let mu = field_constants::adhesion_mu(&w);
                    self.raise(
                        "cold",
                        l,
                        if w.temp_c <= p.cold_c { "한파 (저온 취성 · 윤활)" } else { "레일 결빙" },
                        format!("기온 {:.1} °C, 바퀴–레일 점착 계수 약 {:.2}", w.temp_c, mu),
                        "가감속 완화 · 제빙 · 저온 윤활 확인, 비상 정지 거리 늘어남",
                        "EN 13001-3-1 저온 인성, 바퀴–레일 점착 계수",
                        w.temp_c,
                        p.cold_c,
                        "°C",
                        None,
                    );
                    let e_c = self.expected_incidents_for(i, 1.0, 4.0);
                    let e_s = self.expected_incidents_for(i, self.table.reduced_speed, 4.0);
                    let opts = vec![
                        Self::option("reduce_speed", "감속 · 제빙", format!("4 h 속도 ×{:.1} — 예상 사고 {e_s:.4}", self.table.reduced_speed), true, true, &[]),
                        Self::option("stop", "운전 중지", "2 h 중지 · 레일 제빙".into(), false, true, &[]),
                        Self::option("continue", "그대로 계속", format!("예상 사고 {e_c:.4}"), false, true, &[]),
                    ];
                    if let Some(o) = self.open_decision("cold", None, "결빙 · 한파", format!("{} — 기온 {:.1} °C, 레일 미끄러움. 어떻게 할까요?", self.at(i), w.temp_c), opts, vec![format!("점착 계수 {mu:.2}")], vec![("reduce_speed".into(), e_s), ("stop".into(), 0.0), ("continue".into(), e_c)]) {
                        return Some(o);
                    }
                }
            }
        }

        // 폭염 (지상 작업자 · 냉방 없는 운전실 · 열원 실내)
        if relevant && self.heat_exposed() {
            let stage = p.heat_stage(w.feels_like_c);
            let lvl = match stage {
                0 => None,
                1 => Some(AlertLevel::Info),
                2 => Some(AlertLevel::Caution),
                3 => Some(AlertLevel::Warning),
                _ => Some(AlertLevel::Danger),
            };
            if self.book.update("heat", lvl) {
                let l = lvl.unwrap_or(AlertLevel::Info);
                let rest = p.heat_rest_min(stage);
                self.raise(
                    "heat",
                    l,
                    ["", "폭염 관심", "폭염 주의", "폭염 경고", "폭염 위험"][stage.min(4)],
                    format!("체감온도 {:.1} °C — 단계 {} (기준 {:?} °C)", w.feels_like_c, stage, p.heat_stages_c),
                    "물 · 그늘 · 휴식, 시간당 휴식, 14–17시 옥외 작업 단축",
                    "고용노동부 폭염 대비 근로자 건강보호 가이드, 산업안전보건기준에 관한 규칙",
                    w.feels_like_c,
                    p.heat_stages_c.get(1).cloned().unwrap_or(33.0),
                    "°C",
                    None,
                );
                if stage >= 2 && self.mods.heat_rest_until.map(|u| u <= i).unwrap_or(true) {
                    let midday = stage >= 3;
                    let law = field_constants::HEAT_REST_MANDATORY;
                    let opts = vec![
                        Self::option("rest", "시간당 휴식", format!("매시간 {rest:.0}분 휴식 · 물 · 그늘"), !midday, true, &[]),
                        Self::option("rest_and_midday_stop", "휴식 + 14–17시 중지", "가장 더운 시간 옥외 작업 중지".into(), midday, true, &[]),
                        Self::option("continue", "휴식 없이 계속", "온열 질환 위험".into(), false, !law, if law { &["체감온도 33 °C 이상 휴식 의무"] } else { &[] }),
                    ];
                    if let Some(o) = self.open_decision("heat", None, "폭염", format!("{} — 체감온도 {:.1} °C. 지상 작업자 보호를 어떻게 할까요?", self.at(i), w.feels_like_c), opts, vec![format!("폭염 단계 {stage}")], vec![("rest".into(), 0.0), ("rest_and_midday_stop".into(), 0.0), ("continue".into(), self.expected_incidents_for(i, 1.0, 6.0))]) {
                        return Some(o);
                    }
                }
            }
        }

        // 근무 시작 시각: 중량물 · 정기 점검 · 법정 검사 · 부품 상태
        let start_of_shift = sched && !self.schedule(i.saturating_sub(1)).0 || (i == 0 && sched);
        if start_of_shift {
            if let Some(o) = self.shift_start_checks() {
                return Some(o);
            }
        }
        if !self.queued_changes.is_empty() {
            if let Some(o) = self.component_decisions() {
                return Some(o);
            }
        }
        // 근무 마지막 시간: 남은 물량이 많으면 연장 근무 여부
        let last_hour = sched && !self.schedule(i + 1).0;
        let hourly = self.site.throughput.moves_per_hour;
        let threshold = (2.0 * hourly).max(0.5 * hourly * self.work_hours());
        let asked_recently = self.mods.overtime_asked.map(|d| self.day_num(i) < d + 3).unwrap_or(false);
        if last_hour && self.site.operations.max_overtime_h > 0.0 && self.backlog > threshold && !asked_recently {
            self.mods.overtime_asked = Some(self.day_num(i));
            let h = self.site.operations.max_overtime_h;
            let (_, on_shift, _) = self.schedule(i);
            let fatigue = 1.0 + self.risk.overtime_per_h * (on_shift + h - 8.0).max(0.0);
            self.raise(
                "overtime",
                AlertLevel::Info,
                "남은 물량",
                format!("근무 끝에 {:.0} 이동이 남았습니다 (시간당 수요 {:.0})", self.backlog, hourly),
                "다음 근무로 넘기거나 교대 인력으로 연장",
                "Folkard et al. 2005: 12 h 근무 사고 위험 ×1.27 (8 h 대비)",
                self.backlog,
                threshold,
                "이동",
                None,
            );
            let e_ot = self.risk.base_per_hour * fatigue * h;
            let e_relief = self.risk.base_per_hour * h;
            let opts = vec![
                Self::option("carry", "다음 근무로 넘김", format!("남은 {:.0} 이동을 이월", self.backlog), true, true, &[]),
                Self::option("relief", "교대 인력으로 연장", format!("{h:.0} h 연장, 새 운전원 — 예상 사고 {e_relief:.4}"), false, true, &[]),
                Self::option("overtime", "같은 운전원 연장", format!("{h:.0} h 연장 — 피로 위험 ×{fatigue:.2}, 예상 사고 {e_ot:.4}"), false, on_shift + h <= 12.0, if on_shift + h <= 12.0 { &[] } else { &["하루 12 h 초과 근무"] }),
            ];
            if let Some(o) = self.open_decision("overtime", None, "연장 근무", format!("{} — 오늘 물량 {:.0} 이동이 남았습니다. 어떻게 할까요?", self.at(i), self.backlog), opts, vec![], vec![("carry".into(), 0.0), ("relief".into(), e_relief), ("overtime".into(), e_ot)]) {
                return Some(o);
            }
        }
        None
    }

    fn shift_start_checks(&mut self) -> Option<StepOutcome> {
        let i = self.idx;
        let day = self.abs_day(i);
        let ops = self.site.operations.clone();
        // 법정 안전검사
        let since_legal = day - self.ledger.last_legal_inspection_day;
        let interval = self.legal_interval();
        if ops.legal_inspection_days > 0 && since_legal >= interval - self.policy.inspection_notice_days && !self.tasks.iter().any(|t| t.kind == "legal_inspection") && self.book.level("legal").is_none() {
            let overdue = since_legal >= interval;
            self.book.update("legal", Some(if overdue { AlertLevel::Danger } else { AlertLevel::Warning }));
            self.raise(
                "legal",
                if overdue { AlertLevel::Danger } else { AlertLevel::Warning },
                "법정 안전검사",
                format!("마지막 검사(또는 설치) 뒤 {since_legal:.0} 일 (기한 {interval:.0} 일)"),
                "안전검사 수검 일정 확정",
                "산업안전보건법 시행규칙 제126조: 설치 후 3년 이내 최초, 그 뒤 2년마다 (건설현장 6개월)",
                since_legal,
                interval,
                "일",
                None,
            );
            let opts = vec![
                Self::option("legal_inspection", "안전검사 수검", format!("{:.0} h 운전 중지", field_constants::LEGAL_INSPECTION_H), true, true, &[]),
                Self::option("postpone", "나중에", "검사 기한까지 운전".into(), false, !overdue, if overdue { &["안전검사 기한 지남 — 운전 불가"] } else { &[] }),
            ];
            if let Some(o) = self.open_decision("legal", None, "안전검사", format!("{} — 법정 안전검사 기한이 {}. 언제 받을까요?", self.at(i), if overdue { "지났습니다" } else { "다가옵니다" }), opts, vec![], vec![]) {
                return Some(o);
            }
        }
        // 정기 점검
        let since = day - self.ledger.last_periodic_inspection_day;
        let postponed = self.mods.periodic_postponed_to.map(|d| day < d).unwrap_or(false);
        if ops.periodic_inspection_days > 0 && since >= ops.periodic_inspection_days as f64 && !postponed && !self.tasks.iter().any(|t| t.kind == "periodic_inspection") {
            self.raise(
                "periodic",
                AlertLevel::Caution,
                "정기 점검 기한",
                format!("마지막 정기 점검 뒤 {since:.0} 일 (주기 {} 일)", ops.periodic_inspection_days),
                "브레이크 · 로프 · 훅 · 리밋 · 레일 클램프 점검",
                "ISO 9927-1 · OSHA 1910.179 정기 점검, 자율 점검 프로그램",
                since,
                ops.periodic_inspection_days as f64,
                "일",
                None,
            );
            let opts = vec![
                Self::option("inspect", "비번 시간 정기 점검", format!("{:.0} h, 생산 손실 없음 (비번 시간)", field_constants::PERIODIC_INSPECTION_H), true, true, &[]),
                Self::option("postpone", "1주 연기", "점검 지연 — 위험 배수 증가".into(), false, since < 2.0 * ops.periodic_inspection_days as f64, if since < 2.0 * ops.periodic_inspection_days as f64 { &[] } else { &["점검 주기 두 배 초과"] }),
            ];
            if let Some(o) = self.open_decision("inspection", None, "정기 점검", format!("{} — 정기 점검 기한입니다. 진행할까요?", self.at(i)), opts, vec![], vec![("inspect".into(), 0.0), ("postpone".into(), self.risk.base_per_hour * (self.risk.inspection_overdue - 1.0) * 7.0 * self.work_hours())]) {
                return Some(o);
            }
        }
        // 중량물 작업 (오늘 처음)
        let d = self.day_num(i);
        let crit_share: f64 = self.table.normal.iter().filter(|r| r.fraction >= self.site.loads.critical_fraction).map(|r| r.share).sum();
        if crit_share > 0.0 && !self.lift_plan_valid(d) && self.book.level(&format!("critical_{d}")).is_none() {
            self.book.update(&format!("critical_{d}"), Some(AlertLevel::Info));
            let n = crit_share * self.site.throughput.moves_per_hour * self.work_hours();
            self.raise(
                "critical_lift",
                AlertLevel::Caution,
                "중량물 작업 예정",
                format!("정격 {:.0}% 이상 들기 약 {n:.0} 회 예정", self.site.loads.critical_fraction * 100.0),
                "작업계획서 · 신호수 · 유도 로프 · 출입 통제",
                "산업안전보건기준에 관한 규칙 제38조 (중량물 작업계획서), ASME P30.1 critical lift",
                crit_share,
                0.0,
                "",
                None,
            );
            let e_std = self.risk.base_per_hour * (self.risk.critical_unplanned - 1.0) * crit_share * self.work_hours();
            let e_plan = self.risk.base_per_hour * (self.risk.critical_planned - 1.0) * crit_share * self.work_hours();
            let opts = vec![
                Self::option("lift_plan", "작업계획서 · 신호수", format!("중량물마다 +{:.0} s — 예상 사고 {e_plan:.4}", field_constants::LIFT_PLAN_EXTRA_S), true, true, &[]),
                Self::option("standard", "일반 절차", format!("예상 사고 {e_std:.4}"), false, true, &[]),
            ];
            if let Some(o) = self.open_decision("critical_lift", None, "중량물", format!("{} — 오늘 중량물 들기가 약 {n:.0} 회 있습니다. 어떻게 할까요?", self.at(i)), opts, vec![], vec![("lift_plan".into(), e_plan), ("standard".into(), e_std)]) {
                return Some(o);
            }
        }
        None
    }

    fn component_decisions(&mut self) -> Option<StepOutcome> {
        while let Some(ch) = self.queued_changes.pop() {
            if ch.to <= Health::Watch || self.tasks.iter().any(|t| t.component.as_deref() == Some(ch.id.as_str())) {
                if ch.to == Health::Watch {
                    self.raise(&ch.id, AlertLevel::Info, &format!("{} 관찰", ch.label), format!("수명 사용 {:.0}%", ch.life_used * 100.0), "다음 점검 때 상태 확인", "부품 내구도 원장", ch.life_used, self.model.plan_at, "", None);
                }
                continue;
            }
            let i = self.idx;
            let critical = ch.id.starts_with("rope") || ch.id.starts_with("brake") || ch.id == "hook";
            let over = ch.to >= Health::Replace;
            let lvl = match ch.to {
                Health::Plan => AlertLevel::Caution,
                Health::Replace => AlertLevel::Warning,
                _ => AlertLevel::Danger,
            };
            let rul = ch.rul_days.map(|d| format!(", 남은 약 {d:.0} 일")).unwrap_or_default();
            self.raise(&ch.id, lvl, &format!("{} {}", ch.label, ch.to.label()), format!("수명 사용 {:.0}%{rul}", ch.life_used * 100.0), if over { "교체" } else { "정비 계획 · 부품 준비" }, "부품 내구도 원장 (docs/site-operation.md)", ch.life_used, self.model.replace_at, "", None);
            let h = field_constants::replace_hours(&ch.id);
            let block_defer = over && critical;
            let opts = if over {
                vec![
                    Self::option("replace_now", "지금 교체", format!("{h:.0} h 운전 중지"), critical, true, &[]),
                    Self::option("replace_offshift", "비번 시간 교체", format!("{h:.0} h, 다음 비번 시간"), !critical, ch.to != Health::Overdue || !critical, if ch.to == Health::Overdue && critical { &["교체 기준 초과 — 즉시 교체"] } else { &[] }),
                    Self::option("defer", "미루기", "기준 넘긴 채 운전 — 위험 배수 증가".into(), false, !block_defer, if block_defer { &["안전 핵심 부품 교체 기준 도달"] } else { &[] }),
                ]
            } else {
                vec![
                    Self::option("schedule", "비번 시간 교체 예약", format!("{h:.0} h, 생산 손실 없음"), true, true, &[]),
                    Self::option("inspect_now", "지금 점검", "2 h 점검".into(), false, true, &[]),
                    Self::option("defer", "다음에", "교체 기준 때 다시 알림".into(), false, true, &[]),
                ]
            };
            let comp = ch.id.clone();
            if let Some(o) = self.open_decision("component", Some(&comp), if over { "부품 교체" } else { "정비 계획" }, format!("{} — {} 수명 사용 {:.0}%{rul}. 어떻게 할까요?", self.at(i), ch.label, ch.life_used * 100.0), opts, vec![ch.to.label().to_string()], vec![]) {
                return Some(o);
            }
        }
        None
    }

    // ---------- 안정성 ----------

    /// 바람 일주(밀림) 여유: (제동 점착 + 클램프 + 계류) / 풍하중
    pub fn runaway_margin(&self, gust_mps: f64, tie_down: bool, clamps: bool) -> f64 {
        let st = &self.spec.structure;
        let sp = &self.spec.stability;
        let rho = site::air_density(self.site.environment.altitude_m, 15.0);
        let f_wind = 0.5 * rho * st.bridge_cd_area_m2 * gust_mps * gust_mps;
        if f_wind <= 1e-6 {
            return 99.0;
        }
        let w = (st.bridge_mass_kg + st.trolley_mass_kg + st.hook_mass_kg) * G;
        let mu = field_constants::ADHESION_RUNAWAY;
        let mut r = mu * sp.braked_wheel_fraction * w;
        if clamps && self.site.operations.rail_clamps {
            r += sp.rail_clamp_kn * 1000.0;
        }
        if tie_down {
            r += sp.tie_down_kn * 1000.0;
        }
        (r / f_wind).min(99.0)
    }

    /// 운전 중 바람 저항 여유: 구동 · 제동 바퀴 점착(그 시간 날씨의 μ) / 풍하중
    fn in_service_hold_margin(&self, w: &HourWeather) -> f64 {
        let st = &self.spec.structure;
        let rho = site::air_density(self.site.environment.altitude_m, w.temp_c);
        let f_wind = 0.5 * rho * st.bridge_cd_area_m2 * w.gust_mps * w.gust_mps;
        if f_wind <= 1e-6 {
            return 99.0;
        }
        let weight = (st.bridge_mass_kg + st.trolley_mass_kg + st.hook_mass_kg) * G;
        (field_constants::adhesion_mu(w) * self.spec.stability.braked_wheel_fraction * weight / f_wind).min(99.0)
    }

    /// 지금부터 `until` 까지 운전 중지 · 계류
    fn park_now(&mut self, until: usize, reason: &str) {
        let i = self.idx;
        // 같은 이유의 겹치는 구간은 늘리고, 아니면 새 구간
        if let Some(p) = self.mods.parks.iter_mut().find(|(f, u, r)| *f <= i && i < *u && r == reason) {
            p.1 = p.1.max(until);
        } else {
            self.mods.parks.push((i, until, reason.to_string()));
        }
        self.mods.parks.retain(|(_, u, _)| *u > i);
    }

    fn park_reason(&self, i: usize) -> Option<String> {
        self.mods.parks.iter().find(|(f, u, _)| *f <= i && i < *u).map(|(_, _, r)| r.clone())
    }

    fn is_parked(&self, i: usize) -> bool {
        self.park_reason(i).is_some()
    }

    fn lift_plan_valid(&self, day: u32) -> bool {
        self.mods.lift_plan_until.map(|u| day <= u).unwrap_or(false)
    }

    fn tied_down(&self, i: usize) -> bool {
        self.mods.ties.iter().any(|(f, u)| *f <= i && i < *u)
    }

    /// 전도 여유: 자중 안정 모멘트 / 풍압 전도 모멘트
    pub fn overturn_margin(&self, gust_mps: f64) -> f64 {
        let st = &self.spec.structure;
        let sp = &self.spec.stability;
        let rho = site::air_density(self.site.environment.altitude_m, 15.0);
        let f_wind = 0.5 * rho * st.bridge_cd_area_m2 * gust_mps * gust_mps;
        if f_wind <= 1e-6 || sp.wheelbase_m <= 0.0 {
            return 99.0;
        }
        let w = (st.bridge_mass_kg + st.trolley_mass_kg + st.hook_mass_kg) * G;
        (w * 0.5 * sp.wheelbase_m / (f_wind * sp.wind_center_height_m.max(1.0))).min(99.0)
    }

    fn adhesion_margin(&self, w: &HourWeather, k: usize, speed_scale: f64) -> f64 {
        let st = &self.spec.structure;
        let sp = &self.spec.stability;
        let mu = field_constants::adhesion_mu(w);
        let d = if k == 0 { &self.spec.travel } else { &self.spec.trolley };
        let mass = if k == 0 { st.bridge_mass_kg + st.trolley_mass_kg + st.hook_mass_kg + 0.6 * st.rated_load_kg } else { st.trolley_mass_kg + st.hook_mass_kg + 0.6 * st.rated_load_kg };
        let a = d.accel_mps2 * speed_scale.clamp(0.3, 1.0).sqrt();
        let rho = site::air_density(self.site.environment.altitude_m, w.temp_c);
        let wind = if k == 0 { 0.5 * rho * st.bridge_cd_area_m2 * w.gust_mps * w.gust_mps } else { 0.0 };
        let need = mass * a + wind + st.rolling_resistance * mass * G;
        let avail = mu * sp.braked_wheel_fraction * mass * G;
        (avail / need.max(1.0)).min(99.0)
    }

    // ---------- 처리량 · 위험 ----------

    fn speed_scale(&self, i: usize) -> f64 {
        match self.mods.slow_until {
            Some(u) if i < u => self.mods.slow_scale,
            _ => 1.0,
        }
    }

    fn move_time_s(&self, speed: f64, gust: f64, i: usize) -> f64 {
        let reduced = speed < 0.999;
        let mut t = self.table.mean_cycle_s(reduced) + self.site.throughput.handling_overhead_s;
        if self.outdoor() {
            t += self.table.wind_penalty_s(gust, speed);
        }
        if self.lift_plan_valid(self.day_num(i)) {
            let crit: f64 = self.table.normal.iter().filter(|r| r.fraction >= self.site.loads.critical_fraction).map(|r| r.share).sum();
            t += crit * field_constants::LIFT_PLAN_EXTRA_S;
        }
        t
    }

    fn capacity_per_hour(&self, speed: f64, gust: f64) -> f64 {
        3600.0 / self.move_time_s(speed, gust, self.idx).max(1.0)
    }

    fn risk_inputs(&self, i: usize, speed: f64) -> RiskInputs {
        let w = &self.weather[i.min(self.weather.len() - 1)];
        let (_, on_shift, _) = self.schedule(i);
        let ops = &self.site.operations;
        let stage = if self.heat_exposed() { self.policy.heat_stage(w.feels_like_c) } else { 0 };
        let heat_rest = self.mods.heat_rest_until.map(|u| i < u).unwrap_or(false);
        let crit: f64 = self.table.normal.iter().filter(|r| r.fraction >= self.site.loads.critical_fraction).map(|r| r.share).sum();
        let day = self.abs_day(i);
        let overdue = |pred: &dyn Fn(&str) -> bool| self.ledger.components.iter().any(|c| pred(&c.id) && c.status == Health::Overdue);
        RiskInputs {
            gust_mps: if self.outdoor() { w.gust_mps } else { 0.0 },
            wind_stop_mps: if self.outdoor() { self.policy.wind_stop_mps } else { 0.0 },
            wet: self.outdoor() && w.rain_mm > 0.2,
            icy: self.outdoor() && w.icy,
            visibility_m: w.visibility_m,
            thunder_km: if self.outdoor() { w.thunder_km } else { None },
            lightning_km: self.policy.lightning_km,
            heat_stage: stage,
            heat_rest,
            cold: self.outdoor() && w.temp_c <= self.policy.cold_c,
            night: NIGHT.contains(&((i % 24) as u8)),
            afternoon: (14..22).contains(&(i % 24)),
            lighting_lux: ops.lighting_lux,
            lighting_min_lux: self.policy.lighting_min_lux,
            hours_on_shift: on_shift,
            operator: ops.operator_experience.clone(),
            signalman: ops.signalman,
            exclusion_zone: ops.exclusion_zone,
            maintenance: ops.maintenance_quality.clone(),
            critical_share: crit,
            critical_planned: self.lift_plan_valid(self.day_num(i)),
            rope_overdue: overdue(&|id| id.starts_with("rope")),
            brake_overdue: overdue(&|id| id.starts_with("brake")),
            other_overdue: overdue(&|id| !id.starts_with("rope") && !id.starts_with("brake") && id != "coating"),
            inspection_overdue: ops.periodic_inspection_days > 0 && day - self.ledger.last_periodic_inspection_day > ops.periodic_inspection_days as f64 + 7.0,
            legal_overdue: ops.legal_inspection_days > 0 && day - self.ledger.last_legal_inspection_day > self.legal_interval(),
            speed_scale: speed,
        }
    }

    fn current_risk(&self, i: usize) -> crate::alerts::RiskResult {
        self.risk.evaluate(&self.risk_inputs(i, self.speed_scale(i)))
    }

    /// 앞으로 `hours` 시간 운전했을 때의 기대 사고 수 (속도 배율 `speed`)
    fn expected_incidents_for(&self, from: usize, speed: f64, hours: f64) -> f64 {
        let n = hours.ceil() as usize;
        (0..n).map(|k| self.risk.evaluate(&self.risk_inputs((from + k).min(self.weather.len() - 1), speed)).per_hour).sum::<f64>() * (hours / n.max(1) as f64)
    }

    // ---------- 하루 시작 · 끝 ----------

    fn start_day(&mut self) {
        let i = self.idx;
        let doy = self.weather[i].doy;
        let cv = self.site.throughput.demand_cv.max(0.0);
        let sd = (1.0 + cv * cv).ln().sqrt();
        self.demand_factor = (sd * self.rng_demand.normal() - 0.5 * sd * sd).exp();
        self.day = DayRow { day: self.day_num(i), date: weather::date_label(doy), temp_min_c: f64::MAX, temp_max_c: f64::MIN, feels_like_max_c: f64::MIN, min_visibility_m: f64::MAX, ..Default::default() };
        self.usage = DayUsage::default();
    }

    fn end_day(&mut self) {
        let i = self.idx;
        let abs_day = self.abs_day(i) + 1.0;
        let usage = std::mem::take(&mut self.usage);
        let changes = self.ledger.apply_day(&self.spec, &self.site, &self.model, &usage, abs_day);
        self.queued_changes.extend(changes);
        // 남은 수명이 짧은 순서로 맨 뒤에 (pop 이 먼저 꺼낸다)
        self.queued_changes.sort_by(|a, b| b.rul_days.unwrap_or(f64::MAX).partial_cmp(&a.rul_days.unwrap_or(f64::MAX)).unwrap_or(std::cmp::Ordering::Equal));
        let worst = self.ledger.components.iter().max_by(|a, b| a.life_used.partial_cmp(&b.life_used).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(c) = worst {
            self.day.worst_component = c.label.clone();
            self.day.worst_life = c.life_used;
        }
        self.day.backlog = self.backlog;
        let doy = self.weather[i].doy;
        self.ledger.snapshot(doy, self.day.risk_max, self.day.alerts, self.day.downtime_h, self.day.moves, self.day.op_h);
        self.ledger.next_doy = (doy + 1) % 365;
        self.daily.push(std::mem::take(&mut self.day));
    }

    // ---------- 정비 작업 ----------

    /// 이 시간에 정비 작업이 운전을 막는가 (막으면 이유)
    fn work_tasks(&mut self, scheduled: bool) -> Option<String> {
        let i = self.idx;
        let day = self.abs_day(i);
        let hour = (i % 24) as u8;
        let mut blocking = None;
        let mut done = Vec::new();
        for (k, t) in self.tasks.iter_mut().enumerate() {
            if let (Some(min), Some(c)) = (t.min_life, t.component.as_deref()) {
                let life = self.ledger.components.iter().find(|x| x.id == c).map(|x| x.life_used).unwrap_or(1.0);
                if life < min && !t.started {
                    continue;
                }
            }
            let can = if t.off_shift { !scheduled || t.started } else { true };
            // 비번 시간이 없는 24시간 현장은 근무 시간에라도 한다
            let no_off = self.site.throughput.shifts_per_day as f64 * self.site.throughput.shift_hours >= 24.0 - 1e-9;
            if can || (t.off_shift && no_off) {
                t.started = true;
                t.hours_left -= 1.0;
                if scheduled {
                    blocking = Some(match t.kind.as_str() {
                        "replace" => "정비 · 교체".to_string(),
                        "legal_inspection" => "법정 검사".to_string(),
                        "investigate" => "사고 조사".to_string(),
                        "post_storm_inspection" => "강풍 뒤 점검".to_string(),
                        _ => "점검".to_string(),
                    });
                }
                if t.hours_left <= 1e-9 {
                    done.push(k);
                }
                break; // 한 번에 한 작업
            }
        }
        for k in done.into_iter().rev() {
            let t = self.tasks.remove(k);
            let comp = t.component.clone().unwrap_or_default();
            match t.kind.as_str() {
                "replace" => {
                    let _ = self.ledger.replace(&comp, &self.spec, day, hour, &t.reason);
                    self.kpi.replacements += 1;
                    self.book.update(&comp, None);
                }
                "inspect" => {
                    let _ = self.ledger.inspect(&comp, day, hour, &t.reason);
                }
                "periodic_inspection" => {
                    self.ledger.last_periodic_inspection_day = day;
                    self.mods.periodic_postponed_to = None;
                    self.ledger.note(day, hour, "periodic_inspection", None, "정기 점검 완료".into());
                    for c in self.ledger.components.iter_mut() {
                        c.last_inspection_day = Some(day);
                    }
                }
                "legal_inspection" => {
                    self.ledger.last_legal_inspection_day = day;
                    self.ledger.note(day, hour, "legal_inspection", None, "법정 안전검사 합격".into());
                    self.book.update("legal", None);
                }
                "investigate" => {
                    self.ledger.note(day, hour, "inspect", t.component.as_deref(), "사고 조사 · 재가동 점검 완료".into());
                }
                "post_storm_inspection" => {
                    self.ledger.note(day, hour, "inspect", Some("girder"), "강풍 뒤 점검 완료 (제143조)".into());
                }
                _ => {}
            }
        }
        blocking
    }

    // ---------- 한 시간 ----------

    fn add_down(&mut self, reason: &str, h: f64) {
        if h <= 0.0 {
            return;
        }
        self.day.downtime_h += h;
        if let Some(x) = self.downtime.iter_mut().find(|(r, _)| r == reason) {
            x.1 += h;
        } else {
            self.downtime.push((reason.to_string(), h));
        }
    }

    fn hour_step(&mut self) {
        let i = self.idx;
        let w = self.weather[i].clone();
        let (sched, _, _) = self.schedule(i);
        // 날 요약
        self.day.max_gust_mps = self.day.max_gust_mps.max(w.gust_mps);
        self.day.temp_max_c = self.day.temp_max_c.max(w.temp_c);
        self.day.temp_min_c = self.day.temp_min_c.min(w.temp_c);
        self.day.feels_like_max_c = self.day.feels_like_max_c.max(w.feels_like_c);
        self.day.rain_mm += w.rain_mm;
        self.day.thunder |= w.thunder_km.is_some();
        self.day.min_visibility_m = self.day.min_visibility_m.min(w.visibility_m);

        // 정지 상태 안정성 (폭풍 일주는 근무와 무관)
        if self.outdoor() {
            let parked = !sched || self.is_parked(i);
            let tie = self.tied_down(i);
            let m_run = if parked { self.runaway_margin(w.gust_mps, tie, true) } else { self.in_service_hold_margin(&w) };
            let m_ovt = self.overturn_margin(w.gust_mps);
            let mut mg = Margins { runaway: Some(m_run), overturn: Some(m_ovt), ..Default::default() };
            if sched {
                for k in 0..2 {
                    mg.adhesion[k] = Some(self.adhesion_margin(&w, k, self.speed_scale(i)));
                }
            }
            self.usage.margins.merge(&mg);
            if m_run < 1.0 && self.mods.runaway_until.map(|u| i >= u).unwrap_or(true) {
                self.mods.runaway_until = Some(i + 48);
                self.storm_runaway(&w, m_run, tie);
            }
            // 순간풍속 30 m/s 를 넘긴 뒤에는 점검하고 재가동 (산업안전보건기준에 관한 규칙 제143조)
            if w.gust_mps / self.hfac >= self.policy.post_storm_inspect_mps && !self.tasks.iter().any(|t| t.kind == "post_storm_inspection") && self.book.update("post_storm", Some(AlertLevel::Stop)) {
                self.raise(
                    "post_storm",
                    AlertLevel::Stop,
                    "강풍 뒤 점검 필요",
                    format!("순간풍속 (10 m 환산) {:.1} m/s ≥ {:.0} m/s — 재가동 전에 구조 · 레일 클램프 · 브레이크 · 전기 점검", w.gust_mps / self.hfac, self.policy.post_storm_inspect_mps),
                    "작업 재개 전 점검 · 기록",
                    "산업안전보건기준에 관한 규칙 제143조 (폭풍 등으로 인한 이상 유무 점검)",
                    w.gust_mps,
                    self.policy.post_storm_inspect_mps,
                    "m/s",
                    None,
                );
                self.push_task("post_storm_inspection", Some("girder"), field_constants::POST_STORM_INSPECTION_H, false, "순간풍속 30 m/s 초과 뒤 점검");
            } else if w.gust_mps < self.policy.wind_warn_mps && self.book.level("post_storm").is_some() && !self.tasks.iter().any(|t| t.kind == "post_storm_inspection") {
                self.book.update("post_storm", None);
            }
        }

        let blocking = self.work_tasks(sched);
        if !sched {
            return;
        }
        self.day.scheduled_h += 1.0;
        self.kpi.scheduled_h += 1.0;
        let demand = self.site.throughput.moves_per_hour * self.demand_factor;
        self.day.demand += demand;
        self.kpi.demand_moves += demand;
        let parked = self.is_parked(i);
        let gust_stop = self.outdoor() && w.gust_mps >= self.policy.wind_stop_mps;
        let legal_overdue = self.site.operations.legal_inspection_days > 0 && self.abs_day(i) - self.ledger.last_legal_inspection_day > self.legal_interval() + 30.0;
        let reason: Option<String> = if let Some(b) = blocking {
            Some(b)
        } else if gust_stop {
            Some("바람".into())
        } else if parked {
            self.park_reason(i)
        } else if legal_overdue {
            Some("법정 검사".into())
        } else {
            None
        };
        if let Some(r) = reason {
            self.add_down(&r, 1.0);
            self.backlog += demand;
            return;
        }
        // 운전 분
        let mut minutes = 60.0 - self.site.throughput.break_min_per_shift / self.site.throughput.shift_hours.max(1.0);
        let heat_rest = self.mods.heat_rest_until.map(|u| i < u).unwrap_or(false);
        if heat_rest && self.heat_exposed() {
            let stage = self.policy.heat_stage(w.feels_like_c);
            let rest = self.policy.heat_rest_min(stage);
            let h = (i % 24) as u32;
            if (self.mods.heat_midday_stop && stage >= 3 && (14..17).contains(&h)) || stage >= 4 {
                self.add_down("폭염 휴식", minutes / 60.0);
                self.backlog += demand;
                return;
            }
            minutes -= rest;
            self.add_down("폭염 휴식", rest / 60.0);
        }
        let speed = self.speed_scale(i);
        let cap = minutes * 60.0 / self.move_time_s(speed, if self.outdoor() { w.gust_mps } else { 0.0 }, i).max(1.0);
        let want = demand + self.backlog.min(self.site.throughput.moves_per_hour * self.work_hours());
        let mut moves = want.min(cap).max(0.0);
        // 정격 초과 (신고 무게 오류) — 제한 장치가 막는다
        let over = moves * self.site.loads.overweight_share.max(0.0);
        if over > 0.0 {
            moves -= over;
            self.kpi.refused_overweight += over;
            if self.book.update("overweight", Some(AlertLevel::Warning)) {
                self.raise(
                    "overweight",
                    AlertLevel::Warning,
                    "정격 초과 하중 차단",
                    format!("신고 무게와 달리 정격을 넘는 하중이 {:.1}% 섞여 있습니다", self.site.loads.overweight_share * 100.0),
                    "무게 재확인 · 들기 거부 (정격 하중 제한 장치)",
                    "EN 15011 · EN 12077-2 정격 하중 제한 장치",
                    self.site.loads.overweight_share,
                    0.0,
                    "",
                    None,
                );
            }
        }
        self.backlog = (want - moves - over).max(0.0);
        let op_h = minutes / 60.0;
        self.day.op_h += op_h;
        self.kpi.operating_h += op_h;
        self.day.moves += moves;
        self.kpi.moves += moves;
        self.accumulate(moves, speed, op_h, &w);
        // 위험
        let r = self.risk.evaluate(&self.risk_inputs(i, speed));
        let expected = r.per_hour * op_h;
        self.day.expected_incidents += expected;
        self.kpi.expected_incidents += expected;
        self.day.risk_max = self.day.risk_max.max(r.index);
        self.kpi.risk_max = self.kpi.risk_max.max(r.index);
        self.risk_sum += r.index;
        self.risk_n += 1.0;
        if self.cfg.incidents && self.rng_incident.chance(1.0 - (-expected).exp()) {
            self.incident(&r, &w);
        }
    }

    fn accumulate(&mut self, moves: f64, speed: f64, op_h: f64, w: &HourWeather) {
        if moves <= 0.0 {
            return;
        }
        let reduced = speed < 0.999;
        let rows = self.table.rows(reduced).clone();
        let st = &self.spec.structure;
        let m_tot_rated = st.rated_load_kg + st.hook_mass_kg;
        let kp_d = self.model.girder_design_kp.max(0.01);
        let u3 = self.model.girder_utilization.powi(3);
        let c_u = self.spec.class.design_cycles.max(1.0);
        let mut active = [0.0f64; 3];
        let mut cu = [0.0f64; 3];
        let mut ot = [0.0f64; 3];
        let mut cool = [0.0f64; 3];
        let mut ripple = [0.0f64; 3];
        for r in rows.iter() {
            let n = moves * r.share;
            if n <= 0.0 {
                continue;
            }
            self.usage.moves += n;
            self.usage.rope_damage += n * r.rope_damage;
            for k in 0..3 {
                self.usage.brake_sets[k] += n * r.brake_sets[k];
                self.usage.brake_dyn_wear_mm[k] += n * r.brake_dyn_wear_mm[k];
                self.usage.flh_h[k] += n * r.flh_s[k] / 3600.0;
                self.usage.bearing_h[k] += n * r.run_s[k] * r.speed_frac[k] / 3600.0;
                active[k] += n * r.cycle_time_s;
                cu[k] += n * r.copper_w[k] * r.cycle_time_s;
                ot[k] += n * r.other_w[k] * r.cycle_time_s;
                cool[k] += n * r.cooling[k] * r.cycle_time_s;
                ripple[k] = ripple[k].max(r.ripple_k[k]);
            }
            self.usage.wheel_km_eq[0] += n * r.travel_km_eq;
            self.usage.wheel_km_eq[1] += n * r.trolley_km_eq;
            let ratio = (r.load_kg + st.hook_mass_kg) / m_tot_rated.max(1.0);
            self.usage.girder_damage += n * u3 * ratio.powi(3) / (c_u * kp_d);
            self.usage.hook_cycles_eq += n * (r.load_kg / st.rated_load_kg.max(1.0)).powi(3);
            let e = n * r.energy_kwh;
            self.kpi.energy_kwh += e;
            self.kpi.regen_kwh += n * r.regen_kwh;
            self.day.energy_kwh += e;
            // 요구 안전율 = 제원 Zp 와 법정 최소(제163조 5 이상) 중 큰 값
            let sf = r.rope_sf / self.spec.rope.min_safety_factor.max(self.policy.rope_sf_legal).max(1e-9);
            self.usage.margins.rope_sf = Some(self.usage.margins.rope_sf.map(|x| x.min(sf)).unwrap_or(sf));
            let bf = r.hoist_brake_factor / self.spec.limits.hoist_brake_factor_min.max(1e-9);
            self.usage.margins.hoist_brake = Some(self.usage.margins.hoist_brake.map(|x| x.min(bf)).unwrap_or(bf));
        }
        self.usage.op_hours += op_h;
        // 모터 열: 이 시간 평균 손실의 준정상 상태 + 사이클 맥동 → 절연 열화 (Arrhenius 10 K 규칙)
        let alt = self.site.environment.altitude_m;
        let alt_k = field_constants::altitude_rise_factor(alt);
        for a in Axis::all() {
            let k = a.index();
            let d = self.spec.drive(a);
            let net = d.motor.thermal_network();
            let span = 3600.0;
            let idle = (span - active[k]).max(0.0);
            let kbar = ((cool[k] + idle * d.motor.cooling_factor(0.0)) / span).max(0.05);
            let p_cu = cu[k] / span;
            let p_o = ot[k] / span;
            let rise = ((p_cu + p_o) * net.r_fa / kbar + p_cu * net.r_wf + ripple[k]) * alt_k;
            let t_hot = w.temp_c + rise;
            let t_class = d.motor.insulation_limit_c();
            self.usage.insulation_aging_h[k] += 2f64.powf((t_hot - t_class) / self.model.insulation_halving_k.max(1.0));
            // 허용 권선 온도: 제조사 출력 감소표에 맞춘 값 (fieldcal) — 절연 등급보다 낮은 설계 온도 상승 기준
            let allow = (field_constants::MOTOR_ALLOW_WINDING_C.min(t_class - self.spec.limits.winding_margin_k) - w.temp_c).max(0.0);
            let m = if rise > 1e-6 { (allow / rise).min(99.0) } else { 99.0 };
            self.usage.margins.motor_thermal[k] = Some(self.usage.margins.motor_thermal[k].map(|x| x.min(m)).unwrap_or(m));
        }
    }

    fn incident(&mut self, r: &crate::alerts::RiskResult, w: &HourWeather) {
        let i = self.idx;
        let top = r.factors.first().map(|(n, _)| n.clone()).unwrap_or_default();
        let (kind, comp, damage): (&str, Option<&str>, f64) = match top.as_str() {
            "바람" => ("바람에 의한 하중 흔들림 · 충돌", Some("girder"), 0.01),
            "결빙 레일" | "젖은 레일" => ("제동 거리 증가 · 완충기 충돌", Some("wheel_travel"), 0.03),
            "짙은 안개" | "시정 저하" | "야간" | "조도 부족" => ("시야 불량 충돌", Some("girder"), 0.01),
            "장시간 근무" | "초보 운전원" => ("운전 실수 (충돌 · 끼임 위험)", Some("hook"), 0.02),
            "신호수 없음" | "출입 통제 없음" => ("작업 반경 접근 (아차사고)", None, 0.0),
            "로프 교체 기준 초과" => ("로프 손상 · 하중 흔들림", Some("rope_hoist"), 0.1),
            "브레이크 교체 기준 초과" => ("제동 미끄럼 · 하중 처짐", Some("brake_hoist"), 0.1),
            "중량물 (계획 없음)" | "중량물 (계획)" => ("중량물 줄걸이 이탈 · 하중 흔들림", Some("hook"), 0.03),
            "낙뢰 근접" => ("낙뢰 피해 (전기 · 제어)", Some("motor_hoist"), 0.02),
            "폭염 (휴식 없음)" => ("온열 질환 위험 (지상 작업자)", None, 0.0),
            _ => ("운전 사고", None, 0.0),
        };
        // 심각도: Bird(1969) 1 중대 : 10 경상 : 30 재산 피해 : 600 아차사고
        let total: f64 = field_constants::SEVERITY_SHARES.iter().map(|(_, w)| w).sum();
        let mut u = self.rng_incident.uniform() * total;
        let mut severity = "아차사고";
        for (name, w) in field_constants::SEVERITY_SHARES.iter() {
            if u < *w {
                severity = name;
                break;
            }
            u -= w;
        }
        let downtime = match severity {
            "아차사고" => 1.0,
            "재산 피해" => 8.0 + 16.0 * self.rng_incident.uniform(),
            "경상" => 8.0 + 16.0 * self.rng_incident.uniform(),
            _ => 72.0 + 96.0 * self.rng_incident.uniform(),
        };
        if let (Some(c), true) = (comp, severity == "재산 피해" || severity == "중대") {
            if let Some(x) = self.ledger.component_mut(c) {
                x.life_used += damage * if severity == "중대" { 5.0 } else { 1.0 };
            }
        }
        let day = self.abs_day(i);
        let detail = format!("{} — 주 요인 {}, 위험 배수 ×{:.1}, 순간풍속 {:.1} m/s", kind, if top.is_empty() { "기준 위험".into() } else { top.clone() }, r.multiplier, w.gust_mps);
        self.ledger.incidents += 1;
        self.ledger.note(day, (i % 24) as u8, "incident", comp, format!("[{severity}] {detail}"));
        self.incidents.push(Incident { day: self.day_num(i), hour: (i % 24) as u8, at: self.at(i), kind: kind.into(), severity: severity.into(), causes: r.factors.clone(), downtime_h: downtime, detail: detail.clone() });
        self.kpi.incidents += 1;
        self.day.incidents += 1;
        self.raise("incident", AlertLevel::Danger, &format!("사고 발생: {kind}"), format!("[{severity}] {detail}"), "작업 중지 · 사고 조사 · 관련 부품 점검", "위험 모델 (docs/site-operation.md)", r.multiplier, 1.0, "×", None);
        self.push_task("investigate", comp, downtime.max(field_constants::INVESTIGATION_H), false, &format!("{kind} 조사"));
    }

    fn storm_runaway(&mut self, w: &HourWeather, margin: f64, tie: bool) {
        let i = self.idx;
        let day = self.abs_day(i);
        let detail = format!("순간풍속 {:.1} m/s 에서 일주 여유 {:.2} — 제동 · 클램프{} 저항을 넘어 크레인이 밀렸습니다", w.gust_mps, margin, if tie { " · 계류" } else { "" });
        for (c, dmg) in [("girder", 0.25), ("wheel_travel", 0.3), ("rail_clamp", 1.0)] {
            if let Some(x) = self.ledger.component_mut(c) {
                x.life_used += dmg;
            }
        }
        self.ledger.incidents += 1;
        self.ledger.note(day, (i % 24) as u8, "incident", Some("rail_clamp"), format!("[중대] 폭풍 일주 · 탈선 위험 — {detail}"));
        self.incidents.push(Incident { day: self.day_num(i), hour: (i % 24) as u8, at: self.at(i), kind: "폭풍 일주 (밀림) · 탈선".into(), severity: "중대".into(), causes: vec![("바람".into(), 1.0 / margin.max(1e-3))], downtime_h: 168.0, detail: detail.clone() });
        self.kpi.incidents += 1;
        self.day.incidents += 1;
        self.raise("runaway", AlertLevel::Stop, "폭풍 일주 (밀림)", detail, "구조 · 차륜 · 레일 · 클램프 정밀 점검 후 재가동", "EN 13001-2 비운전 폭풍 · 일주 방지 장치", margin, 1.0, "×", None);
        self.push_task("investigate", Some("rail_clamp"), 168.0, false, "폭풍 일주 복구");
    }

    /// 다음 결정이 생기거나 끝날 때까지 (최대 `max_hours` 시간) 진행한다
    pub fn run(&mut self, max_hours: usize) -> StepOutcome {
        if let Some(p) = &self.pending {
            return StepOutcome::Decision { decision: Box::new(p.clone()) };
        }
        let end = self.total_hours();
        let stop_at = (self.idx + max_hours.max(1)).min(end);
        while self.idx < stop_at {
            if let Some(o) = self.check_alerts() {
                return o;
            }
            self.hour_step();
            if self.idx % 24 == 23 {
                self.end_day();
            }
            self.idx += 1;
            if self.idx < end && self.idx % 24 == 0 {
                self.start_day();
            }
        }
        if self.idx >= end {
            if !self.finished {
                self.finished = true;
                self.finish_kpis();
            }
            StepOutcome::Finished
        } else {
            StepOutcome::Paused { hour_index: self.idx }
        }
    }

    fn finish_kpis(&mut self) {
        let k = &mut self.kpi;
        k.days = self.cfg.days;
        k.availability = if k.scheduled_h > 0.0 { (k.operating_h / k.scheduled_h).min(1.0) } else { 0.0 };
        k.moves_per_op_hour = if k.operating_h > 0.0 { k.moves / k.operating_h } else { 0.0 };
        k.backlog_moves = self.backlog;
        k.risk_mean = if self.risk_n > 0.0 { self.risk_sum / self.risk_n } else { 0.0 };
        let mut d = self.downtime.clone();
        d.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        k.downtime_by_reason = d;
        k.maintenance_h = k.downtime_by_reason.iter().filter(|(r, _)| r.contains("정비") || r.contains("점검") || r.contains("검사")).map(|(_, h)| h).sum();
        let mut levels: Vec<(String, u32)> = Vec::new();
        for a in self.alerts.iter() {
            let l = a.level.label().to_string();
            if let Some(x) = levels.iter_mut().find(|(n, _)| *n == l) {
                x.1 += 1;
            } else {
                levels.push((l, 1));
            }
        }
        k.alerts_by_level = levels;
    }

    pub fn ledger(&self) -> &HealthLedger {
        &self.ledger
    }

    pub fn report(&self) -> OperationReport {
        let mut kpi = self.kpi.clone();
        if !self.finished {
            kpi.days = (self.idx / 24) as u32;
            kpi.availability = if kpi.scheduled_h > 0.0 { kpi.operating_h / kpi.scheduled_h } else { 0.0 };
            kpi.moves_per_op_hour = if kpi.operating_h > 0.0 { kpi.moves / kpi.operating_h } else { 0.0 };
            kpi.downtime_by_reason = self.downtime.clone();
        }
        let n_days = (self.idx / 24).max(1);
        let months = weather::monthly_stats(&self.weather[..(n_days * 24).min(self.weather.len())], if self.site.environment.indoor { 10.0 } else { self.site.environment.crane_height_m }, self.site.climate.resolve().map(|c| c.alpha).unwrap_or(0.14));
        let mut notes = self.notes.clone();
        notes.push("사고는 위험 모델의 시간당 사고율에서 시드로 뽑은 확률 결과입니다 (예측이 아니라 위험 노출의 크기)".into());
        if self.site.environment.indoor {
            notes.push("실내 현장: 바람 · 강수 · 낙뢰 · 시정 알림은 쓰지 않습니다".into());
        }
        OperationReport {
            site: self.site.id.clone(),
            site_name: self.site.name.clone(),
            asset_id: self.ledger.asset_id.clone(),
            crane: self.spec.name.clone(),
            start_date: weather::date_label(self.start_doy),
            policy: self.cfg.policy.key().into(),
            seed: self.cfg.seed,
            kpi,
            daily: self.daily.clone(),
            alerts: self.alerts.clone(),
            decisions: self.decisions.clone(),
            incidents: self.incidents.clone(),
            durability: health::evaluate(&self.ledger),
            ledger: self.ledger.clone(),
            table: self.table.clone(),
            weather_months: months.into_iter().filter(|m| m.days > 0.0).collect(),
            finished: self.finished,
            notes,
        }
    }
}

/// 사람 없이 끝까지 돌린다 (`Ask` 정책이면 권장으로 대신 고른다)
pub fn run_to_end(sim: &mut OpSim) -> OperationReport {
    loop {
        match sim.run(usize::MAX / 4) {
            StepOutcome::Finished => break,
            StepOutcome::Decision { decision } => {
                let key = decision.prompt.recommended.clone();
                if sim.decide(&decision.id, &key).is_err() {
                    let k = decision.prompt.options.iter().find(|o| o.enabled).map(|o| o.key.clone()).unwrap_or_default();
                    let _ = sim.decide(&decision.id, &k);
                }
            }
            StepOutcome::Paused { .. } => {}
        }
    }
    sim.report()
}


#[cfg(test)]
mod tests {
    use super::*;

    fn site(id: &str) -> SitePreset {
        crate::site::builtin().into_iter().find(|p| p.id == id).unwrap_or_else(|| crate::site::builtin().remove(0))
    }

    #[test]
    fn two_weeks_accumulate_wear_and_report() {
        let s = site("busan_newport_rmg");
        let cfg = OperationConfig { asset_id: "t".into(), days: 14, start_month: Some(7), start_day: Some(1), seed: 3, ..Default::default() };
        let mut sim = OpSim::new(s, cfg, None).unwrap();
        let r = run_to_end(&mut sim);
        assert!(r.finished);
        assert_eq!(r.daily.len(), 14);
        assert!(r.kpi.moves > 100.0, "{}", r.kpi.moves);
        assert!(r.kpi.availability > 0.3 && r.kpi.availability <= 1.0);
        assert!(r.ledger.component("rope_hoist").unwrap().life_used > 0.0);
        assert!(r.ledger.history.len() == 14);
        assert!(r.kpi.energy_kwh > 0.0);
    }

    #[test]
    fn ask_policy_pauses_for_decisions_and_blocks_unsafe_choice() {
        let s = site("busan_newport_rmg");
        let cfg = OperationConfig { asset_id: "t".into(), days: 60, start_month: Some(8), start_day: Some(1), seed: 5, policy: DecisionPolicy::Ask, ..Default::default() };
        let mut sim = OpSim::new(s, cfg, None).unwrap();
        let mut asked = 0;
        loop {
            match sim.run(10_000) {
                StepOutcome::Finished => break,
                StepOutcome::Decision { decision } => {
                    asked += 1;
                    if let Some(b) = decision.prompt.options.iter().find(|o| !o.enabled) {
                        assert!(sim.decide(&decision.id, &b.key).is_err());
                    }
                    assert!(decision.prompt.options[0].recommended, "recommended option first");
                    sim.decide(&decision.id, &decision.prompt.recommended).unwrap();
                }
                StepOutcome::Paused { .. } => {}
            }
        }
        assert!(asked > 0);
        let r = sim.report();
        assert!(r.decisions.iter().all(|d| d.chosen.is_some()));
        assert!(r.kpi.decisions_by_operator as usize == asked);
    }

    #[test]
    fn ignoring_warnings_raises_expected_incidents() {
        let s = site("busan_newport_rmg");
        let mk = |p| OperationConfig { asset_id: "t".into(), days: 45, start_month: Some(8), start_day: Some(1), seed: 9, policy: p, incidents: false, ..Default::default() };
        let safe = run_to_end(&mut OpSim::new(s.clone(), mk(DecisionPolicy::Recommended), None).unwrap());
        let risky = run_to_end(&mut OpSim::new(s, mk(DecisionPolicy::Risky), None).unwrap());
        let rate = |r: &OperationReport| r.kpi.expected_incidents / r.kpi.operating_h.max(1.0);
        assert!(rate(&risky) > rate(&safe), "{} vs {}", rate(&risky), rate(&safe));
        // 알림을 무시하면 폭풍 일주 같은 사고로 오히려 정지가 길어질 수 있다 — 처리량은 비교하지 않는다
    }

    #[test]
    fn ledger_continues_across_runs() {
        let s = site("incheon_steel_stockyard");
        let cfg = OperationConfig { asset_id: "t".into(), days: 5, seed: 1, ..Default::default() };
        let r1 = run_to_end(&mut OpSim::new(s.clone(), cfg.clone(), None).unwrap());
        let l1 = r1.ledger.clone();
        let rope1 = l1.component("rope_hoist").unwrap().life_used;
        let r2 = run_to_end(&mut OpSim::new(s, cfg, Some(l1)).unwrap());
        assert_eq!(r2.ledger.day, 10.0);
        assert_eq!(r2.ledger.runs, 2);
        assert!(r2.ledger.component("rope_hoist").unwrap().life_used > rope1);
        assert_eq!(r2.ledger.history.len(), 10);
    }
}
