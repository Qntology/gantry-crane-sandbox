//! 현장 프리셋 — 물량 · 하중(무게) · 환경 · 날씨(기후) · 운영 조건(작업자 · 정비 · 안전 장치)을 한 묶음으로.
//!
//! 내장 프리셋은 조사한 실데이터(출처는 `sources` 와 docs/site-operation.md)로 만든 대표 현장이다. 사용자는 내장
//! 프리셋을 바탕으로 값을 바꿔 새 이름으로 저장하고(`SiteStore::save`), 저장한 프리셋을 다시 불러 쓴다.
//! 내장 프리셋 이름으로는 저장할 수 없다 (덮어쓰기 방지).

use crate::alerts::AlertPolicy;
use crate::antisway::AntiSwayMode;
use crate::climate_data;
use crate::crane_profile::{CraneRef, CraneSpec};
use crate::cyclesim::{AntiSwayConfig, LoadShape, MoveMode, Scenario, SimConfig, StartState, Waypoint};
use crate::duty::{DutyConfig, LoadBin};
use crate::environment::Environment;
use crate::weather::{Climate, HourWeather};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 운반 물량 (수요)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Throughput {
    /// 운전 시간당 목표 이동 수 (회/h) — 하역 계획 · 생산 계획이 요구하는 양
    pub moves_per_hour: f64,
    /// 교대 1회 시간 (h)
    pub shift_hours: f64,
    pub shifts_per_day: u32,
    /// 첫 교대 시작 시각 (0–23)
    pub first_shift_start_h: u32,
    /// 주당 운전 일수 (5 · 6 · 7)
    pub days_per_week: u32,
    /// 교대당 식사 · 휴식으로 빠지는 시간 (분)
    #[serde(default)]
    pub break_min_per_shift: f64,
    /// 일 수요 변동 계수 (표준편차 / 평균)
    #[serde(default)]
    pub demand_cv: f64,
    /// 이동 한 번에 동역학 밖에서 드는 시간 (스프레더 잠금 · 줄걸이 · 정렬 · 신호 대기, s)
    pub handling_overhead_s: f64,
    /// 대표 이동 거리: 주행 · 횡행 · 권상 행정 (m)
    pub travel_m: f64,
    pub trolley_m: f64,
    pub lift_m: f64,
}

/// 하중 (무게) 조건
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProfile {
    /// "container" · "coil" · "precast" · "block" · "general"
    pub kind: String,
    /// 정격 하중 대비 비율 · 비중
    pub spectrum: Vec<LoadBin>,
    /// 하중 항력 계수 × 바람 받는 면적 (m²). 없으면 EN 13001-2 기본식
    #[serde(default)]
    pub cd_area_m2: Option<f64>,
    /// 스프레더 · 리프팅 빔 · 줄걸이 등 하중 밖 부속 질량 (kg) — 훅 질량에 더한다
    #[serde(default)]
    pub attachment_kg: f64,
    /// 정격 대비 이 비율 이상은 중량물 작업 (작업계획서 · 신호수, ASME P30.1 critical lift)
    #[serde(default = "default_critical")]
    pub critical_fraction: f64,
    /// 신고 무게와 달리 정격을 넘는 하중이 섞이는 비율 (정격 하중 제한 장치가 막아야 하는 들기)
    #[serde(default)]
    pub overweight_share: f64,
    /// 하중 무게 출처 · 설명
    #[serde(default)]
    pub note: String,
}

fn default_critical() -> f64 {
    0.9
}

/// 설치 환경
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteEnvironment {
    pub indoor: bool,
    #[serde(default)]
    pub altitude_m: f64,
    /// ISO 9223 부식성 범주 "C1" … "C5", "CX"
    pub corrosivity: String,
    /// 분진 · 오염: "clean" · "normal" · "dusty" · "severe" (ISO 281 오염 계수 범주)
    pub dust: String,
    /// 바람을 받는 높이 (거더 · 트롤리, m)
    pub crane_height_m: f64,
    /// 실내 온도 = base + coupling × (바깥 − base) + heat_source
    #[serde(default = "default_indoor_base")]
    pub indoor_base_c: f64,
    #[serde(default = "default_indoor_coupling")]
    pub indoor_coupling: f64,
    /// 열원 (제철 · 주조 공장 등) 가산 온도 (K)
    #[serde(default)]
    pub heat_source_c: f64,
    /// 레일 · 주행로 상태: "good" · "fair" · "poor"
    #[serde(default = "default_rail")]
    pub rail_condition: String,
    /// 해안 노출 (바다와의 거리 · 방위 · 해안 형태) — 있으면 염해 모델이 부식을 계산한다 (`marine`)
    #[serde(default)]
    pub coastal: Option<crate::marine::CoastalExposure>,
    /// 염해 방지 조치 — 프리셋 이름 또는 전체 값 (없으면 "none")
    #[serde(default)]
    pub protection: Option<crate::marine::ProtectionRef>,
}

impl SiteEnvironment {
    pub fn protection(&self) -> Result<crate::marine::MarineProtection, String> {
        match &self.protection {
            Some(p) => p.resolve(),
            None => crate::marine::protection_preset("none").ok_or_else(|| "염해 방지 프리셋 none 이 없습니다".to_string()),
        }
    }
}

fn default_indoor_base() -> f64 {
    18.0
}
fn default_indoor_coupling() -> f64 {
    0.45
}
fn default_rail() -> String {
    "good".into()
}

/// 물량 · 무게 · 환경 · 날씨 밖의 운영 조건
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsProfile {
    /// 운전원 숙련도: "novice" · "standard" · "expert"
    pub operator_experience: String,
    /// 운전실 냉방 (폭염 시 운전원 휴식 기준 완화)
    pub cab_air_conditioned: bool,
    /// 지상 작업자(신호수 · 줄걸이 · 래싱)가 붙는 작업인가 — 폭염 휴식 대상
    pub ground_crew: bool,
    pub signalman: bool,
    /// 작업 반경 출입 통제
    pub exclusion_zone: bool,
    /// 정비 품질: "good" · "average" · "poor"
    pub maintenance_quality: String,
    /// 정기 점검 주기 (일) — 월 점검 30
    pub periodic_inspection_days: u32,
    /// 법정 안전검사 주기 (일) — 산업안전보건법 시행규칙 제126조: 설치 후 3년 이내 최초, 그 뒤 2년마다 (건설현장 6개월)
    pub legal_inspection_days: u32,
    /// 설치 뒤 최초 안전검사 기한 (일)
    #[serde(default = "default_legal_first")]
    pub legal_first_days: u32,
    pub anemometer: bool,
    /// 레일 클램프 · 계류 장치 (옥외)
    pub rail_clamps: bool,
    /// 정격 하중 제한 장치 (EN 15011 · EN 12077-2)
    pub load_limiter: bool,
    /// 야간 작업 조도 (lx)
    pub lighting_lux: f64,
    /// 하루 연장 가능 시간 (h)
    pub max_overtime_h: f64,
}

impl Default for OperationsProfile {
    fn default() -> Self {
        Self {
            operator_experience: "standard".into(),
            cab_air_conditioned: true,
            ground_crew: true,
            signalman: true,
            exclusion_zone: true,
            maintenance_quality: "average".into(),
            periodic_inspection_days: 30,
            legal_inspection_days: 730,
            legal_first_days: 1095,
            anemometer: true,
            rail_clamps: true,
            load_limiter: true,
            lighting_lux: 100.0,
            max_overtime_h: 2.0,
        }
    }
}

/// 프리셋 이름 또는 전체 기후 자료
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClimateRef {
    Preset(String),
    Data(Box<Climate>),
}

impl ClimateRef {
    pub fn resolve(&self) -> Result<Climate, String> {
        match self {
            ClimateRef::Preset(n) => climate_data::preset(n).ok_or_else(|| {
                let names: Vec<String> = climate_data::all().into_iter().map(|c| c.id).collect();
                format!("기후 프리셋 '{}' 이 없습니다 (사용 가능: {})", n, names.join(", "))
            }),
            ClimateRef::Data(c) => {
                c.validate()?;
                Ok((**c).clone())
            }
        }
    }
}

fn default_legal_first() -> u32 {
    1095
}

fn default_antisway() -> AntiSwayMode {
    AntiSwayMode::Zvd
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitePreset {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// "container_yard" · "steel_yard" · "shipyard" · "precast_yard" · "workshop" · "custom"
    pub application: String,
    pub crane: CraneRef,
    #[serde(default = "default_antisway")]
    pub antisway: AntiSwayMode,
    pub throughput: Throughput,
    pub loads: LoadProfile,
    pub environment: SiteEnvironment,
    pub climate: ClimateRef,
    #[serde(default)]
    pub operations: OperationsProfile,
    /// 알림 문턱 (바람 · 낙뢰 · 시정 · 폭염 · 한파 · 부품 상태)
    #[serde(default)]
    pub alerts: AlertPolicy,
    /// 화물 시나리오 (프리셋 이름 또는 전체 값) — 있으면 하중 분포 · 화물별 바람 제한 · 위험물 · 물동 패턴을 쓴다
    #[serde(default)]
    pub cargo: Option<crate::cargo::CargoRef>,
    #[serde(default)]
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSummary {
    pub id: String,
    pub name: String,
    pub application: String,
    pub builtin: bool,
    pub path: Option<String>,
}

const CORROSIVITY: [&str; 6] = ["C1", "C2", "C3", "C4", "C5", "CX"];
const DUST: [&str; 4] = ["clean", "normal", "dusty", "severe"];

/// ISO 9223 범주 → 0–5
pub fn corrosivity_index(c: &str) -> Option<usize> {
    CORROSIVITY.iter().position(|x| x.eq_ignore_ascii_case(c.trim()))
}

pub fn dust_index(d: &str) -> Option<usize> {
    DUST.iter().position(|x| x.eq_ignore_ascii_case(d.trim()))
}

impl SitePreset {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() || !self.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(format!("프리셋 id '{}' 는 영문 · 숫자 · _ · - 만 쓸 수 있습니다", self.id));
        }
        let spec = self.crane.resolve()?;
        self.climate.resolve()?;
        let t = &self.throughput;
        for (k, v) in [("moves_per_hour", t.moves_per_hour), ("shift_hours", t.shift_hours), ("handling_overhead_s", t.handling_overhead_s + 1.0), ("lift_m", t.lift_m)] {
            if !(v.is_finite() && v > 0.0) {
                return Err(format!("throughput.{k} 는 양수여야 합니다 ({v})"));
            }
        }
        if t.shifts_per_day == 0 || t.shifts_per_day as f64 * t.shift_hours > 24.0 + 1e-9 {
            return Err(format!("교대 {} × {} h 는 하루 24 h 안이어야 합니다", t.shifts_per_day, t.shift_hours));
        }
        if !(1..=7).contains(&t.days_per_week) {
            return Err("days_per_week 는 1–7 이어야 합니다".into());
        }
        if self.loads.spectrum.is_empty() || self.loads.spectrum.iter().all(|b| b.share <= 0.0) {
            return Err("하중 스펙트럼이 비어 있습니다".into());
        }
        if self.loads.spectrum.iter().any(|b| !(b.fraction >= 0.0 && b.fraction <= 1.5)) {
            return Err("하중 비율은 0–1.5 (정격 대비) 여야 합니다".into());
        }
        if corrosivity_index(&self.environment.corrosivity).is_none() {
            return Err(format!("부식성 범주 '{}' 를 모릅니다 (C1–C5, CX)", self.environment.corrosivity));
        }
        if dust_index(&self.environment.dust).is_none() {
            return Err(format!("분진 범주 '{}' 를 모릅니다 (clean · normal · dusty · severe)", self.environment.dust));
        }
        if let Some(c) = &self.environment.coastal {
            c.validate()?;
        }
        if let Some(c) = &self.cargo {
            c.resolve()?.validate()?;
        }
        self.environment.protection()?.validate()?;
        let lift_room = spec.structure.rope_max_m - spec.structure.rope_min_m;
        if t.lift_m > lift_room + 1e-9 {
            return Err(format!("권상 행정 {} m 가 로프 범위 {:.1} m 보다 깁니다", t.lift_m, lift_room));
        }
        Ok(())
    }

    /// 화물 시나리오가 있으면 하중 분포 · 정격 초과 비율을 화물에서 다시 만든 사본 (없으면 그대로)
    pub fn effective(&self) -> Result<SitePreset, String> {
        let mut p = self.clone();
        if let Some(c) = &self.cargo {
            let s = c.resolve()?;
            let rated = self.crane.resolve()?.structure.rated_load_kg;
            p.loads.spectrum = s.spectrum(rated)?;
            p.loads.overweight_share = s.overweight_share(rated)?;
            p.loads.note = format!("화물 시나리오 '{}' 에서 만든 하중 분포", s.name);
        }
        Ok(p)
    }

    pub fn spec(&self) -> Result<CraneSpec, String> {
        let mut s = self.crane.resolve()?;
        // 스프레더 · 리프팅 빔은 훅 질량으로 다룬다
        s.structure.hook_mass_kg += self.loads.attachment_kg.max(0.0);
        if self.environment.indoor {
            s.limits.wind_in_service_mps = 0.0;
        }
        Ok(s)
    }

    /// 근무(듀티) 설정 — 교대 하나
    pub fn duty_config(&self) -> DutyConfig {
        DutyConfig {
            cycles_per_hour: self.throughput.moves_per_hour,
            hours: self.throughput.shift_hours,
            spectrum: self.loads.spectrum.clone(),
            ..Default::default()
        }
    }

    /// 대표 운반 사이클 시나리오: 걸기 → 들어 옮기기 → 내려놓기 → 복귀
    pub fn scenario(&self, load_kg: f64, speed_scale: f64, env: Environment) -> Result<Scenario, String> {
        let spec = self.spec()?;
        let s = &spec.structure;
        let t = &self.throughput;
        let top = (s.rope_min_m + 0.5).min(s.rope_max_m - 0.5);
        let low = (top + t.lift_m).min(s.rope_max_m);
        let x0 = (s.runway_m * 0.1).max(1.0);
        let y0 = (0.5 * (s.span_m - t.trolley_m.min(s.span_m))).max(0.5);
        let x1 = (x0 + t.travel_m).min(s.runway_m - 1.0).max(x0);
        let y1 = (y0 + t.trolley_m).min(s.span_m - 0.5);
        // 긴 로프(골리앗)는 흔들림 주기가 길어 시간 간격을 늘려도 된다
        let dt = if s.rope_max_m > 30.0 { 0.005 } else { 0.002 };
        let wp = |label: &str, x: Option<f64>, y: Option<f64>, l: f64, dwell: f64, set: Option<f64>| Waypoint {
            label: label.into(),
            x,
            y,
            l: Some(l),
            mode: MoveMode::Sequential,
            dwell_s: dwell,
            set_load_kg: set,
        };
        Ok(Scenario {
            name: format!("{}_{:.0}kg", self.id, load_kg),
            crane: CraneRef::Spec(Box::new(spec)),
            start: StartState { x: x0, y: y0, l: top, load_kg: 0.0 },
            waypoints: vec![
                wp("하중 걸기", None, None, low, 2.0, Some(load_kg.max(1.0))),
                wp("들어 옮기기", Some(x1), Some(y1), top, 0.0, None),
                wp("내려놓기", None, None, low, 2.0, Some(0.0)),
                wp("복귀", Some(x0), Some(y0), top, 0.0, None),
            ],
            antisway: AntiSwayConfig { mode: self.antisway, ..Default::default() },
            env,
            sim: SimConfig { dt, sample_dt: 0.05, settle_s: 3.0, ..Default::default() },
            load: LoadShape { cd_area_m2: self.loads.cd_area_m2, hook_cd_area_m2: None },
            speed_scale: speed_scale.clamp(0.2, 1.0),
            accel_scale: 1.0,
            events: Vec::new(),
        })
    }
}

/// 공기 밀도 (kg/m³): 표준 대기 기압 × 이상 기체
pub fn air_density(altitude_m: f64, temp_c: f64) -> f64 {
    let p = 101_325.0 * (1.0 - 2.255_77e-5 * altitude_m.max(0.0)).powf(5.255_88);
    p / (287.05 * (temp_c + 273.15).max(200.0))
}

/// 한 시간 날씨 → 동역학 환경 (평균 풍속 + 돌풍)
pub fn environment_for(w: &HourWeather, altitude_m: f64) -> Environment {
    let mean = w.wind_components(w.wind_mean_mps);
    Environment {
        ambient_c: w.temp_c,
        air_density_kg_m3: air_density(altitude_m, w.temp_c),
        wind_mean_mps: mean,
        gust_amp_mps: (w.gust_mps - w.wind_mean_mps).max(0.0),
        gust_period_s: 8.0,
        adhesion_mu: Some(crate::field_constants::adhesion_mu(w)),
    }
}

/// 내장 + 사용자 프리셋 저장소
pub struct SiteStore {
    dir: PathBuf,
}

impl SiteStore {
    pub fn new(dir: &Path) -> Self {
        Self { dir: dir.to_path_buf() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path_of(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    pub fn list(&self) -> Vec<SiteSummary> {
        let mut out: Vec<SiteSummary> = builtin()
            .into_iter()
            .map(|p| SiteSummary { id: p.id, name: p.name, application: p.application, builtin: true, path: None })
            .collect();
        if let Ok(rd) = std::fs::read_dir(&self.dir) {
            let mut files: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "json").unwrap_or(false)).collect();
            files.sort();
            for p in files {
                if let Ok(sp) = std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str::<SitePreset>(&t).map_err(|e| e.to_string())) {
                    if out.iter().any(|s| s.id == sp.id) {
                        continue;
                    }
                    out.push(SiteSummary { id: sp.id, name: sp.name, application: sp.application, builtin: false, path: Some(p.to_string_lossy().into_owned()) });
                }
            }
        }
        out
    }

    pub fn get(&self, id: &str) -> Result<SitePreset, String> {
        if let Some(p) = builtin().into_iter().find(|p| p.id == id) {
            return Ok(p);
        }
        let p = self.path_of(id);
        let txt = std::fs::read_to_string(&p).map_err(|_| {
            let ids: Vec<String> = self.list().into_iter().map(|s| s.id).collect();
            format!("현장 프리셋 '{}' 이 없습니다 (사용 가능: {})", id, ids.join(", "))
        })?;
        serde_json::from_str(&txt).map_err(|e| format!("{}: {e}", p.display()))
    }

    /// 사용자 프리셋으로 저장한다 (내장 이름은 거부)
    pub fn save(&self, p: &SitePreset) -> Result<PathBuf, String> {
        if is_builtin(&p.id) {
            return Err(format!("'{}' 는 내장 프리셋 이름입니다. 다른 id 로 저장하세요", p.id));
        }
        p.validate()?;
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))?;
        let path = self.path_of(&p.id);
        std::fs::write(&path, serde_json::to_string_pretty(p).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(path)
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        if is_builtin(id) {
            return Err(format!("내장 프리셋 '{id}' 은 지울 수 없습니다"));
        }
        let p = self.path_of(id);
        std::fs::remove_file(&p).map_err(|e| format!("{}: {e}", p.display()))
    }
}

pub fn is_builtin(id: &str) -> bool {
    builtin().iter().any(|p| p.id == id)
}

/// 점 표기 경로(예: "throughput.moves_per_hour")의 값을 바꾼 프리셋. 값은 JSON 으로 읽고, 안 되면 문자열로 둔다.
pub fn with_overrides(p: &SitePreset, sets: &[(String, String)]) -> Result<SitePreset, String> {
    let mut v = serde_json::to_value(p).map_err(|e| e.to_string())?;
    for (path, raw) in sets {
        let val: serde_json::Value = serde_json::from_str(raw).unwrap_or_else(|_| serde_json::Value::String(raw.clone()));
        let mut cur = &mut v;
        let parts: Vec<&str> = path.split('.').collect();
        for (i, k) in parts.iter().enumerate() {
            let last = i + 1 == parts.len();
            let obj = cur.as_object_mut().ok_or_else(|| format!("'{path}': '{k}' 앞이 객체가 아닙니다"))?;
            if last {
                if !obj.contains_key(*k) {
                    return Err(format!("'{path}': '{k}' 항목이 없습니다"));
                }
                obj.insert(k.to_string(), val.clone());
                break;
            }
            cur = obj.get_mut(*k).ok_or_else(|| format!("'{path}': '{k}' 항목이 없습니다"))?;
        }
    }
    serde_json::from_value(v).map_err(|e| format!("바꾼 값이 프리셋 형식에 맞지 않습니다: {e}"))
}

/// 내장 현장 프리셋
pub fn builtin() -> Vec<SitePreset> {
    crate::site_presets::all()
}

pub(crate) fn mk_bins(v: &[(f64, f64)]) -> Vec<LoadBin> {
    v.iter().map(|(f, s)| LoadBin { fraction: *f, share: *s }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_presets_validate_and_ids_unique() {
        let all = builtin();
        assert!(all.len() >= 5);
        for p in all.iter() {
            p.validate().unwrap_or_else(|e| panic!("{}: {e}", p.id));
        }
        let mut ids: Vec<&str> = all.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
    }

    #[test]
    fn save_load_override_and_protect_builtin() {
        let dir = std::env::temp_dir().join(format!("crane_sites_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let st = SiteStore::new(&dir);
        let base = builtin().remove(0);
        assert!(st.save(&base).is_err(), "builtin id must not be overwritten");
        let mut mine = with_overrides(&base, &[("throughput.moves_per_hour".into(), "12.5".into()), ("environment.corrosivity".into(), "C4".into())]).unwrap();
        mine.id = "my_site".into();
        assert_eq!(mine.throughput.moves_per_hour, 12.5);
        assert_eq!(mine.environment.corrosivity, "C4");
        st.save(&mine).unwrap();
        let back = st.get("my_site").unwrap();
        assert_eq!(back.throughput.moves_per_hour, 12.5);
        assert!(st.list().iter().any(|s| s.id == "my_site" && !s.builtin));
        assert!(with_overrides(&base, &[("throughput.nope".into(), "1".into())]).is_err());
        st.delete("my_site").unwrap();
        assert!(st.get("my_site").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scenario_runs_for_every_builtin_site() {
        for p in builtin() {
            let spec = p.spec().unwrap();
            let load = 0.6 * spec.structure.rated_load_kg;
            let sc = p.scenario(load, 1.0, Environment::default()).unwrap();
            let r = crate::cyclesim::simulate(&sc).unwrap_or_else(|e| panic!("{}: {e}", p.id));
            assert_eq!(r.moves.len(), 4, "{}", p.id);
            assert!(r.load_peak_kg >= load - 1.0);
        }
    }

    #[test]
    fn air_density_falls_with_altitude_and_heat() {
        assert!((air_density(0.0, 15.0) - 1.225).abs() < 0.005);
        assert!(air_density(2000.0, 15.0) < 1.05);
        assert!(air_density(0.0, 35.0) < air_density(0.0, 0.0));
    }
}
