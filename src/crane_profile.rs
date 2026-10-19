//! 크레인 제원: 구조, 축별 구동(모터 · 인버터 · 감속기 · 바퀴/드럼 · 브레이크), 로프, 한계값, 설계 등급.
//!
//! 축 좌표 (모든 축을 '앞으로 = +' 인 하나의 일반 좌표 q 로 다룬다):
//! - Travel  (주행, 갠트리 전체)  q = x  (m)
//! - Trolley (횡행, 트롤리)       q = y  (m)
//! - Hoist   (권상)               q = −l (m) — 로프 길이 l 이 줄면(감아올리면) +
//!
//! 모터 각속도 ω_m = q̇ · kin,  kin = 감속비 × 줄 수 / 반경 (rad/m).
//! 프리셋 값은 동작을 보여 주기 위한 **예시 제원**이다. 실제 크레인 데이터로 바꿔 쓴다.

use crate::brake::BrakeParams;
use crate::motor::{self, MotorParams};
use crate::rope::{RopeFatigue, RopeParams};
use serde::{Deserialize, Serialize};

pub const G: f64 = 9.81;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    Travel,
    Trolley,
    Hoist,
}

impl Axis {
    pub fn all() -> [Axis; 3] {
        [Axis::Travel, Axis::Trolley, Axis::Hoist]
    }
    pub fn key(&self) -> &'static str {
        match self {
            Axis::Travel => "travel",
            Axis::Trolley => "trolley",
            Axis::Hoist => "hoist",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Axis::Travel => "주행",
            Axis::Trolley => "횡행",
            Axis::Hoist => "권상",
        }
    }
    pub fn index(&self) -> usize {
        match self {
            Axis::Travel => 0,
            Axis::Trolley => 1,
            Axis::Hoist => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GearParams {
    pub ratio: f64,
    pub efficiency: f64,
    /// 모터 축 기준 감속기 관성 (kg·m²)
    pub inertia_kgm2: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegenMode {
    /// 제동 저항으로 열 소모
    BrakeResistor,
    /// 전원 회생 (AFE)
    ActiveFrontEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InverterParams {
    /// 연속 전류 한계 / 모터 정격 전류
    pub continuous_factor: f64,
    /// 과부하 전류 한계 / 모터 정격 전류
    pub overload_factor: f64,
    /// 과부하 허용 시간 (s)
    pub overload_s: f64,
    pub efficiency: f64,
    pub regen: RegenMode,
}

impl Default for InverterParams {
    fn default() -> Self {
        Self { continuous_factor: 1.0, overload_factor: 1.5, overload_s: 60.0, efficiency: 0.97, regen: RegenMode::BrakeResistor }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedLoop {
    /// 속도 루프 대역폭 (rad/s)
    pub bandwidth_rad_s: f64,
    /// 적분 이득 = 대역폭² / ratio
    pub integral_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisDrive {
    pub axis: Axis,
    /// 이 축의 같은 모터 수 (주행은 양쪽 1대씩 = 2)
    pub motors: u32,
    pub motor: MotorParams,
    pub inverter: InverterParams,
    pub gear: GearParams,
    /// 바퀴 반경 (주행·횡행) 또는 드럼 반경 (권상) (m)
    pub radius_m: f64,
    /// 권상 줄 수 (주행·횡행은 1)
    pub reeving_falls: f64,
    pub brake: BrakeParams,
    pub max_speed_mps: f64,
    pub accel_mps2: f64,
    /// 저크 한계 (m/s³), 0 이면 저크 제한 없음
    pub jerk_mps3: f64,
    pub speed_loop: SpeedLoop,
    /// 양쪽 주행 모터의 토크 분담 차이 (그룹 구동, 0.02 = ±1%)
    #[serde(default)]
    pub side_mismatch: f64,
    /// 경하중 고속 배율과 그 하중 한계 (정격 대비). STAHL SH 인버터 권상: 정격 100 Hz, 하중 40% 이하에서 150 Hz 까지
    #[serde(default = "one")]
    pub light_load_speed_factor: f64,
    #[serde(default = "default_light_fraction")]
    pub light_load_fraction: f64,
}

fn one() -> f64 {
    1.0
}

fn default_light_fraction() -> f64 {
    0.4
}

impl AxisDrive {
    /// 하중쪽 1 m 당 모터 회전 (rad/m)
    pub fn kin(&self) -> f64 {
        self.gear.ratio * self.reeving_falls / self.radius_m.max(1e-9)
    }

    pub fn motor_speed_rad_s(&self, q_dot: f64) -> f64 {
        q_dot * self.kin()
    }

    /// 하중쪽 일반 좌표로 환산한 회전체 질량 (kg)
    pub fn reflected_mass_kg(&self) -> f64 {
        let k = self.kin();
        self.motors as f64 * (self.motor.inertia_kgm2 + self.gear.inertia_kgm2) * k * k
    }

    /// 모터 1대 토크 τ → 하중쪽 힘 (N), 전동이면 효율을 곱하고 회생이면 나눈다
    pub fn force_from_torque(&self, tau_per_motor: f64, q_dot: f64) -> f64 {
        let eta = self.gear.efficiency.clamp(0.3, 1.0);
        let raw = self.motors as f64 * tau_per_motor * self.kin();
        if tau_per_motor * q_dot >= 0.0 {
            raw * eta
        } else {
            raw / eta
        }
    }

    /// 하중쪽 힘 F 를 내는 모터 1대 토크
    pub fn torque_for_force(&self, force: f64, q_dot: f64) -> f64 {
        let eta = self.gear.efficiency.clamp(0.3, 1.0);
        let per = force / (self.motors as f64 * self.kin()).max(1e-9);
        if force * q_dot >= 0.0 {
            per / eta
        } else {
            per * eta
        }
    }

    /// 모터 정격 속도에 해당하는 하중쪽 속도 (m/s)
    pub fn rated_line_speed_mps(&self) -> f64 {
        self.motor.rated_speed_rad_s() / self.kin().max(1e-9)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.motor.validate()?;
        let checks = [
            ("motors", self.motors as f64),
            ("gear.ratio", self.gear.ratio),
            ("radius_m", self.radius_m),
            ("reeving_falls", self.reeving_falls),
            ("max_speed_mps", self.max_speed_mps),
            ("accel_mps2", self.accel_mps2),
            ("speed_loop.bandwidth_rad_s", self.speed_loop.bandwidth_rad_s),
            ("brake.holding_torque_nm", self.brake.holding_torque_nm),
        ];
        for (k, v) in checks {
            if !(v.is_finite() && v > 0.0) {
                return Err(format!("{} 축 {} 는 양수여야 합니다 ({})", self.axis.label(), k, v));
            }
        }
        let vmax_motor = self.motor.rated_speed_rad_s() * 2.0 / self.kin();
        if self.max_speed_mps * self.light_load_speed_factor.max(1.0) > vmax_motor {
            return Err(format!(
                "{} 축 최고 속도 {:.3} m/s 가 모터 2배 속도({:.3} m/s)를 넘습니다",
                self.axis.label(),
                self.max_speed_mps,
                vmax_motor
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureParams {
    /// 거더 스팬 = 횡행 범위 (m)
    pub span_m: f64,
    /// 주행 범위 (m)
    pub runway_m: f64,
    /// 로프 길이 범위 (m): 훅 최상단 / 최하단
    pub rope_min_m: f64,
    pub rope_max_m: f64,
    /// 트롤리 로프 걸이점 높이 (m, 뷰어용)
    pub suspension_height_m: f64,
    /// 움직이는 갠트리(거더·다리·새들) 질량, 트롤리 제외 (kg)
    pub bridge_mass_kg: f64,
    pub trolley_mass_kg: f64,
    pub hook_mass_kg: f64,
    /// 정격 하중 SWL (kg, 훅 제외)
    pub rated_load_kg: f64,
    /// 구름 저항 계수 (주행 저항 / 바퀴 하중)
    pub rolling_resistance: f64,
    /// 점성 저항 (N·s/m): 주행, 횡행
    pub viscous_travel: f64,
    pub viscous_trolley: f64,
    /// 거더 수직 1차 고유진동수 (Hz)
    pub girder_fn_hz: f64,
    /// 편행 안내 강성 (N/m, 플랜지 접촉) · 접촉 전 틈 (m)
    pub skew_stiffness_n_per_m: f64,
    pub skew_gap_m: f64,
    /// 편행 회전 감쇠 (N·m·s/rad)
    pub skew_damping: f64,
    /// 갠트리 구조 풍압 면적 × 항력 계수 (m²)
    pub bridge_cd_area_m2: f64,
    /// 하중 흔들림 등가 감쇠비 (로프 내부 마찰 등)
    pub sway_damping_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    /// 정격하중 대비 경고 비율 (Schneider Overload_EN15011: 90% 경고) / 차단 비율 (Demag: 10% 이상 과부하 차단)
    pub overload_warn: f64,
    pub overload_trip: f64,
    /// 권선 온도 경고 여유 (절연 한계 − margin)
    pub winding_margin_k: f64,
    /// 잔류 흔들림 허용 (하중 수평 변위, mm)
    pub sway_tolerance_mm: f64,
    /// 위치 결정 허용 오차 (mm)
    pub position_tolerance_mm: f64,
    /// 운전 허용 풍속 (m/s). 기본 20 m/s = EN 13001-2:2021 표 5 '보통' 운전 풍 상태의 3초 돌풍
    /// (평균 13.3 m/s, 250 N/m²). 크레인 설계 자료의 값으로 교체한다.
    pub wind_in_service_mps: f64,
    /// 편행 한계 (mm)
    pub skew_limit_mm: f64,
    /// 권상 브레이크 토크 / 정격하중 정적 토크 최소 비
    pub hoist_brake_factor_min: f64,
    /// 로프 손상 경고 (Miner 합)
    pub rope_damage_warn: f64,
    /// 브레이크 라이닝 남은 비율: 정비 계획 / 점검 (Konecranes 브레이크 모니터링: 공극 여유 30% · 15%)
    pub brake_lining_warn: f64,
    pub brake_lining_inspect: f64,
    /// 남은 설계 수명(SWP) 비율: 경고 / 종합 정비 (Demag StatusControl 30% · 10%, Konecranes 종합 정비 10%)
    pub swp_warn_remaining: f64,
    pub swp_overhaul_remaining: f64,
    /// 구조 공진 판단 대역 (±비율)
    pub resonance_band: f64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            overload_warn: 0.9,
            overload_trip: 1.1,
            winding_margin_k: 10.0,
            sway_tolerance_mm: 50.0,
            position_tolerance_mm: 50.0,
            wind_in_service_mps: 20.0,
            skew_limit_mm: 25.0,
            hoist_brake_factor_min: 1.6,
            rope_damage_warn: 0.8,
            brake_lining_warn: 0.30,
            brake_lining_inspect: 0.15,
            swp_warn_remaining: 0.30,
            swp_overhaul_remaining: 0.10,
            resonance_band: 0.15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignClass {
    /// 권상 기구 등급 (예: M5)
    pub mechanism_class: String,
    /// 기구 등급의 설계 수명 (전부하 환산 시간, h)
    pub design_full_load_hours: f64,
    /// 크레인 사용 등급 설계 작업 사이클 수 (U 등급 상한)
    pub design_cycles: f64,
}

impl Default for DesignClass {
    fn default() -> Self {
        // ISO 4301-1:2016 Annex B: M5 ↔ 1,600 전부하 시간, U5 = 2.5×10⁵ ~ 5×10⁵ 사이클
        Self { mechanism_class: "M5".into(), design_full_load_hours: 1600.0, design_cycles: 5.0e5 }
    }
}

/// 기구 등급별 설계 수명 (전부하 시간): M1–M8 = 100 · 200 · 400 · 800 · 1,600 · 3,200 · 6,300 · 12,500
/// (ISO 4301-1:2016 Annex B · ISO 12482:2014 D_M, FEM 9.512 3m/4m/5m = 3,200/6,300/12,500)
pub fn mechanism_design_hours(class: &str) -> Option<f64> {
    let h = [100.0, 200.0, 400.0, 800.0, 1600.0, 3200.0, 6300.0, 12500.0];
    crate::rope_data::mechanism_index(class).map(|i| h[i])
}

/// FEM 듀티 표의 기구 등급별 한계 (최대 기동 회/h, 최대 사이클 회/h, 운전율) — M3–M8
/// 운전율은 1Bm/1Am/2m/3m (M3–M6) 만 25/30/40/50% 로 주어진다.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FemDutyLimits {
    pub max_starts_per_hour: f64,
    pub max_cycles_per_hour: f64,
    pub cdf: Option<f64>,
}

pub fn fem_duty_limits(class: &str) -> Option<FemDutyLimits> {
    let i = crate::rope_data::mechanism_index(class)?;
    if i < 2 {
        return None;
    }
    let starts = [150.0, 180.0, 240.0, 300.0, 360.0, 360.0];
    let cycles = [25.0, 30.0, 40.0, 50.0, 60.0, 60.0];
    let cdf = [Some(0.25), Some(0.30), Some(0.40), Some(0.50), None, None];
    let k = i - 2;
    Some(FemDutyLimits { max_starts_per_hour: starts[k], max_cycles_per_hour: cycles[k], cdf: cdf[k] })
}

impl DesignClass {
    /// 기구 등급 이름으로 설계 수명을 채운다
    pub fn for_mechanism(class: &str, design_cycles: f64) -> Option<Self> {
        Some(Self { mechanism_class: class.to_string(), design_full_load_hours: mechanism_design_hours(class)?, design_cycles })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraneSpec {
    pub name: String,
    pub description: String,
    pub structure: StructureParams,
    pub travel: AxisDrive,
    pub trolley: AxisDrive,
    pub hoist: AxisDrive,
    pub rope: RopeParams,
    #[serde(default)]
    pub limits: Limits,
    #[serde(default)]
    pub class: DesignClass,
}

impl CraneSpec {
    pub fn drive(&self, axis: Axis) -> &AxisDrive {
        match axis {
            Axis::Travel => &self.travel,
            Axis::Trolley => &self.trolley,
            Axis::Hoist => &self.hoist,
        }
    }

    pub fn drive_mut(&mut self, axis: Axis) -> &mut AxisDrive {
        match axis {
            Axis::Travel => &mut self.travel,
            Axis::Trolley => &mut self.trolley,
            Axis::Hoist => &mut self.hoist,
        }
    }

    /// 정격 하중을 들고 정지했을 때 권상 모터 1대의 정적 토크 (Nm)
    pub fn hoist_static_torque_nm(&self, load_kg: f64) -> f64 {
        let w = (load_kg + self.structure.hook_mass_kg) * G;
        self.hoist.torque_for_force(w, 0.0)
    }

    pub fn validate(&self) -> Result<(), String> {
        for a in Axis::all() {
            self.drive(a).validate()?;
            if self.drive(a).axis != a {
                return Err(format!("{} 구동의 axis 필드가 {:?} 입니다", a.label(), self.drive(a).axis));
            }
        }
        let s = &self.structure;
        if !(s.rope_min_m > 0.0 && s.rope_max_m > s.rope_min_m) {
            return Err("로프 길이 범위가 잘못되었습니다".into());
        }
        if s.rated_load_kg <= 0.0 || s.bridge_mass_kg <= 0.0 || s.trolley_mass_kg <= 0.0 {
            return Err("질량은 양수여야 합니다".into());
        }
        if self.rope.diameter_mm <= 0.0 || self.rope.min_breaking_force_kn <= 0.0 {
            return Err("로프 제원이 잘못되었습니다".into());
        }
        Ok(())
    }
}

fn drive(
    axis: Axis,
    motors: u32,
    m: MotorParams,
    ratio: f64,
    eta: f64,
    radius: f64,
    falls: f64,
    vmax: f64,
    acc: f64,
    jerk: f64,
    bw: f64,
    brake_factor: f64,
) -> AxisDrive {
    let tn = m.rated_torque_nm();
    let gear_j = 0.2 * m.inertia_kgm2;
    AxisDrive {
        axis,
        motors,
        brake: BrakeParams::example(tn, brake_factor),
        motor: m,
        inverter: InverterParams::default(),
        gear: GearParams { ratio, efficiency: eta, inertia_kgm2: gear_j },
        radius_m: radius,
        reeving_falls: falls,
        max_speed_mps: vmax,
        accel_mps2: acc,
        jerk_mps3: jerk,
        speed_loop: SpeedLoop { bandwidth_rad_s: bw, integral_ratio: 4.0 },
        side_mismatch: if axis == Axis::Travel { 0.02 } else { 0.0 },
        light_load_speed_factor: if axis == Axis::Hoist { 1.5 } else { 1.0 },
        light_load_fraction: 0.4,
    }
}

/// 감속비: 모터 정격 속도에서 하중쪽 최고 속도가 나오도록
fn ratio_for(m: &MotorParams, radius: f64, falls: f64, vmax: f64) -> f64 {
    m.rated_speed_rad_s() * radius / (falls * vmax)
}

fn example_rope(d_mm: f64, mbf_kn: f64, sheave_mm: f64, drum_mm: f64) -> RopeParams {
    RopeParams {
        construction: "예시: 최소 파단력은 6×36WS-IWRC 1960 규모, 굽힘 피로는 8×19 Seale FC 상수 — 실제 로프 제조사 값으로 교체".into(),
        diameter_mm: d_mm,
        min_breaking_force_kn: mbf_kn,
        tensile_grade_mpa: 1960.0,
        sheave_diameter_mm: sheave_mm,
        drum_diameter_mm: drum_mm,
        sheave_count: 2,
        mass_kg_per_m: 0.004 * d_mm * d_mm,
        // EA ≈ E_rope(1.0e5 N/mm²) × 금속 단면적(0.5·d²)
        axial_stiffness_ea_n: 1.0e5 * 0.5 * d_mm * d_mm,
        // ISO 16625 M5 단층 권상 Zp
        min_safety_factor: crate::rope_data::iso16625_zp("M5").unwrap_or(4.5),
        fatigue: crate::rope_data::default_fatigue(),
    }
}

/// 옥외 갠트리 크레인 10 t × 스팬 20 m (예시)
pub fn gantry_10t_20m() -> CraneSpec {
    let mt = motor::preset("im_4kw_4p").unwrap();
    let mc = motor::preset("im_1p5kw_4p").unwrap();
    let mh = motor::preset("im_11kw_4p").unwrap();
    let (vt, vc, vh) = (40.0 / 60.0, 20.0 / 60.0, 5.0 / 60.0);
    let it = ratio_for(&mt, 0.2, 1.0, vt);
    let ic = ratio_for(&mc, 0.125, 1.0, vc);
    let ih = ratio_for(&mh, 0.2, 4.0, vh);
    CraneSpec {
        name: "gantry_10t_20m".into(),
        description: "옥외 갠트리 크레인 10 t, 스팬 20 m, 양정 10 m (예시 제원)".into(),
        structure: StructureParams {
            span_m: 20.0,
            runway_m: 60.0,
            rope_min_m: 2.0,
            rope_max_m: 12.0,
            suspension_height_m: 14.0,
            bridge_mass_kg: 18_000.0,
            trolley_mass_kg: 3_000.0,
            hook_mass_kg: 200.0,
            rated_load_kg: 10_000.0,
            rolling_resistance: 0.006,
            viscous_travel: 200.0,
            viscous_trolley: 60.0,
            girder_fn_hz: 2.5,
            skew_stiffness_n_per_m: 2.0e6,
            skew_gap_m: 0.01,
            skew_damping: 2.0e5,
            bridge_cd_area_m2: 1.6 * 35.0,
            // 산업용 30 t 크레인 훅 감쇠비 약 0.01 (Sorensen et al., IFAC 2008)
            sway_damping_ratio: 0.01,
        },
        travel: drive(Axis::Travel, 2, mt, it, 0.95, 0.2, 1.0, vt, 0.2, 0.5, 6.0, 1.2),
        trolley: drive(Axis::Trolley, 1, mc, ic, 0.95, 0.125, 1.0, vc, 0.2, 0.5, 8.0, 1.2),
        hoist: drive(Axis::Hoist, 1, mh, ih, 0.90, 0.2, 4.0, vh, 0.1, 0.3, 12.0, 2.0),
        rope: example_rope(16.0, 178.0, 320.0, 400.0),
        limits: Limits::default(),
        class: DesignClass::default(),
    }
}

/// 실내 천장 크레인 5 t × 스팬 16 m (예시)
pub fn overhead_5t_16m() -> CraneSpec {
    let mt = motor::preset("im_1p5kw_4p").unwrap();
    let mc = motor::preset("im_0p75kw_4p").unwrap();
    let mh = motor::preset("im_7p5kw_4p").unwrap();
    let (vt, vc, vh) = (30.0 / 60.0, 20.0 / 60.0, 6.0 / 60.0);
    let it = ratio_for(&mt, 0.16, 1.0, vt);
    let ic = ratio_for(&mc, 0.1, 1.0, vc);
    let ih = ratio_for(&mh, 0.15, 4.0, vh);
    let mut limits = Limits::default();
    limits.wind_in_service_mps = 0.0; // 실내: 바람 게이트 없음
    CraneSpec {
        name: "overhead_5t_16m".into(),
        description: "실내 천장 크레인(EOT) 5 t, 스팬 16 m, 양정 6.5 m (예시 제원)".into(),
        structure: StructureParams {
            span_m: 16.0,
            runway_m: 40.0,
            rope_min_m: 1.5,
            rope_max_m: 8.0,
            suspension_height_m: 9.0,
            bridge_mass_kg: 8_000.0,
            trolley_mass_kg: 1_200.0,
            hook_mass_kg: 100.0,
            rated_load_kg: 5_000.0,
            rolling_resistance: 0.006,
            viscous_travel: 120.0,
            viscous_trolley: 40.0,
            girder_fn_hz: 4.0,
            skew_stiffness_n_per_m: 1.5e6,
            skew_gap_m: 0.008,
            skew_damping: 8.0e4,
            bridge_cd_area_m2: 0.0,
            // 산업용 30 t 크레인 훅 감쇠비 약 0.01 (Sorensen et al., IFAC 2008)
            sway_damping_ratio: 0.01,
        },
        travel: drive(Axis::Travel, 2, mt, it, 0.95, 0.16, 1.0, vt, 0.25, 0.6, 6.0, 1.2),
        trolley: drive(Axis::Trolley, 1, mc, ic, 0.95, 0.1, 1.0, vc, 0.2, 0.5, 8.0, 1.2),
        hoist: drive(Axis::Hoist, 1, mh, ih, 0.90, 0.15, 4.0, vh, 0.12, 0.3, 12.0, 2.0),
        rope: example_rope(12.0, 100.0, 240.0, 300.0),
        limits,
        class: DesignClass::default(),
    }
}

pub fn presets() -> Vec<CraneSpec> {
    vec![gantry_10t_20m(), overhead_5t_16m()]
}

pub fn preset(name: &str) -> Option<CraneSpec> {
    presets().into_iter().find(|c| c.name == name)
}

/// JSON 에서 프리셋 이름 또는 전체 제원으로 크레인을 지정한다
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CraneRef {
    Preset(String),
    Spec(Box<CraneSpec>),
}

impl CraneRef {
    pub fn resolve(&self) -> Result<CraneSpec, String> {
        match self {
            CraneRef::Preset(n) => preset(n).ok_or_else(|| {
                let names: Vec<String> = presets().into_iter().map(|c| c.name).collect();
                format!("크레인 프리셋 '{}' 이 없습니다 (사용 가능: {})", n, names.join(", "))
            }),
            CraneRef::Spec(s) => {
                s.validate()?;
                Ok((**s).clone())
            }
        }
    }
}

#[allow(dead_code)]
fn _fatigue_type_check(_: &RopeFatigue) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_validate() {
        for c in presets() {
            c.validate().unwrap();
        }
    }

    #[test]
    fn max_speed_maps_to_rated_motor_speed() {
        let c = gantry_10t_20m();
        for a in Axis::all() {
            let d = c.drive(a);
            let w = d.motor_speed_rad_s(d.max_speed_mps);
            assert!((w - d.motor.rated_speed_rad_s()).abs() < 1e-9);
        }
    }

    #[test]
    fn hoist_holds_rated_load_below_rated_torque() {
        for c in presets() {
            let t = c.hoist_static_torque_nm(c.structure.rated_load_kg);
            let tn = c.hoist.motor.rated_torque_nm();
            assert!(t < tn, "{}: {t} vs {tn}", c.name);
            assert!(t > 0.6 * tn, "{}: hoist motor oversized {t} vs {tn}", c.name);
            // 브레이크 토크 여유
            assert!(c.hoist.brake.holding_torque_nm / t >= c.limits.hoist_brake_factor_min);
        }
    }

    #[test]
    fn force_torque_roundtrip() {
        let c = gantry_10t_20m();
        for v in [-0.5, 0.5] {
            for f in [-1000.0, 1000.0] {
                let t = c.travel.torque_for_force(f, v);
                let back = c.travel.force_from_torque(t, v);
                assert!((back - f).abs() < 1e-6, "{f} {v} {back}");
            }
        }
    }

    #[test]
    fn class_tables() {
        assert_eq!(mechanism_design_hours("M5"), Some(1600.0));
        assert_eq!(mechanism_design_hours("M8"), Some(12500.0));
        let l = fem_duty_limits("M5").unwrap();
        assert_eq!((l.max_starts_per_hour, l.max_cycles_per_hour, l.cdf), (240.0, 40.0, Some(0.40)));
        assert!(fem_duty_limits("M8").unwrap().cdf.is_none());
        assert!(fem_duty_limits("M2").is_none());
    }

    #[test]
    fn crane_ref_parses_preset_and_spec() {
        let r: CraneRef = serde_json::from_str("\"gantry_10t_20m\"").unwrap();
        assert_eq!(r.resolve().unwrap().name, "gantry_10t_20m");
        let full = serde_json::to_string(&overhead_5t_16m()).unwrap();
        let r2: CraneRef = serde_json::from_str(&full).unwrap();
        assert_eq!(r2.resolve().unwrap().name, "overhead_5t_16m");
    }
}
