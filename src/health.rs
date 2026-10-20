//! 부품 내구도 · 안정성 원장 — 현장 운영 시뮬레이션이 하루마다 누적하고 파일로 계속 저장한다.
//!
//! 부품마다 '수명 사용 비율'(0 신품 → 1 교체 기준)을 쌓고, 그날의 안정성 여유(≥ 1 이면 기준 안)를 함께 남긴다.
//! 같은 자산(asset_id)으로 다시 돌리면 원장을 읽어 이어서 쌓으므로 실제 현장처럼 몇 달 · 몇 년의 내구도 평가가 된다.
//!
//! | 부품 | 누적 방식 | 근거 |
//! | --- | --- | --- |
//! | 권상 와이어로프 | Feyrer 굽힘 피로 Miner 합 ÷ (현장 보정 × 부식 환경 배수) | Feyrer 갱신 자료, ISO 4309 폐기 |
//! | 브레이크 라이닝 | 비상 정지 마찰일 마모 + 체결 1회당 기계 마모 × 분진 배수 | mayr ROBA-stop-S, 현장 교체 주기로 보정 |
//! | 모터 | 권선 절연 열 열화 2^((T − T_등급)/10) ÷ 20,000 h 와 베어링 L10 소비 중 큰 쪽 | IEC 60085 · Montsinger 10 K 규칙, ISO 281 |
//! | 감속기 · 기구 | 전부하 환산 시간 ∫(τ/τ_ref)³dt ÷ (기구 등급 설계 시간 × 분진 배수) | ISO 4301-1 · ISO 12482 · FEM 9.512 |
//! | 차륜 | 정격 바퀴 하중 환산 주행 거리 × 레일 상태 배수 ÷ 기준 수명 거리 | 현장 교체 주기로 보정 |
//! | 거더 용접부 | 들기마다 응력 범위 Δσ ∝ 매단 질량, Miner (Δσ/Δσc)³ / 2×10⁶, 부식 두께 감소로 응력 증가 | EN 13001-3-1 |
//! | 훅 · 훅블록 | 들기 × (하중비)³ ÷ (사용 등급 사이클 × 배수) | ASME B30.10 점검 기준, ISO 4301-1 U 등급 |
//! | 도장 · 부식 | 도장 수명(년) 소비 → 도장 파괴 뒤 강재 부식 ISO 9224 멱법칙 | ISO 12944-1 내구성, ISO 9223 · 9224 |
//! | 레일 클램프 | 체결 횟수 ÷ 설계 횟수 + 부식 시간 | 계류 · 폭풍 고정 장치 |

use crate::crane_profile::{Axis, CraneSpec};
use crate::site::{corrosivity_index, dust_index, SitePreset};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const HEALTH_RECIPE: &str = "crane-health-v1";

/// 현장 내구도 모델 상수 — `fieldcal` 이 실데이터에 맞춰 보정한다
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurabilityModel {
    /// Feyrer 실험실 굽힘 수명 → 현장 수명 배수 (윤활 · 시브 정렬 · 다층 권취 등 실험실 밖 요인)
    pub rope_field_factor: f64,
    /// 부식성 범주(C1 … CX)별 로프 수명 배수
    pub rope_env_factor: Vec<f64>,
    /// 브레이크 체결 1회당 기계 마모 (mm) — 인버터가 먼저 세운 정상 정지에서도 남는 마모
    pub brake_wear_per_set_mm: f64,
    /// 분진 범주(clean · normal · dusty · severe)별 베어링 · 감속기 · 브레이크 수명 배수
    pub dust_life_factor: Vec<f64>,
    /// 권선 절연: 등급 온도에서의 열 수명 (h) · 수명 반감 온도 (K)
    pub insulation_life_h: f64,
    pub insulation_halving_k: f64,
    /// 모터 베어링 L10h (정격 하중 · 정격 속도, h)
    pub motor_bearing_l10_h: f64,
    /// 차륜 기준 수명 — 정격 바퀴 하중으로 주행한 거리 (km)
    pub wheel_life_km: f64,
    /// 레일 상태(good · fair · poor)별 차륜 마모 배수
    pub wheel_rail_factor: Vec<f64>,
    /// 거더 용접 상세 등급 Δσc (MPa, 2×10⁶ 회, m = 3) · 설계 이용률
    pub girder_detail_mpa: f64,
    pub girder_utilization: f64,
    /// 구조 설계 하중 스펙트럼 계수 (설계 사이클에서 피로 손상 = 이용률³ 이 되도록)
    pub girder_design_kp: f64,
    /// 거더 대표 판 두께 (mm) — 부식 두께 감소의 응력 증가 계산용
    pub girder_plate_mm: f64,
    /// 거더 부식 한계 (판 두께 대비 비율)
    #[serde(default = "default_corr_limit")]
    pub girder_corrosion_limit: f64,
    /// 훅 설계 사이클 = 사용 등급 설계 사이클 × 배수
    pub hook_cycle_factor: f64,
    /// 부식성 범주별 도장 수명 (년)
    pub coating_life_years: Vec<f64>,
    /// 부식성 범주별 탄소강 첫해 부식 속도 (µm/년, ISO 9223)
    pub corrosion_um_per_year: Vec<f64>,
    /// ISO 9224 장기 부식 지수 b (d = r_corr · t^b)
    pub corrosion_exponent: f64,
    /// 레일 클램프 설계 작동 횟수 · 부식 수명 (년, C3 기준)
    pub rail_clamp_ops: f64,
    pub rail_clamp_years: f64,
    /// 상태 문턱 (수명 사용 비율): 관찰 · 정비 계획 · 교체
    pub watch_at: f64,
    pub plan_at: f64,
    pub replace_at: f64,
    /// 남은 수명이 이 날 수보다 짧으면 정비 계획 상태로
    #[serde(default = "default_plan_horizon")]
    pub plan_horizon_days: f64,
}

fn default_corr_limit() -> f64 {
    0.10
}

fn default_plan_horizon() -> f64 {
    30.0
}

impl Default for DurabilityModel {
    fn default() -> Self {
        crate::field_constants::durability()
    }
}

impl DurabilityModel {
    fn by_corrosivity(v: &[f64], c: &str) -> f64 {
        corrosivity_index(c).and_then(|i| v.get(i).cloned()).unwrap_or(1.0)
    }
    fn by_dust(v: &[f64], d: &str) -> f64 {
        dust_index(d).and_then(|i| v.get(i).cloned()).unwrap_or(1.0)
    }
    pub fn rope_env(&self, corrosivity: &str) -> f64 {
        Self::by_corrosivity(&self.rope_env_factor, corrosivity).max(0.05)
    }
    pub fn dust(&self, dust: &str) -> f64 {
        Self::by_dust(&self.dust_life_factor, dust).max(0.05)
    }
    pub fn coating_years(&self, corrosivity: &str) -> f64 {
        Self::by_corrosivity(&self.coating_life_years, corrosivity).max(0.5)
    }
    pub fn corrosion_rate(&self, corrosivity: &str) -> f64 {
        Self::by_corrosivity(&self.corrosion_um_per_year, corrosivity).max(0.0)
    }
    pub fn rail_factor(&self, cond: &str) -> f64 {
        let i = match cond.trim() {
            "good" => 0,
            "fair" => 1,
            "poor" => 2,
            _ => 0,
        };
        self.wheel_rail_factor.get(i).cloned().unwrap_or(1.0).max(0.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    Rope,
    Brake,
    Motor,
    Gearbox,
    Wheel,
    Girder,
    Hook,
    Coating,
    RailClamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Good,
    Watch,
    Plan,
    Replace,
    /// 교체 기준을 넘긴 채 운전 중
    Overdue,
}

impl Health {
    pub fn label(&self) -> &'static str {
        match self {
            Health::Good => "양호",
            Health::Watch => "관찰",
            Health::Plan => "정비 계획",
            Health::Replace => "교체 필요",
            Health::Overdue => "기준 초과",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub kind: ComponentKind,
    #[serde(default)]
    pub axis: Option<Axis>,
    pub label: String,
    /// 수명 사용 비율 (0 신품, 1 교체 기준)
    pub life_used: f64,
    /// 대표 물리량과 단위 (라이닝 두께 mm, 손상 합, 부식 µm …)
    pub value: f64,
    pub unit: String,
    /// 세부 손상 (이름, 값) — 모터: 절연 · 베어링, 거더: 피로 · 부식 두께
    #[serde(default)]
    pub sub: Vec<(String, f64)>,
    /// 최근 운전일의 하루 소비율 (지수 평활, /일)
    pub rate_per_day: f64,
    pub rul_days: Option<f64>,
    pub status: Health,
    pub installed_day: f64,
    pub replacements: u32,
    pub last_inspection_day: Option<f64>,
    /// 안정성 여유 (≥ 1 이면 기준 안): 최근 하루 최소 · 누적 최소
    pub margin: Option<f64>,
    pub margin_min: Option<f64>,
    #[serde(default)]
    pub margin_label: String,
    pub basis: String,
}

impl Component {
    #[allow(clippy::too_many_arguments)]
    fn new(id: &str, kind: ComponentKind, axis: Option<Axis>, label: &str, unit: &str, value: f64, margin_label: &str, basis: &str) -> Self {
        Self {
            id: id.into(),
            kind,
            axis,
            label: label.into(),
            life_used: 0.0,
            value,
            unit: unit.into(),
            sub: Vec::new(),
            rate_per_day: 0.0,
            rul_days: None,
            status: Health::Good,
            installed_day: 0.0,
            replacements: 0,
            last_inspection_day: None,
            margin: None,
            margin_min: None,
            margin_label: margin_label.into(),
            basis: basis.into(),
        }
    }

    pub fn sub_value(&self, k: &str) -> f64 {
        self.sub.iter().find(|(n, _)| n == k).map(|(_, v)| *v).unwrap_or(0.0)
    }

    fn set_sub(&mut self, k: &str, v: f64) {
        if let Some(x) = self.sub.iter_mut().find(|(n, _)| n == k) {
            x.1 = v;
        } else {
            self.sub.push((k.to_string(), v));
        }
    }
}

/// 하루 동안 쓴 양 — 운영 시뮬레이션이 채운다
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DayUsage {
    pub moves: f64,
    pub op_hours: f64,
    /// Feyrer 굽힘 손상 (환경 배수 전)
    pub rope_damage: f64,
    pub brake_sets: [f64; 3],
    pub brake_dyn_wear_mm: [f64; 3],
    /// 등급 온도 환산 절연 열화 시간 (h)
    pub insulation_aging_h: [f64; 3],
    /// 정격 환산 베어링 사용 시간 (h)
    pub bearing_h: [f64; 3],
    /// 전부하 환산 시간 (h)
    pub flh_h: [f64; 3],
    /// 정격 바퀴 하중 환산 주행 거리 (km): 주행 · 횡행
    pub wheel_km_eq: [f64; 2],
    /// 거더 피로 Miner 증분 (부식 응력 증가 전)
    pub girder_damage: f64,
    /// Σ(하중비)³ 들기
    pub hook_cycles_eq: f64,
    pub clamp_ops: f64,
    /// 안정성 여유 (그날 최소)
    pub margins: Margins,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Margins {
    /// 로프 안전율 / 요구 안전율
    pub rope_sf: Option<f64>,
    /// 권상 브레이크 토크 / 정적 토크 ÷ 요구 배수
    pub hoist_brake: Option<f64>,
    /// 모터 열 여유: 허용 상승 / 실제 상승
    pub motor_thermal: [Option<f64>; 3],
    /// 바퀴–레일 점착 여유: 쓸 수 있는 견인력 / 필요한 가감속 힘 (주행 · 횡행)
    pub adhesion: [Option<f64>; 2],
    /// 바람 일주(밀림) 여유 · 전도 여유
    pub runaway: Option<f64>,
    pub overturn: Option<f64>,
}

fn min_opt(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (x, None) => x,
        (None, y) => y,
    }
}

impl Margins {
    pub fn merge(&mut self, o: &Margins) {
        self.rope_sf = min_opt(self.rope_sf, o.rope_sf);
        self.hoist_brake = min_opt(self.hoist_brake, o.hoist_brake);
        for k in 0..3 {
            self.motor_thermal[k] = min_opt(self.motor_thermal[k], o.motor_thermal[k]);
        }
        for k in 0..2 {
            self.adhesion[k] = min_opt(self.adhesion[k], o.adhesion[k]);
        }
        self.runaway = min_opt(self.runaway, o.runaway);
        self.overturn = min_opt(self.overturn, o.overturn);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEvent {
    pub day: f64,
    pub hour: u8,
    /// "replace" · "inspect" · "periodic_inspection" · "legal_inspection" · "incident" · "decision" · "note"
    pub kind: String,
    #[serde(default)]
    pub component: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub day: f64,
    pub doy: u32,
    pub op_hours: f64,
    pub moves: f64,
    /// 부품 순서대로 수명 사용 비율 · 안정성 여유
    pub life: Vec<f64>,
    pub margin: Vec<Option<f64>>,
    pub risk_max: f64,
    pub alerts: u32,
    pub downtime_h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthLedger {
    pub recipe: String,
    pub asset_id: String,
    pub crane: String,
    pub site: String,
    pub created_unix: i64,
    pub updated_unix: i64,
    /// 누적 시뮬레이션 날 수
    pub day: f64,
    /// 다음에 시작할 연중 날짜 (계절을 이어 간다)
    pub next_doy: u32,
    pub operating_hours: f64,
    pub moves: f64,
    pub energy_kwh: f64,
    pub incidents: u32,
    pub runs: u32,
    pub last_periodic_inspection_day: f64,
    pub last_legal_inspection_day: f64,
    pub components: Vec<Component>,
    pub events: Vec<HealthEvent>,
    pub history: Vec<Snapshot>,
}

fn now_unix() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// 상태 변화 (알림으로 이어진다)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChange {
    pub id: String,
    pub label: String,
    pub from: Health,
    pub to: Health,
    pub life_used: f64,
    pub rul_days: Option<f64>,
}

impl HealthLedger {
    pub fn new(asset_id: &str, spec: &CraneSpec, site: &SitePreset) -> Self {
        let mut c = Vec::new();
        let rope_basis = "Feyrer 굽힘 피로 Miner 합, ISO 4309 폐기 검사";
        c.push(Component::new("rope_hoist", ComponentKind::Rope, Some(Axis::Hoist), "권상 와이어로프", "Miner", 0.0, "안전율 / 요구 Zp", rope_basis));
        for a in Axis::all() {
            let b = &spec.drive(a).brake;
            c.push(Component::new(
                &format!("brake_{}", a.key()),
                ComponentKind::Brake,
                Some(a),
                &format!("{} 브레이크 라이닝", a.label()),
                "mm",
                b.lining_new_mm,
                if a == Axis::Hoist { "제동 토크 / 정적 토크 ÷ 1.6" } else { "" },
                "마찰일 마모 (mayr) + 체결당 기계 마모, Konecranes 30% · 15% 여유 기준",
            ));
        }
        for a in Axis::all() {
            c.push(Component::new(
                &format!("motor_{}", a.key()),
                ComponentKind::Motor,
                Some(a),
                &format!("{} 모터 (절연 · 베어링)", a.label()),
                "h",
                0.0,
                "허용 온도 상승 / 실제 상승",
                "IEC 60085 열 수명 20,000 h · 10 K 반감, ISO 281 L10",
            ));
        }
        for a in Axis::all() {
            c.push(Component::new(
                &format!("gear_{}", a.key()),
                ComponentKind::Gearbox,
                Some(a),
                &format!("{} 감속기 · 기구 (SWP)", a.label()),
                "전부하 h",
                0.0,
                "",
                "ISO 4301-1 · ISO 12482 · FEM 9.512 기구 등급 설계 시간",
            ));
        }
        for a in [Axis::Travel, Axis::Trolley] {
            c.push(Component::new(&format!("wheel_{}", a.key()), ComponentKind::Wheel, Some(a), &format!("{} 차륜", a.label()), "km", 0.0, "점착 견인력 / 필요 힘", "정격 바퀴 하중 환산 주행 거리"));
        }
        c.push(Component::new("girder", ComponentKind::Girder, None, "거더 용접부 (피로 · 부식)", "Miner", 0.0, "전도 안정 모멘트 / 전도 모멘트", "EN 13001-3-1 상세 등급, ISO 9224"));
        c.push(Component::new("hook", ComponentKind::Hook, Some(Axis::Hoist), "훅 · 훅블록", "회", 0.0, "", "ASME B30.10 점검 기준, ISO 4301-1 U 등급"));
        c.push(Component::new("coating", ComponentKind::Coating, None, "도장 · 방청", "년", 0.0, "", "ISO 12944-1 내구성, ISO 9223 부식성"));
        if !site.environment.indoor {
            c.push(Component::new("rail_clamp", ComponentKind::RailClamp, Some(Axis::Travel), "레일 클램프 · 계류 장치", "회", 0.0, "제동 · 클램프 저항 / 풍하중", "EN 13001-2 비운전 폭풍, 계류 장치 점검"));
        }
        let t = now_unix();
        Self {
            recipe: HEALTH_RECIPE.into(),
            asset_id: asset_id.into(),
            crane: spec.name.clone(),
            site: site.id.clone(),
            created_unix: t,
            updated_unix: t,
            day: 0.0,
            next_doy: 0,
            operating_hours: 0.0,
            moves: 0.0,
            energy_kwh: 0.0,
            incidents: 0,
            runs: 0,
            last_periodic_inspection_day: 0.0,
            last_legal_inspection_day: 0.0,
            components: c,
            events: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn component(&self, id: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.id == id)
    }

    pub fn component_mut(&mut self, id: &str) -> Option<&mut Component> {
        self.components.iter_mut().find(|c| c.id == id)
    }

    fn status_of(life: f64, m: &DurabilityModel) -> Health {
        if life >= m.replace_at + 0.05 {
            Health::Overdue
        } else if life >= m.replace_at {
            Health::Replace
        } else if life >= m.plan_at {
            Health::Plan
        } else if life >= m.watch_at {
            Health::Watch
        } else {
            Health::Good
        }
    }

    /// 하루 사용량을 원장에 더한다. `day` 는 끝난 날의 누적 날 수.
    pub fn apply_day(&mut self, spec: &CraneSpec, site: &SitePreset, m: &DurabilityModel, u: &DayUsage, day: f64) -> Vec<StatusChange> {
        let env = &site.environment;
        let dust = m.dust(&env.dust);
        let corr = env.corrosivity.clone();
        let rate_corr = m.corrosion_rate(&corr);
        let coat_years = m.coating_years(&corr);
        let class_cycles = spec.class.design_cycles.max(1.0);
        let design_h = spec.class.design_full_load_hours.max(1.0);
        let mut changes = Vec::new();
        let operated = u.op_hours > 0.0;
        for c in self.components.iter_mut() {
            let before = c.life_used;
            let old_status = c.status;
            match c.kind {
                ComponentKind::Rope => {
                    let k = m.rope_field_factor.max(1e-6) * m.rope_env(&corr);
                    c.value += u.rope_damage;
                    c.life_used += u.rope_damage / k;
                    c.margin = u.margins.rope_sf;
                }
                ComponentKind::Brake => {
                    let a = c.axis.unwrap_or(Axis::Hoist);
                    let k = a.index();
                    let b = &spec.drive(a).brake;
                    let wear = u.brake_sets[k] * m.brake_wear_per_set_mm + u.brake_dyn_wear_mm[k];
                    c.value = (c.value - wear).max(0.0);
                    let usable = (b.lining_new_mm - b.lining_min_mm).max(1e-9);
                    c.life_used = ((b.lining_new_mm - c.value) / usable).max(0.0);
                    if a == Axis::Hoist {
                        c.margin = u.margins.hoist_brake;
                    }
                }
                ComponentKind::Motor => {
                    let k = c.axis.unwrap_or(Axis::Hoist).index();
                    let ins = c.sub_value("insulation") + u.insulation_aging_h[k] / m.insulation_life_h.max(1.0);
                    let brg = c.sub_value("bearing") + u.bearing_h[k] / (m.motor_bearing_l10_h.max(1.0) * dust);
                    c.set_sub("insulation", ins);
                    c.set_sub("bearing", brg);
                    c.value += u.insulation_aging_h[k];
                    c.life_used = ins.max(brg);
                    c.margin = u.margins.motor_thermal[k];
                }
                ComponentKind::Gearbox => {
                    let k = c.axis.unwrap_or(Axis::Hoist).index();
                    c.value += u.flh_h[k];
                    c.life_used += u.flh_h[k] / (design_h * dust);
                }
                ComponentKind::Wheel => {
                    let k = c.axis.unwrap_or(Axis::Travel).index().min(1);
                    let km = u.wheel_km_eq[k] * m.rail_factor(&env.rail_condition);
                    c.value += u.wheel_km_eq[k];
                    c.life_used += km / m.wheel_life_km.max(1.0);
                    c.margin = u.margins.adhesion[k];
                }
                ComponentKind::Girder => {
                    // 도장이 깨진 뒤 부식 두께 감소 → 판 두께가 줄어 응력이 커진다 (σ ∝ 1/t)
                    let loss_um = c.sub_value("corrosion_um");
                    let t0 = m.girder_plate_mm.max(1.0) * 1000.0;
                    let mag = t0 / (t0 - loss_um).max(0.5 * t0);
                    let fat = c.sub_value("fatigue") + u.girder_damage * mag.powi(3);
                    c.set_sub("fatigue", fat);
                    c.value = fat;
                    let corr = loss_um / (m.girder_corrosion_limit.max(0.01) * t0);
                    c.life_used = fat.max(corr);
                    c.margin = u.margins.overturn;
                }
                ComponentKind::Hook => {
                    c.value += u.moves;
                    c.life_used += u.hook_cycles_eq / (class_cycles * m.hook_cycle_factor.max(0.1));
                }
                ComponentKind::Coating => {
                    c.value += 1.0 / 365.0;
                    c.life_used += 1.0 / (365.0 * coat_years);
                }
                ComponentKind::RailClamp => {
                    c.value += u.clamp_ops;
                    let years = m.rail_clamp_years.max(0.5) * (m.corrosion_rate("C3").max(1.0) / rate_corr.max(1.0)).min(3.0);
                    c.life_used += u.clamp_ops / m.rail_clamp_ops.max(1.0) + 1.0 / (365.0 * years);
                    c.margin = u.margins.runaway;
                }
            }
            if let Some(v) = c.margin {
                c.margin_min = Some(c.margin_min.map(|x| x.min(v)).unwrap_or(v));
            }
            let d = (c.life_used - before).max(0.0);
            if operated || matches!(c.kind, ComponentKind::Coating | ComponentKind::RailClamp) {
                c.rate_per_day = if c.rate_per_day == 0.0 { d } else { 0.9 * c.rate_per_day + 0.1 * d };
            }
            c.rul_days = if c.rate_per_day > 1e-12 { Some(((m.replace_at - c.life_used) / c.rate_per_day).max(0.0)) } else { None };
            c.status = Self::status_of(c.life_used, m);
            if c.status < Health::Plan && c.rul_days.map(|d| d < m.plan_horizon_days).unwrap_or(false) && c.life_used >= 0.5 * m.watch_at {
                c.status = Health::Plan;
            }
            if c.status != old_status {
                changes.push(StatusChange { id: c.id.clone(), label: c.label.clone(), from: old_status, to: c.status, life_used: c.life_used, rul_days: c.rul_days });
            }
        }
        // 도장 수명이 다하면 거더 부식이 시작된다 (ISO 9224: d = r·t^b, t = 도장 파괴 뒤 년)
        let coat_life = self.component("coating").map(|c| c.life_used).unwrap_or(0.0);
        if coat_life > 1.0 {
            let t_exposed = (coat_life - 1.0) * coat_years;
            let loss = rate_corr * t_exposed.max(0.0).powf(m.corrosion_exponent.clamp(0.3, 1.0));
            if let Some(g) = self.component_mut("girder") {
                g.set_sub("corrosion_um", loss);
            }
        }
        self.day = day;
        self.operating_hours += u.op_hours;
        self.moves += u.moves;
        self.updated_unix = now_unix();
        changes
    }

    /// 부품 교체 — 수명 사용 비율과 물리량을 신품으로
    pub fn replace(&mut self, id: &str, spec: &CraneSpec, day: f64, hour: u8, why: &str) -> Result<(), String> {
        let c = self.components.iter_mut().find(|c| c.id == id).ok_or_else(|| format!("부품 '{id}' 이 없습니다"))?;
        let used = c.life_used;
        c.life_used = 0.0;
        c.rate_per_day = 0.0;
        c.rul_days = None;
        c.status = Health::Good;
        c.sub.clear();
        c.margin_min = c.margin;
        c.installed_day = day;
        c.replacements += 1;
        c.last_inspection_day = Some(day);
        c.value = match (c.kind, c.axis) {
            (ComponentKind::Brake, Some(a)) => spec.drive(a).brake.lining_new_mm,
            _ => 0.0,
        };
        let label = c.label.clone();
        self.events.push(HealthEvent { day, hour, kind: "replace".into(), component: Some(id.into()), detail: format!("{label} 교체 (수명 사용 {:.0}%) — {why}", used * 100.0) });
        Ok(())
    }

    pub fn inspect(&mut self, id: &str, day: f64, hour: u8, why: &str) -> Result<(), String> {
        let c = self.components.iter_mut().find(|c| c.id == id).ok_or_else(|| format!("부품 '{id}' 이 없습니다"))?;
        c.last_inspection_day = Some(day);
        let detail = format!("{} 점검 (수명 사용 {:.0}%, {}) — {why}", c.label, c.life_used * 100.0, c.status.label());
        self.events.push(HealthEvent { day, hour, kind: "inspect".into(), component: Some(id.into()), detail });
        Ok(())
    }

    pub fn note(&mut self, day: f64, hour: u8, kind: &str, component: Option<&str>, detail: String) {
        self.events.push(HealthEvent { day, hour, kind: kind.into(), component: component.map(String::from), detail });
    }

    pub fn snapshot(&mut self, doy: u32, risk_max: f64, alerts: u32, downtime_h: f64, day_moves: f64, day_hours: f64) {
        self.history.push(Snapshot {
            day: self.day,
            doy,
            op_hours: day_hours,
            moves: day_moves,
            life: self.components.iter().map(|c| c.life_used).collect(),
            margin: self.components.iter().map(|c| c.margin).collect(),
            risk_max,
            alerts,
            downtime_h,
        });
    }

    /// 가장 먼저 교체 기준에 닿을 부품들 (남은 날 오름차순)
    pub fn critical(&self, n: usize) -> Vec<&Component> {
        let mut v: Vec<&Component> = self.components.iter().collect();
        v.sort_by(|a, b| {
            let ka = (a.status != Health::Good, -(a.life_used));
            let kb = (b.status != Health::Good, -(b.life_used));
            kb.0.cmp(&ka.0).then(a.rul_days.unwrap_or(f64::MAX).partial_cmp(&b.rul_days.unwrap_or(f64::MAX)).unwrap_or(std::cmp::Ordering::Equal)).then(ka.1.partial_cmp(&kb.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        v.into_iter().take(n).collect()
    }
}

/// 내구도 평가 한 줄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurabilityRow {
    pub id: String,
    pub label: String,
    pub status: Health,
    pub status_label: String,
    pub life_used: f64,
    pub rul_days: Option<f64>,
    pub replacements: u32,
    /// 지금까지 교체 사이 평균 수명 (일) — 교체가 있었거나 지금 수명 사용량으로 외삽
    pub projected_life_days: Option<f64>,
    /// 1,000 이동당 수명 소비 (%)
    pub per_1000_moves_pct: Option<f64>,
    pub margin: Option<f64>,
    pub margin_min: Option<f64>,
    pub margin_label: String,
    pub basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurabilityReport {
    pub asset_id: String,
    pub crane: String,
    pub site: String,
    pub days: f64,
    pub operating_hours: f64,
    pub moves: f64,
    pub incidents: u32,
    pub rows: Vec<DurabilityRow>,
    /// 다음 교체 예정 (부품, 남은 날)
    pub next_due: Vec<(String, f64)>,
    pub notes: Vec<String>,
}

pub fn evaluate(l: &HealthLedger) -> DurabilityReport {
    let mut rows = Vec::new();
    for c in l.components.iter() {
        let age = (l.day - c.installed_day).max(0.0);
        let projected = if c.life_used > 1e-6 && age > 0.0 { Some(age / c.life_used) } else { None };
        let per_k = if l.moves > 0.0 && c.life_used > 0.0 {
            // 지금 부품이 겪은 이동 수 ≈ 전체 이동 × (부품 나이 / 전체 날)
            let moves_on = l.moves * (age / l.day.max(1e-9)).min(1.0);
            if moves_on > 0.0 {
                Some(c.life_used / moves_on * 1000.0 * 100.0)
            } else {
                None
            }
        } else {
            None
        };
        rows.push(DurabilityRow {
            id: c.id.clone(),
            label: c.label.clone(),
            status: c.status,
            status_label: c.status.label().into(),
            life_used: c.life_used,
            rul_days: c.rul_days,
            replacements: c.replacements,
            projected_life_days: projected,
            per_1000_moves_pct: per_k,
            margin: c.margin,
            margin_min: c.margin_min,
            margin_label: c.margin_label.clone(),
            basis: c.basis.clone(),
        });
    }
    let mut due: Vec<(String, f64)> = l.components.iter().filter_map(|c| c.rul_days.map(|d| (c.label.clone(), d))).collect();
    due.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    due.truncate(5);
    let mut notes = Vec::new();
    if l.day < 30.0 {
        notes.push("운영 기간이 30일 미만이라 남은 수명 추정의 불확실성이 큽니다".into());
    }
    DurabilityReport {
        asset_id: l.asset_id.clone(),
        crane: l.crane.clone(),
        site: l.site.clone(),
        days: l.day,
        operating_hours: l.operating_hours,
        moves: l.moves,
        incidents: l.incidents,
        rows,
        next_due: due,
        notes,
    }
}

/// 자산별 원장 파일 저장소 (`<dir>/<asset_id>.json`)
pub struct HealthStore {
    dir: PathBuf,
}

impl HealthStore {
    pub fn new(dir: &Path) -> Self {
        Self { dir: dir.to_path_buf() }
    }

    fn path_of(&self, asset: &str) -> PathBuf {
        let safe: String = asset.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect();
        self.dir.join(format!("{safe}.json"))
    }

    pub fn load(&self, asset: &str) -> Result<Option<HealthLedger>, String> {
        let p = self.path_of(asset);
        if !p.exists() {
            return Ok(None);
        }
        let txt = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let l: HealthLedger = serde_json::from_str(&txt).map_err(|e| format!("{}: {e}", p.display()))?;
        Ok(Some(l))
    }

    pub fn save(&self, l: &HealthLedger) -> Result<PathBuf, String> {
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))?;
        let p = self.path_of(&l.asset_id);
        let tmp = p.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string(l).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &p).map_err(|e| format!("{}: {e}", p.display()))?;
        Ok(p)
    }

    pub fn list(&self) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(&self.dir)
            .map(|rd| rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "json").unwrap_or(false)).filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned())).collect())
            .unwrap_or_default();
        v.sort();
        v
    }

    pub fn delete(&self, asset: &str) -> Result<(), String> {
        let p = self.path_of(asset);
        std::fs::remove_file(&p).map_err(|e| format!("{}: {e}", p.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site() -> SitePreset {
        crate::site::builtin().into_iter().find(|p| !p.environment.indoor).unwrap()
    }

    #[test]
    fn ledger_accumulates_status_and_replacement_resets() {
        let s = site();
        let spec = s.spec().unwrap();
        let m = DurabilityModel::default();
        let mut l = HealthLedger::new("T-1", &spec, &s);
        assert!(l.component("rail_clamp").is_some());
        let u = DayUsage { moves: 500.0, op_hours: 20.0, rope_damage: 0.02, brake_sets: [400.0, 400.0, 800.0], hook_cycles_eq: 100.0, flh_h: [1.0, 1.0, 2.0], ..Default::default() };
        let mut changes = Vec::new();
        for d in 1..=15 {
            changes.extend(l.apply_day(&spec, &s, &m, &u, d as f64));
        }
        let rope = l.component("rope_hoist").unwrap();
        assert!(rope.life_used > 0.3 && rope.life_used < 1.0, "{}", rope.life_used);
        assert!(rope.rul_days.unwrap() > 0.0);
        assert!(changes.iter().any(|c| c.id == "rope_hoist"));
        let lining_before = l.component("brake_hoist").unwrap().value;
        assert!(lining_before < spec.hoist.brake.lining_new_mm);
        l.replace("rope_hoist", &spec, 15.0, 8, "test").unwrap();
        let rope = l.component("rope_hoist").unwrap();
        assert_eq!(rope.life_used, 0.0);
        assert_eq!(rope.replacements, 1);
        assert_eq!(l.events.len(), 1);
        let rep = evaluate(&l);
        assert_eq!(rep.rows.len(), l.components.len());
    }

    #[test]
    fn coating_failure_starts_girder_corrosion_and_store_roundtrip() {
        let s = site();
        let spec = s.spec().unwrap();
        let m = DurabilityModel::default();
        let mut l = HealthLedger::new("T 2", &spec, &s);
        if let Some(c) = l.component_mut("coating") {
            c.life_used = 1.5;
        }
        l.apply_day(&spec, &s, &m, &DayUsage::default(), 1.0);
        assert!(l.component("girder").unwrap().sub_value("corrosion_um") > 0.0);
        let dir = std::env::temp_dir().join(format!("crane_health_{}", std::process::id()));
        let st = HealthStore::new(&dir);
        st.save(&l).unwrap();
        let back = st.load("T 2").unwrap().unwrap();
        assert_eq!(back.components.len(), l.components.len());
        assert_eq!(st.list(), vec!["T_2".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
