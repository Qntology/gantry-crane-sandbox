//! 운반 사이클 시뮬레이션.
//!
//! 경유점 목록을 따라 축별 속도 기준 프로파일(사다리꼴 → 저크 평활 → 입력 셰이핑)을 만들고,
//! 구동 속도 제어(PI + 앞먹임) · 인버터 전류 한계 · 브레이크 시퀀스 · 본체 동역학 · 모터 전기/열 ·
//! 로프 굽힘 행정 · 브레이크 마찰일을 한 시간축에서 함께 적분한다.
//!
//! 엔드밀 `loadsim.rs` 의 자리: 공구경로 대신 경유점, 절삭 부하 대신 하중 흔들림 · 모터 부하를 기록하고
//! G-code 대신 축별 속도 프로파일(`MoveProgram`)을 내보낸다.

use crate::antisway::{self, AntiSwayMode, Shaper};
use crate::brake::{BrakePhase, BrakeState};
use crate::crane_dynamics::{Aux, Inputs, PhysState, Plant, IVL, IVPSI, IVX, IVY};
use crate::crane_profile::{Axis, CraneRef, CraneSpec, RegenMode, G};
use crate::environment::Environment;
use crate::motion::{self, Profile, ProfileLimits};
use crate::motor::{MotorLosses, MotorThermal};
use crate::rope::RopeCounter;
use serde::{Deserialize, Serialize};

fn one() -> f64 {
    1.0
}
fn default_vtol() -> f64 {
    0.05
}
fn default_q() -> [f64; 4] {
    [10.0, 1.0, 1000.0, 100.0]
}
fn default_mode() -> MoveMode {
    MoveMode::Sequential
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveMode {
    /// 권상(올림) → 주행·횡행 → 권상(내림)
    Sequential,
    /// 세 축 동시
    Simultaneous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    /// 로프 길이 목표 (m)
    #[serde(default)]
    pub l: Option<f64>,
    #[serde(default = "default_mode")]
    pub mode: MoveMode,
    /// 도착 · 안정 후 머무는 시간 (s)
    #[serde(default)]
    pub dwell_s: f64,
    /// 도착 후 하중 질량을 이 값으로 바꾼다 (걸기 = 질량, 풀기 = 0)
    #[serde(default)]
    pub set_load_kg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiSwayConfig {
    pub mode: AntiSwayMode,
    /// 셰이퍼 설계 감쇠비
    #[serde(default)]
    pub zeta: f64,
    /// 설계 진동수 오차 (0.1 = 실제보다 10% 높게 설계 — 강건성 실험)
    #[serde(default)]
    pub freq_error: f64,
    #[serde(default = "default_vtol")]
    pub ei_vtol: f64,
    #[serde(default = "default_q")]
    pub lqr_q: [f64; 4],
    #[serde(default = "one")]
    pub lqr_r: f64,
}

impl Default for AntiSwayConfig {
    fn default() -> Self {
        Self { mode: AntiSwayMode::Off, zeta: 0.0, freq_error: 0.0, ei_vtol: default_vtol(), lqr_q: default_q(), lqr_r: 1.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub dt: f64,
    pub sample_dt: f64,
    /// 이동 끝 뒤 잔류 흔들림을 재는 시간 (s)
    pub settle_s: f64,
    /// 속도 0 이 이 시간 유지되면 브레이크 체결 (s)
    pub hold_before_brake_s: f64,
    /// 한 단계 최대 시간 (s)
    pub max_phase_s: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self { dt: 0.002, sample_dt: 0.02, settle_s: 4.0, hold_before_brake_s: 0.3, max_phase_s: 900.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartState {
    pub x: f64,
    pub y: f64,
    pub l: f64,
    pub load_kg: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadShape {
    /// 하중 항력 계수 × 정면 면적 (m²). 비우면 EN 13001-2:2021 의 매단 하중 기본식
    /// c = 2.4, A = 0.0005·m_H (m_H: 하중 + 훅 kg, A ≥ 0.8 m²) 을 쓴다.
    #[serde(default)]
    pub cd_area_m2: Option<f64>,
}

impl LoadShape {
    pub fn cd_area(&self, hoisted_mass_kg: f64) -> f64 {
        self.cd_area_m2.unwrap_or_else(|| 2.4 * (0.0005 * hoisted_mass_kg.max(0.0)).max(0.8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// 안전 토크 차단(STO) + 모든 브레이크 즉시 체결 명령
    EmergencyStop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub t_s: f64,
    pub kind: EventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub crane: CraneRef,
    pub start: StartState,
    pub waypoints: Vec<Waypoint>,
    #[serde(default)]
    pub antisway: AntiSwayConfig,
    #[serde(default)]
    pub env: Environment,
    #[serde(default)]
    pub sim: SimConfig,
    #[serde(default)]
    pub load: LoadShape,
    #[serde(default = "one")]
    pub speed_scale: f64,
    #[serde(default = "one")]
    pub accel_scale: f64,
    #[serde(default)]
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AxisSamples {
    pub v_ref: Vec<f64>,
    pub v: Vec<f64>,
    pub rpm: Vec<f64>,
    pub torque_nm: Vec<f64>,
    pub current_a: Vec<f64>,
    pub power_kw: Vec<f64>,
    pub winding_c: Vec<f64>,
    /// 0 해제 · 1 체결 중 · 2 체결 · 3 해제 중
    pub brake: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Samples {
    pub t: Vec<f64>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub l: Vec<f64>,
    pub theta_x_deg: Vec<f64>,
    pub theta_y_deg: Vec<f64>,
    pub sway_mm: Vec<f64>,
    pub load_x: Vec<f64>,
    pub load_y: Vec<f64>,
    pub load_z: Vec<f64>,
    pub tension_kn: Vec<f64>,
    pub wind_mps: Vec<f64>,
    pub skew_mm: Vec<f64>,
    pub load_kg: Vec<f64>,
    /// [주행, 횡행, 권상]
    pub axes: Vec<AxisSamples>,
}

impl Samples {
    pub fn len(&self) -> usize {
        self.t.len()
    }
    pub fn is_empty(&self) -> bool {
        self.t.is_empty()
    }

    pub fn to_csv(&self) -> String {
        let mut h = vec![
            "t_s", "x_m", "y_m", "l_m", "theta_x_deg", "theta_y_deg", "sway_mm", "load_x_m", "load_y_m", "load_z_m", "tension_kn", "wind_mps", "skew_mm", "load_kg",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        for a in Axis::all() {
            for c in ["v_ref_mps", "v_mps", "rpm", "torque_nm", "current_a", "power_kw", "winding_c", "brake"] {
                h.push(format!("{}_{}", a.key(), c));
            }
        }
        let mut out = h.join(",");
        out.push('\n');
        for i in 0..self.len() {
            let mut row: Vec<String> = [
                self.t[i], self.x[i], self.y[i], self.l[i], self.theta_x_deg[i], self.theta_y_deg[i], self.sway_mm[i], self.load_x[i], self.load_y[i], self.load_z[i],
                self.tension_kn[i], self.wind_mps[i], self.skew_mm[i], self.load_kg[i],
            ]
            .iter()
            .map(|v| format!("{:.5}", v))
            .collect();
            for a in self.axes.iter() {
                for v in [a.v_ref[i], a.v[i], a.rpm[i], a.torque_nm[i], a.current_a[i], a.power_kw[i], a.winding_c[i]] {
                    row.push(format!("{:.5}", v));
                }
                row.push(a.brake[i].to_string());
            }
            out.push_str(&row.join(","));
            out.push('\n');
        }
        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisProgram {
    pub axis: Axis,
    pub distance_m: f64,
    pub t_start_s: f64,
    pub duration_s: f64,
    pub v_max_mps: f64,
    pub a_max_mps2: f64,
    pub jerk_mps3: f64,
    pub shaper: Option<Shaper>,
    /// 기준 속도 (sample_dt 간격)
    pub v_ref: Vec<f64>,
    pub sample_dt: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveProgram {
    pub label: String,
    pub mode: MoveMode,
    pub axes: Vec<AxisProgram>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveMetrics {
    pub label: String,
    pub start_s: f64,
    /// 기준 프로파일이 끝난 시각
    pub end_s: f64,
    pub settle_end_s: f64,
    pub duration_s: f64,
    pub target: [f64; 3],
    pub rope_length_m: f64,
    pub shaper: Option<String>,
    pub shaper_freq_hz: Option<f64>,
    pub max_sway_mm: f64,
    pub max_sway_deg: f64,
    /// 기준 프로파일이 끝난 뒤 안정 구간 전체의 최대 흔들림 (하중 수평 변위, mm)
    pub residual_sway_mm: f64,
    /// 안정 구간 마지막 반 주기의 최대 흔들림 (mm) — 폐루프처럼 끝난 뒤에도 감쇠시키는 방식을 비교할 때
    pub final_sway_mm: f64,
    /// 이동 끝부터 흔들림이 허용값 안으로 들어올 때까지 (s). 안정 구간 안에 못 들어오면 안정 구간 길이
    pub settle_time_s: f64,
    pub position_error_mm: f64,
    pub load_kg: f64,
    /// 비상 정지로 중단된 이동 (위치 · 흔들림 판정에서 뺀다)
    #[serde(default)]
    pub aborted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisMetrics {
    pub axis: Axis,
    pub motors: u32,
    pub motor: String,
    pub rated_torque_nm: f64,
    pub peak_torque_nm: f64,
    /// 최대 |토크| / 그때의 토크 한계
    pub peak_torque_ratio: f64,
    pub torque_saturated_s: f64,
    pub rated_current_a: f64,
    pub peak_current_a: f64,
    /// 사이클 전체 RMS 전류 (모터 1대)
    pub rms_current_a: f64,
    pub inverter_overload_peak_s: f64,
    pub energy_in_kwh: f64,
    pub energy_regen_kwh: f64,
    pub regen_to_resistor: bool,
    pub peak_winding_c: f64,
    pub end_winding_c: f64,
    pub insulation_limit_c: f64,
    pub run_time_s: f64,
    pub starts: u32,
    /// 사이클 운전율 (통전 시간 / 사이클 시간)
    pub cdf: f64,
    pub starts_per_hour: f64,
    pub rated_cdf: f64,
    pub rated_starts_per_hour: f64,
    pub brake_lining_mm: f64,
    pub brake_lining_remaining: f64,
    pub brake_work_j: f64,
    pub brake_max_stop_work_j: f64,
    pub brake_max_work_allowed_j: f64,
    pub brake_max_temp_c: f64,
    pub brake_temp_limit_c: f64,
    pub brake_operations: u64,
    pub brake_dynamic_stops: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RopeMetrics {
    pub max_tension_per_fall_kn: f64,
    pub min_safety_factor: f64,
    pub required_safety_factor: f64,
    pub d_ratio_sheave: f64,
    pub strokes: u64,
    pub damage: f64,
    pub fatigue_source: String,
}

/// 듀티 시뮬레이션용 손실 기록 (대표 모터 1대)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AxisLossProfile {
    pub axis: Option<Axis>,
    pub dt: f64,
    pub copper_w: Vec<f64>,
    pub other_w: Vec<f64>,
    pub omega: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleReport {
    pub scenario: String,
    pub crane: String,
    pub antisway: String,
    pub rated_load_kg: f64,
    pub load_peak_kg: f64,
    pub cycle_time_s: f64,
    pub moves: Vec<MoveMetrics>,
    pub axes: Vec<AxisMetrics>,
    pub rope: RopeMetrics,
    /// 권상 브레이크 토크 / 최대 하중 정적 토크
    pub hoist_brake_factor: f64,
    pub max_skew_mm: f64,
    pub max_wind_mps: f64,
    pub slack_events: u32,
    pub estop: bool,
    pub notes: Vec<String>,
    pub program: Vec<MoveProgram>,
    pub samples: Samples,
    /// 듀티 열 적분용 내부 기록 (내보내지 않는다)
    #[serde(skip_serializing, default)]
    pub loss_profiles: Vec<AxisLossProfile>,
}

#[derive(Debug, Clone, Default)]
struct AxisStats {
    peak_tau: f64,
    peak_tau_ratio: f64,
    sat_time: f64,
    peak_current: f64,
    i2t: f64,
    energy_in_j: f64,
    energy_regen_j: f64,
    run_time: f64,
    starts: u32,
    was_energized: bool,
    peak_winding: f64,
    overload_peak: f64,
}

#[derive(Debug, Clone)]
struct AxisRt {
    axis: Axis,
    profile: Option<Profile>,
    /// 기준 프로파일의 누적 거리 (폐루프 위치 기준용)
    cum: Vec<f64>,
    t0: f64,
    q0: f64,
    q_target: f64,
    integ: f64,
    tau: [f64; 2],
    brake: BrakeState,
    thermal: Vec<MotorThermal>,
    overload_s: f64,
    hold_timer: f64,
    keep_active: bool,
    v_cmd: f64,
    lqr_k: [f64; 4],
    last_v_ref: f64,
    last_current: f64,
    last_power_w: f64,
    last_losses: MotorLosses,
    energized: bool,
    stats: AxisStats,
}

#[derive(Debug, Clone, Default)]
struct MoveTracker {
    active: bool,
    settling: bool,
    settle_start: f64,
    final_start: f64,
    max_sway_m: f64,
    max_sway_rad: f64,
    residual_m: f64,
    final_m: f64,
    last_above_tol: f64,
}

/// 사이클 사이에 이어지는 상태 (듀티 연속 실행용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCarry {
    pub thermal: Vec<Vec<MotorThermal>>,
    pub brakes: Vec<BrakeState>,
    pub rope: RopeCounter,
}

pub struct Sim<'a> {
    spec: CraneSpec,
    sc: &'a Scenario,
    st: PhysState,
    t: f64,
    load_kg: f64,
    load_peak_kg: f64,
    axes: Vec<AxisRt>,
    rope: RopeCounter,
    estop: bool,
    estop_events: Vec<f64>,
    samples: Samples,
    next_sample_t: f64,
    last_aux: Aux,
    max_skew_m: f64,
    max_wind: f64,
    slack_events: u32,
    slack_prev: bool,
    moves: Vec<MoveMetrics>,
    program: Vec<MoveProgram>,
    losses: Vec<AxisLossProfile>,
    tracker: MoveTracker,
    notes: Vec<String>,
}

impl<'a> Sim<'a> {
    pub fn new(sc: &'a Scenario, carry: Option<&CycleCarry>) -> Result<Self, String> {
        let spec = sc.crane.resolve()?;
        if sc.sim.dt <= 0.0 || sc.sim.dt > 0.02 {
            return Err(format!("sim.dt 는 0 ~ 0.02 s 여야 합니다 ({})", sc.sim.dt));
        }
        if sc.sim.sample_dt < sc.sim.dt {
            return Err("sim.sample_dt 는 sim.dt 이상이어야 합니다".into());
        }
        let hook_kg = spec.structure.hook_mass_kg;
        let (rmin, rmax, runway, span) = (spec.structure.rope_min_m, spec.structure.rope_max_m, spec.structure.runway_m, spec.structure.span_m);
        let l0 = sc.start.l.clamp(rmin, rmax);
        let st = PhysState::hanging(sc.start.x.clamp(0.0, runway), sc.start.y.clamp(0.0, span), l0);
        let amb = sc.env.ambient_c;
        let axes = Axis::all()
            .iter()
            .enumerate()
            .map(|(k, a)| {
                let d = spec.drive(*a);
                let thermal = carry
                    .and_then(|c| c.thermal.get(k).cloned())
                    .filter(|v| v.len() == d.motors as usize)
                    .unwrap_or_else(|| vec![MotorThermal::at_ambient(amb); d.motors as usize]);
                let brake = carry.and_then(|c| c.brakes.get(k).cloned()).unwrap_or_else(|| BrakeState::new(&d.brake, amb));
                AxisRt {
                    axis: *a,
                    profile: None,
                    cum: Vec::new(),
                    t0: 0.0,
                    q0: 0.0,
                    q_target: 0.0,
                    integ: 0.0,
                    tau: [0.0; 2],
                    brake,
                    thermal,
                    overload_s: 0.0,
                    hold_timer: 0.0,
                    keep_active: false,
                    v_cmd: 0.0,
                    lqr_k: [0.0; 4],
                    last_v_ref: 0.0,
                    last_current: 0.0,
                    last_power_w: 0.0,
                    last_losses: MotorLosses::default(),
                    energized: false,
                    stats: AxisStats { peak_winding: amb, ..Default::default() },
                }
            })
            .collect();
        let mut rope = carry.map(|c| c.rope.clone()).unwrap_or_default();
        rope.max_tension_per_fall_n = 0.0;
        let mut estop_events: Vec<f64> = sc.events.iter().filter(|e| e.kind == EventKind::EmergencyStop).map(|e| e.t_s).collect();
        estop_events.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut samples = Samples::default();
        samples.axes = vec![AxisSamples::default(); 3];
        let load_kg = sc.start.load_kg.max(0.0);
        Ok(Self {
            spec,
            sc,
            st,
            t: 0.0,
            load_kg,
            load_peak_kg: load_kg,
            axes,
            rope,
            estop: false,
            estop_events,
            samples,
            next_sample_t: 0.0,
            last_aux: Aux { tension_n: (load_kg + hook_kg) * G, ..Default::default() },
            max_skew_m: 0.0,
            max_wind: 0.0,
            slack_events: 0,
            slack_prev: false,
            moves: Vec::new(),
            program: Vec::new(),
            losses: Axis::all()
                .iter()
                .map(|a| AxisLossProfile { axis: Some(*a), dt: sc.sim.sample_dt, ..Default::default() })
                .collect(),
            tracker: MoveTracker::default(),
            notes: Vec::new(),
        })
    }

    fn q(&self, a: Axis) -> f64 {
        match a {
            Axis::Travel => self.st.x(),
            Axis::Trolley => self.st.y(),
            Axis::Hoist => -self.st.l(),
        }
    }

    fn q_dot(&self, a: Axis) -> f64 {
        match a {
            Axis::Travel => self.st.vx(),
            Axis::Trolley => self.st.vy(),
            Axis::Hoist => -self.st.vl(),
        }
    }

    fn set_q_dot_zero(&mut self, a: Axis) {
        match a {
            Axis::Travel => {
                self.st.s[IVX] = 0.0;
                self.st.s[IVPSI] = 0.0;
            }
            Axis::Trolley => self.st.s[IVY] = 0.0,
            Axis::Hoist => self.st.s[IVL] = 0.0,
        }
        self.st.project();
    }

    fn load_cd_area(&self) -> f64 {
        self.sc.load.cd_area(self.load_kg + self.spec.structure.hook_mass_kg)
    }

    fn plant(&self) -> Plant<'_> {
        Plant { spec: &self.spec, env: &self.sc.env, load_kg: self.load_kg, load_cd_area: self.load_cd_area() }
    }

    /// 축 방향 흔들림 각과 각속도
    fn sway_along(&self, a: Axis) -> (f64, f64) {
        let (tx, ty, _) = self.st.sway_angles();
        let (rx, ry) = self.st.sway_rates();
        match a {
            Axis::Travel => (tx, rx),
            Axis::Trolley => (ty, ry),
            Axis::Hoist => (0.0, 0.0),
        }
    }

    fn step(&mut self) {
        let dt = self.sc.sim.dt;
        let t = self.t;
        if !self.estop {
            if let Some(te) = self.estop_events.first().cloned() {
                if t >= te {
                    self.estop = true;
                    self.estop_events.remove(0);
                    for rt in self.axes.iter_mut() {
                        rt.brake.command_set();
                        rt.profile = None;
                        rt.keep_active = false;
                    }
                    self.notes.push(format!("{:.2} s 비상 정지: 토크 차단 + 브레이크 체결", t));
                }
            }
        }
        let ms = self.plant().masses();
        let tension_meas = self.last_aux.tension_n;
        let closed = self.sc.antisway.mode == AntiSwayMode::ClosedLoop;
        let mut inputs = Inputs::default();
        for k in 0..3 {
            let axis = Axis::all()[k];
            let q = self.q(axis);
            let qd = self.q_dot(axis);
            let sway = self.sway_along(axis);
            let d = self.spec.drive(axis);
            let hold_s = self.sc.sim.hold_before_brake_s;
            let estop = self.estop;
            let rt = &mut self.axes[k];

            // 기준 속도
            let (mut v_ref, mut a_ref) = (0.0, 0.0);
            let mut active = false;
            let mut x_ref = rt.q_target;
            let mut done = false;
            if let Some(p) = &rt.profile {
                let tp = t - rt.t0;
                if tp <= p.duration() {
                    active = true;
                    if tp >= 0.0 {
                        v_ref = p.velocity_at(tp);
                        a_ref = p.accel_at(tp);
                        if closed {
                            x_ref = rt.q0 + p.position_with(&rt.cum, tp);
                        }
                    } else {
                        x_ref = rt.q0;
                    }
                } else {
                    done = true;
                }
            }
            if done {
                rt.profile = None;
            }

            // 브레이크 시퀀스
            let wants = (active || rt.keep_active) && !estop;
            if wants {
                rt.brake.command_release();
                rt.hold_timer = 0.0;
            } else if rt.brake.phase == BrakePhase::Released {
                if qd.abs() < 0.003 {
                    rt.hold_timer += dt;
                    if rt.hold_timer >= hold_s {
                        rt.brake.command_set();
                    }
                } else {
                    rt.hold_timer = 0.0;
                }
            }
            rt.brake.update(&d.brake, dt);
            let energized = !estop && matches!(rt.brake.phase, BrakePhase::Released | BrakePhase::Releasing | BrakePhase::Setting);

            // 각도 궤환 (LQR)
            if closed && axis != Axis::Hoist && energized && rt.brake.released() && (active || rt.keep_active) {
                let z = [q - x_ref, qd - v_ref, sway.0, sway.1];
                let a_fb: f64 = -(0..4).map(|i| rt.lqr_k[i] * z[i]).sum::<f64>();
                let amax = d.accel_mps2 * self.sc.accel_scale * 1.5;
                let a_cmd = (a_ref + a_fb).clamp(-amax, amax);
                let vmax = d.max_speed_mps * self.sc.speed_scale * 1.1;
                rt.v_cmd = (rt.v_cmd + a_cmd * dt).clamp(-vmax, vmax);
                v_ref = rt.v_cmd;
                a_ref = a_cmd;
            } else {
                rt.v_cmd = qd;
            }
            rt.last_v_ref = v_ref;

            // 인버터 전류 한계 (과부하 시간 예산)
            let i_rated = d.motor.rated_current_a;
            let i_lim = if rt.overload_s < d.inverter.overload_s {
                d.inverter.overload_factor * i_rated
            } else {
                d.inverter.continuous_factor * i_rated
            };
            let omega = d.motor_speed_rad_s(qd);
            let tau_lim = d.motor.torque_limit_nm(omega, i_lim);

            // 속도 제어 (PI + 가속·부하 앞먹임)
            let mut tau = 0.0;
            if energized {
                let (m_eff, f_ff) = match axis {
                    Axis::Travel => {
                        let n = (self.spec.structure.bridge_mass_kg + self.spec.structure.trolley_mass_kg + ms.m) * G;
                        (ms.mx + ms.m, self.spec.structure.rolling_resistance * n * (v_ref / 0.01).tanh())
                    }
                    Axis::Trolley => {
                        let n = (self.spec.structure.trolley_mass_kg + ms.m) * G;
                        (ms.my + ms.m, self.spec.structure.rolling_resistance * n * (v_ref / 0.01).tanh())
                    }
                    // 하중계(로프 장력) 앞먹임 = 토크 기억
                    Axis::Hoist => (ms.mh + ms.m, tension_meas),
                };
                let e = v_ref - qd;
                let bw = d.speed_loop.bandwidth_rad_s;
                let ki = bw * bw / d.speed_loop.integral_ratio.max(0.1);
                let f_des = m_eff * (a_ref + bw * e + ki * rt.integ) + f_ff;
                let tau_des = d.torque_for_force(f_des, qd);
                tau = tau_des.clamp(-tau_lim, tau_lim);
                let sat = tau_des.abs() > tau_lim + 1e-9;
                if !sat || e.signum() != tau_des.signum() {
                    rt.integ += e * dt;
                }
                if sat {
                    rt.stats.sat_time += dt;
                }
            } else {
                rt.integ = 0.0;
            }

            // 브레이크: 잡고 있음 + 정지 → 잠금, 잡고 있음 + 움직임 → 미끄럼 마찰
            let engaged = rt.brake.holding();
            let moving = qd.abs() > 1e-4;
            inputs.locked[k] = engaged && !moving;
            if engaged && moving {
                inputs.brake_force[k] = d.motors as f64 * d.brake.holding_torque_nm * d.kin();
                tau = 0.0;
            }
            if inputs.locked[k] {
                tau = if energized { tau } else { 0.0 };
            }
            match axis {
                Axis::Travel => {
                    let f = d.force_from_torque(tau, qd);
                    let mm = d.side_mismatch;
                    inputs.f_travel_left = 0.5 * f * (1.0 + 0.5 * mm);
                    inputs.f_travel_right = 0.5 * f * (1.0 - 0.5 * mm);
                    rt.tau = [tau * (1.0 + 0.5 * mm), tau * (1.0 - 0.5 * mm)];
                }
                Axis::Trolley => {
                    inputs.f_trolley = d.force_from_torque(tau, qd);
                    rt.tau = [tau, tau];
                }
                Axis::Hoist => {
                    inputs.f_hoist = d.force_from_torque(tau, qd);
                    rt.tau = [tau, tau];
                }
            }
            rt.energized = energized;
            if energized && tau_lim > 0.0 {
                rt.stats.peak_tau_ratio = rt.stats.peak_tau_ratio.max(tau.abs() / tau_lim);
            }
        }

        // 적분
        let qd_before: Vec<f64> = Axis::all().iter().map(|a| self.q_dot(*a)).collect();
        let aux = {
            let cd = self.load_cd_area();
            let spec = &self.spec;
            let plant = Plant { spec, env: &self.sc.env, load_kg: self.load_kg, load_cd_area: cd };
            plant.step(t, &mut self.st, &inputs, dt)
        };
        self.last_aux = aux;

        // 미끄럼 제동: 마찰일 누적, 멈추면 잠금
        for k in 0..3 {
            if inputs.brake_force[k] > 0.0 {
                let axis = Axis::all()[k];
                let after = self.q_dot(axis);
                let v_avg = 0.5 * (qd_before[k].abs() + after.abs());
                {
                    let d = self.spec.drive(axis);
                    let work = inputs.brake_force[k] * v_avg * dt / d.motors as f64;
                    self.axes[k].brake.add_friction_work(&d.brake, work);
                }
                if qd_before[k].signum() != after.signum() || after.abs() < 1e-3 {
                    self.set_q_dot_zero(axis);
                }
            }
        }

        // 모터 전기 · 열
        let amb = self.sc.env.ambient_c;
        for k in 0..3 {
            let axis = Axis::all()[k];
            let qd = self.q_dot(axis);
            let d = self.spec.drive(axis);
            let omega = d.motor_speed_rad_s(qd);
            let rt = &mut self.axes[k];
            let mut max_i = 0.0f64;
            let mut p_axis = 0.0;
            let mut l0 = MotorLosses::default();
            for j in 0..d.motors as usize {
                let tau_j = if axis == Axis::Travel { rt.tau[j % 2] } else { rt.tau[0] };
                let (i, losses, p) = if rt.energized {
                    let losses = d.motor.losses_w(tau_j, omega);
                    (d.motor.current_a(tau_j, omega), losses, tau_j * omega + losses.total())
                } else {
                    let mech = d.motor.losses_w(0.0, omega).mech_w;
                    (0.0, MotorLosses { copper_w: 0.0, iron_w: 0.0, mech_w: mech }, 0.0)
                };
                if j == 0 {
                    l0 = losses;
                }
                rt.thermal[j].step(&d.motor, &losses, omega, amb, dt);
                rt.stats.peak_winding = rt.stats.peak_winding.max(rt.thermal[j].winding_c);
                max_i = max_i.max(i);
                rt.stats.i2t += i * i * dt / d.motors as f64;
                if p >= 0.0 {
                    rt.stats.energy_in_j += p / d.inverter.efficiency.max(0.5) * dt;
                } else {
                    rt.stats.energy_regen_j += -p * d.inverter.efficiency * dt;
                }
                p_axis += p;
                rt.stats.peak_tau = rt.stats.peak_tau.max(tau_j.abs());
            }
            rt.last_current = max_i;
            rt.last_power_w = p_axis;
            rt.last_losses = l0;
            rt.stats.peak_current = rt.stats.peak_current.max(max_i);
            if max_i > d.inverter.continuous_factor * d.motor.rated_current_a * 1.0001 {
                rt.overload_s += dt;
            } else {
                rt.overload_s = (rt.overload_s - 0.25 * dt).max(0.0);
            }
            rt.stats.overload_peak = rt.stats.overload_peak.max(rt.overload_s);
            if rt.energized {
                rt.stats.run_time += dt;
                if !rt.stats.was_energized {
                    rt.stats.starts += 1;
                }
            }
            rt.stats.was_energized = rt.energized;
            rt.brake.cool(&d.brake, amb, dt);
        }

        // 로프 · 편행 · 바람 · 느슨함
        let falls = self.spec.hoist.reeving_falls.max(1.0);
        self.rope.step(&self.spec.rope, self.st.vl(), aux.tension_n / falls, dt);
        self.max_skew_m = self.max_skew_m.max((self.st.psi() * self.spec.structure.span_m).abs());
        let w = aux.wind;
        self.max_wind = self.max_wind.max((w[0] * w[0] + w[1] * w[1]).sqrt());
        if aux.slack && !self.slack_prev {
            self.slack_events += 1;
        }
        self.slack_prev = aux.slack;

        // 이동 지표
        let sway_m = self.st.sway_offset_m();
        if self.tracker.active {
            let tr = &mut self.tracker;
            tr.max_sway_m = tr.max_sway_m.max(sway_m);
            tr.max_sway_rad = tr.max_sway_rad.max(self.st.sway_angles().2);
            if tr.settling {
                tr.residual_m = tr.residual_m.max(sway_m);
                if t >= tr.final_start {
                    tr.final_m = tr.final_m.max(sway_m);
                }
                if sway_m * 1000.0 > self.spec.limits.sway_tolerance_mm {
                    tr.last_above_tol = t + dt;
                }
            }
        }

        self.t += dt;
        if self.t + 1e-9 >= self.next_sample_t {
            self.record();
            self.next_sample_t += self.sc.sim.sample_dt;
        }
    }

    fn record(&mut self) {
        let s = &mut self.samples;
        let st = &self.st;
        let (tx, ty, _) = st.sway_angles();
        let lp = st.load_pos(self.spec.structure.suspension_height_m);
        let w = self.last_aux.wind;
        s.t.push(self.t);
        s.x.push(st.x());
        s.y.push(st.y());
        s.l.push(st.l());
        s.theta_x_deg.push(tx.to_degrees());
        s.theta_y_deg.push(ty.to_degrees());
        s.sway_mm.push(st.sway_offset_m() * 1000.0);
        s.load_x.push(lp[0]);
        s.load_y.push(lp[1]);
        s.load_z.push(lp[2]);
        s.tension_kn.push(self.last_aux.tension_n / 1000.0);
        s.wind_mps.push((w[0] * w[0] + w[1] * w[1]).sqrt());
        s.skew_mm.push(st.psi() * self.spec.structure.span_m * 1000.0);
        s.load_kg.push(self.load_kg);
        for k in 0..3 {
            let axis = Axis::all()[k];
            let d = self.spec.drive(axis);
            let qd = match axis {
                Axis::Travel => st.vx(),
                Axis::Trolley => st.vy(),
                Axis::Hoist => -st.vl(),
            };
            let rt = &self.axes[k];
            let a = &mut s.axes[k];
            a.v_ref.push(rt.last_v_ref);
            a.v.push(qd);
            a.rpm.push(d.motor_speed_rad_s(qd) * 60.0 / (2.0 * std::f64::consts::PI));
            a.torque_nm.push(rt.tau[0]);
            a.current_a.push(rt.last_current);
            a.power_kw.push(rt.last_power_w / 1000.0);
            a.winding_c.push(rt.thermal.iter().map(|t| t.winding_c).fold(f64::MIN, f64::max));
            a.brake.push(match rt.brake.phase {
                BrakePhase::Released => 0,
                BrakePhase::Setting => 1,
                BrakePhase::Set => 2,
                BrakePhase::Releasing => 3,
            });
            let lp = &mut self.losses[k];
            lp.copper_w.push(rt.last_losses.copper_w);
            lp.other_w.push(rt.last_losses.iron_w + rt.last_losses.mech_w);
            lp.omega.push(d.motor_speed_rad_s(qd));
        }
    }

    fn any_profile(&self) -> bool {
        self.axes.iter().any(|a| a.profile.is_some())
    }

    fn run_for(&mut self, secs: f64) {
        let end = self.t + secs;
        while self.t < end - 1e-9 {
            self.step();
        }
    }

    fn run_phase(&mut self, targets: &[(Axis, f64)], l_design: f64, prog: &mut MoveProgram) {
        if targets.is_empty() || self.estop {
            return;
        }
        let dt = self.sc.sim.dt;
        let sc: &'a Scenario = self.sc;
        let asw = &sc.antisway;
        let f_true = antisway::pendulum_freq_hz(l_design);
        let f_design = f_true * (1.0 + asw.freq_error);
        let shaper = Shaper::for_mode(asw.mode, f_design, asw.zeta, asw.ei_vtol);
        let mut rel: f64 = 0.0;
        for (a, _) in targets {
            let rt = &self.axes[a.index()];
            if !rt.brake.released() {
                rel = rel.max(self.spec.drive(*a).brake.release_delay_s);
            }
        }
        let t_start = self.t + rel + 0.02;
        for (a, q_t) in targets {
            let d = self.spec.drive(*a).clone();
            let q_now = self.q(*a);
            let dist = q_t - q_now;
            // 경하중 고속: 하중이 정격의 light_load_fraction 이하이면 최고 속도 × light_load_speed_factor
            let light = self.load_kg <= d.light_load_fraction * self.spec.structure.rated_load_kg;
            let v_factor = if light { d.light_load_speed_factor.max(1.0) } else { 1.0 };
            let lim = ProfileLimits {
                v_max: d.max_speed_mps * v_factor * self.sc.speed_scale,
                a_max: d.accel_mps2 * self.sc.accel_scale,
                jerk: d.jerk_mps3,
            };
            let sh = if *a != Axis::Hoist { shaper.as_ref() } else { None };
            let p = motion::build(dist, &lim, sh, dt);
            prog.axes.push(AxisProgram {
                axis: *a,
                distance_m: dist,
                t_start_s: t_start,
                duration_s: p.duration(),
                v_max_mps: lim.v_max,
                a_max_mps2: lim.a_max,
                jerk_mps3: lim.jerk,
                shaper: sh.cloned(),
                v_ref: p.downsample(self.sc.sim.sample_dt),
                sample_dt: self.sc.sim.sample_dt,
            });
            let rt = &mut self.axes[a.index()];
            rt.cum = p.cumulative();
            rt.profile = Some(p);
            rt.t0 = t_start;
            rt.q0 = q_now;
            rt.q_target = *q_t;
            if asw.mode == AntiSwayMode::ClosedLoop && *a != Axis::Hoist {
                rt.lqr_k = antisway::lqr_gain(l_design, asw.lqr_q, asw.lqr_r.max(1e-6), dt);
            }
        }
        let limit = self.t + self.sc.sim.max_phase_s;
        while self.any_profile() && self.t < limit && !self.estop {
            self.step();
        }
        if self.any_profile() && !self.estop {
            self.notes.push(format!("{}: 단계 시간 한계({} s) 초과", prog.label, self.sc.sim.max_phase_s));
            for rt in self.axes.iter_mut() {
                rt.profile = None;
            }
        }
    }

    fn run_waypoint(&mut self, idx: usize, wp: &Waypoint) {
        if self.estop {
            return;
        }
        let estop_before = self.estop;
        let s = self.spec.structure.clone();
        let (x0, y0, l0) = (self.st.x(), self.st.y(), self.st.l());
        let xt = wp.x.unwrap_or(x0).clamp(0.0, s.runway_m);
        let yt = wp.y.unwrap_or(y0).clamp(0.0, s.span_m);
        let lt = wp.l.unwrap_or(l0).clamp(s.rope_min_m, s.rope_max_m);
        let label = if wp.label.is_empty() { format!("WP{}", idx + 1) } else { wp.label.clone() };
        let mut prog = MoveProgram { label: label.clone(), mode: wp.mode, axes: Vec::new() };
        let start = self.t;
        self.tracker = MoveTracker { active: true, ..Default::default() };
        let eps = 1e-6;
        let mut horiz: Vec<(Axis, f64)> = Vec::new();
        if (xt - x0).abs() > eps {
            horiz.push((Axis::Travel, xt));
        }
        if (yt - y0).abs() > eps {
            horiz.push((Axis::Trolley, yt));
        }
        let design_l;
        match wp.mode {
            MoveMode::Sequential => {
                if lt < l0 - eps {
                    self.run_phase(&[(Axis::Hoist, -lt)], self.st.l(), &mut prog);
                }
                design_l = self.st.l();
                self.run_phase(&horiz, design_l, &mut prog);
                if lt > self.st.l() + eps {
                    self.run_phase(&[(Axis::Hoist, -lt)], self.st.l(), &mut prog);
                }
            }
            MoveMode::Simultaneous => {
                let mut all = horiz.clone();
                if (lt - l0).abs() > eps {
                    all.push((Axis::Hoist, -lt));
                }
                design_l = 0.5 * (l0 + lt);
                self.run_phase(&all, design_l, &mut prog);
            }
        }
        let end = self.t;
        // 안정 구간: 폐루프는 주행·횡행 제어를 유지한다
        let closed = self.sc.antisway.mode == AntiSwayMode::ClosedLoop && !self.estop;
        if closed {
            for a in [Axis::Travel, Axis::Trolley] {
                let q_now = self.q(a);
                let rt = &mut self.axes[a.index()];
                if horiz.iter().any(|(h, _)| *h == a) {
                    rt.keep_active = true;
                    rt.q0 = q_now;
                }
            }
        }
        self.tracker.settling = true;
        self.tracker.settle_start = self.t;
        self.tracker.last_above_tol = self.t;
        let half_period = 0.5 / antisway::pendulum_freq_hz(self.st.l());
        self.tracker.final_start = self.t + (self.sc.sim.settle_s - half_period).max(0.0);
        self.run_for(self.sc.sim.settle_s);
        for rt in self.axes.iter_mut() {
            rt.keep_active = false;
        }
        let tr = self.tracker.clone();
        let lp = self.st.load_pos(s.suspension_height_m);
        let pos_err = ((lp[0] - xt).powi(2) + (lp[1] - yt).powi(2)).sqrt();
        let shaper = prog.axes.iter().find_map(|a| a.shaper.clone());
        self.moves.push(MoveMetrics {
            label,
            start_s: start,
            end_s: end,
            settle_end_s: self.t,
            duration_s: end - start,
            target: [xt, yt, lt],
            rope_length_m: design_l,
            shaper: shaper.as_ref().map(|s| s.kind.clone()),
            shaper_freq_hz: shaper.as_ref().map(|s| s.freq_hz),
            max_sway_mm: tr.max_sway_m * 1000.0,
            max_sway_deg: tr.max_sway_rad.to_degrees(),
            residual_sway_mm: tr.residual_m * 1000.0,
            final_sway_mm: tr.final_m * 1000.0,
            settle_time_s: (tr.last_above_tol - tr.settle_start).max(0.0),
            position_error_mm: pos_err * 1000.0,
            load_kg: self.load_kg,
            aborted: self.estop && !estop_before,
        });
        self.program.push(prog);
        self.tracker.active = false;
        if let Some(m) = wp.set_load_kg {
            self.load_kg = m.max(0.0);
            self.load_peak_kg = self.load_peak_kg.max(self.load_kg);
        }
        if wp.dwell_s > 0.0 {
            self.run_for(wp.dwell_s);
        }
    }

    pub fn run(self) -> CycleReport {
        self.run_carry().0
    }

    /// 실행하고, 다음 사이클로 넘길 상태(모터 온도 · 브레이크 · 로프 손상)도 돌려준다
    pub fn run_carry(mut self) -> (CycleReport, CycleCarry) {
        let wps = self.sc.waypoints.clone();
        self.record();
        self.next_sample_t = self.sc.sim.sample_dt;
        for (i, wp) in wps.iter().enumerate() {
            self.run_waypoint(i, wp);
        }
        if self.estop {
            // 비상 정지 뒤: 브레이크가 다 잡을 때까지 + 흔들림 관찰
            self.run_for(self.sc.sim.settle_s.max(2.0));
        }
        self.rope.close(&self.spec.rope);
        let carry = CycleCarry {
            thermal: self.axes.iter().map(|a| a.thermal.clone()).collect(),
            brakes: self.axes.iter().map(|a| a.brake.clone()).collect(),
            rope: self.rope.clone(),
        };
        (self.finish(), carry)
    }

    fn finish(self) -> CycleReport {
        let cycle = self.t.max(1e-9);
        let spec = &self.spec;
        let axes: Vec<AxisMetrics> = self
            .axes
            .iter()
            .map(|rt| {
                let d = spec.drive(rt.axis);
                let s = &rt.stats;
                AxisMetrics {
                    axis: rt.axis,
                    motors: d.motors,
                    motor: d.motor.name.clone(),
                    rated_torque_nm: d.motor.rated_torque_nm(),
                    peak_torque_nm: s.peak_tau,
                    peak_torque_ratio: s.peak_tau_ratio,
                    torque_saturated_s: s.sat_time,
                    rated_current_a: d.motor.rated_current_a,
                    peak_current_a: s.peak_current,
                    rms_current_a: (s.i2t / cycle).sqrt(),
                    inverter_overload_peak_s: s.overload_peak,
                    energy_in_kwh: s.energy_in_j / 3.6e6,
                    energy_regen_kwh: s.energy_regen_j / 3.6e6,
                    regen_to_resistor: d.inverter.regen == RegenMode::BrakeResistor,
                    peak_winding_c: s.peak_winding,
                    end_winding_c: rt.thermal.iter().map(|t| t.winding_c).fold(f64::MIN, f64::max),
                    insulation_limit_c: d.motor.insulation_limit_c(),
                    run_time_s: s.run_time,
                    starts: s.starts,
                    cdf: s.run_time / cycle,
                    starts_per_hour: s.starts as f64 * 3600.0 / cycle,
                    rated_cdf: d.motor.duty.cdf_percent / 100.0,
                    rated_starts_per_hour: d.motor.duty.starts_per_hour,
                    brake_lining_mm: rt.brake.lining_mm,
                    brake_lining_remaining: rt.brake.remaining_fraction(&d.brake),
                    brake_work_j: rt.brake.work_total_j,
                    brake_max_stop_work_j: rt.brake.max_stop_work_j,
                    brake_max_work_allowed_j: d.brake.permitted_work_per_stop_j((rt.brake.operations as f64 * 3600.0 / cycle).max(1.0)),
                    brake_max_temp_c: rt.brake.max_temp_c,
                    brake_temp_limit_c: d.brake.max_temp_c,
                    brake_operations: rt.brake.operations,
                    brake_dynamic_stops: rt.brake.dynamic_stops,
                }
            })
            .collect();
        let falls = spec.hoist.reeving_falls.max(1.0);
        let s_max = self.rope.max_tension_per_fall_n;
        let static_t = spec.hoist_static_torque_nm(self.load_peak_kg);
        let fatigue_source = spec.rope.fatigue.source().to_string();
        let mut notes = self.notes.clone();
        if spec.rope.d_ratio_sheave() < 16.0 {
            notes.push(format!("시브 D/d = {:.1} 이 작습니다", spec.rope.d_ratio_sheave()));
        }
        let _ = falls;
        CycleReport {
            scenario: self.sc.name.clone(),
            crane: spec.name.clone(),
            antisway: self.sc.antisway.mode.key().into(),
            rated_load_kg: spec.structure.rated_load_kg,
            load_peak_kg: self.load_peak_kg,
            cycle_time_s: self.t,
            moves: self.moves.clone(),
            axes,
            rope: RopeMetrics {
                max_tension_per_fall_kn: s_max / 1000.0,
                min_safety_factor: spec.rope.safety_factor(s_max),
                required_safety_factor: spec.rope.min_safety_factor,
                d_ratio_sheave: spec.rope.d_ratio_sheave(),
                strokes: self.rope.strokes,
                damage: self.rope.damage,
                fatigue_source,
            },
            hoist_brake_factor: spec.hoist.brake.holding_torque_nm * spec.hoist.motors as f64 / static_t.max(1e-9),
            max_skew_mm: self.max_skew_m * 1000.0,
            max_wind_mps: self.max_wind,
            slack_events: self.slack_events,
            estop: self.estop,
            notes,
            program: self.program.clone(),
            samples: self.samples.clone(),
            loss_profiles: self.losses.clone(),
        }
    }
}

/// 시나리오 하나를 실행한다
pub fn simulate(sc: &Scenario) -> Result<CycleReport, String> {
    Ok(Sim::new(sc, None)?.run())
}

/// 이전 사이클 상태에서 이어서 실행한다
pub fn simulate_carry(sc: &Scenario, carry: Option<&CycleCarry>) -> Result<(CycleReport, CycleCarry), String> {
    Ok(Sim::new(sc, carry)?.run_carry())
}

/// 같은 시나리오를 흔들림 억제 방식별로 실행한다
pub fn compare_antisway(sc: &Scenario, modes: &[AntiSwayMode]) -> Result<Vec<CycleReport>, String> {
    modes
        .iter()
        .map(|m| {
            let mut s = sc.clone();
            s.antisway.mode = *m;
            simulate(&s)
        })
        .collect()
}

/// 데모용 기본 시나리오: 하중을 걸어 들고, 옮기고, 내려놓고, 돌아온다
pub fn demo_scenario(crane: &str, load_kg: f64, mode: AntiSwayMode) -> Scenario {
    let spec = crate::crane_profile::preset(crane).unwrap_or_else(crate::crane_profile::gantry_10t_20m);
    let s = &spec.structure;
    // 트럭 적재함 · 적치대 높이 정도의 6 m 행정
    let top = s.rope_min_m + 1.0;
    let low = (top + 6.0).min(s.rope_max_m - 0.5).max(top + 1.0);
    let x_far = (s.runway_m * 0.5).min(30.0);
    let y_far = s.span_m * 0.8;
    Scenario {
        name: format!("demo_{}_{}kg", spec.name, load_kg as i64),
        crane: CraneRef::Preset(spec.name.clone()),
        start: StartState { x: 2.0, y: s.span_m * 0.2, l: top, load_kg: 0.0 },
        waypoints: vec![
            Waypoint { label: "하중 걸기".into(), x: None, y: None, l: Some(low), mode: MoveMode::Sequential, dwell_s: 5.0, set_load_kg: Some(load_kg) },
            Waypoint { label: "들어 옮기기".into(), x: Some(x_far), y: Some(y_far), l: Some(top), mode: MoveMode::Sequential, dwell_s: 0.0, set_load_kg: None },
            Waypoint { label: "내려놓기".into(), x: None, y: None, l: Some(low), mode: MoveMode::Sequential, dwell_s: 5.0, set_load_kg: Some(0.0) },
            Waypoint { label: "복귀".into(), x: Some(2.0), y: Some(s.span_m * 0.2), l: Some(top), mode: MoveMode::Sequential, dwell_s: 0.0, set_load_kg: None },
        ],
        antisway: AntiSwayConfig { mode, ..Default::default() },
        env: Environment::default(),
        sim: SimConfig::default(),
        load: LoadShape::default(),
        speed_scale: 1.0,
        accel_scale: 1.0,
        events: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn short_move(mode: AntiSwayMode) -> Scenario {
        Scenario {
            name: "t".into(),
            crane: CraneRef::Preset("gantry_10t_20m".into()),
            start: StartState { x: 2.0, y: 5.0, l: 8.0, load_kg: 8000.0 },
            waypoints: vec![Waypoint { label: "move".into(), x: Some(14.0), y: None, l: None, mode: MoveMode::Sequential, dwell_s: 0.0, set_load_kg: None }],
            antisway: AntiSwayConfig { mode, ..Default::default() },
            env: Environment::default(),
            sim: SimConfig { settle_s: 8.0, ..Default::default() },
            load: LoadShape::default(),
            speed_scale: 1.0,
            accel_scale: 1.0,
            events: Vec::new(),
        }
    }

    #[test]
    fn travel_reaches_target_and_shaping_reduces_residual_sway() {
        let off = simulate(&short_move(AntiSwayMode::Off)).unwrap();
        let zv = simulate(&short_move(AntiSwayMode::Zv)).unwrap();
        let zvd = simulate(&short_move(AntiSwayMode::Zvd)).unwrap();
        let cl = simulate(&short_move(AntiSwayMode::ClosedLoop)).unwrap();
        for r in [&off, &zv, &zvd, &cl] {
            let m = &r.moves[0];
            assert!(m.position_error_mm < 300.0, "{} pos err {}", r.antisway, m.position_error_mm);
        }
        let r_off = off.moves[0].residual_sway_mm;
        assert!(r_off > 30.0, "unshaped move should leave sway: {r_off}");
        assert!(zv.moves[0].residual_sway_mm < 0.25 * r_off, "zv {} vs off {}", zv.moves[0].residual_sway_mm, r_off);
        assert!(zvd.moves[0].residual_sway_mm < 0.25 * r_off, "zvd {} vs off {}", zvd.moves[0].residual_sway_mm, r_off);
        // 폐루프는 기준이 끝난 뒤에도 감쇠시킨다 → 안정 구간 끝의 흔들림과 안정화 시간으로 비교
        let f_off = off.moves[0].final_sway_mm;
        assert!(cl.moves[0].final_sway_mm < 0.2 * f_off, "closed loop final {} vs off {}", cl.moves[0].final_sway_mm, f_off);
        assert!(cl.moves[0].settle_time_s < off.moves[0].settle_time_s);
        // 셰이핑은 이동 시간을 늘린다
        assert!(zvd.moves[0].duration_s > off.moves[0].duration_s);
    }

    #[test]
    fn hoist_holds_load_without_drop_on_brake_release() {
        let mut sc = short_move(AntiSwayMode::Off);
        sc.waypoints = vec![Waypoint { label: "up".into(), x: None, y: None, l: Some(5.0), mode: MoveMode::Sequential, dwell_s: 1.0, set_load_kg: None }];
        let r = simulate(&sc).unwrap();
        let l = &r.samples.l;
        // 브레이크 해제 직후 로프가 늘어나지(하중이 떨어지지) 않는다
        let max_drop = l.windows(2).take(20).map(|w| w[1] - w[0]).fold(0.0f64, f64::max);
        assert!(max_drop < 0.005, "{max_drop}");
        assert!((r.samples.l.last().unwrap() - 5.0).abs() < 0.02);
        let hoist = &r.axes[2];
        assert!(hoist.energy_in_kwh > 0.0);
        assert!(hoist.peak_winding_c >= 25.0);
        assert!(r.rope.strokes >= 1);
        assert!(r.rope.damage > 0.0);
    }

    #[test]
    fn emergency_stop_wears_brakes() {
        let mut sc = short_move(AntiSwayMode::Off);
        sc.events = vec![Event { t_s: 15.0, kind: EventKind::EmergencyStop }];
        let r = simulate(&sc).unwrap();
        assert!(r.estop);
        let travel = &r.axes[0];
        assert!(travel.brake_dynamic_stops >= 1);
        assert!(travel.brake_work_j > 100.0, "{}", travel.brake_work_j);
        assert!(travel.brake_lining_mm < r_lining_new(&sc));
    }

    fn r_lining_new(sc: &Scenario) -> f64 {
        sc.crane.resolve().unwrap().travel.brake.lining_new_mm
    }

    #[test]
    fn demo_cycle_runs_and_exports_csv() {
        let sc = demo_scenario("overhead_5t_16m", 4000.0, AntiSwayMode::Zvd);
        let r = simulate(&sc).unwrap();
        assert_eq!(r.moves.len(), 4);
        assert!(r.load_peak_kg == 4000.0);
        let csv = r.samples.to_csv();
        assert!(csv.lines().count() > 100);
        assert!(r.cycle_time_s > 60.0);
        for a in r.axes.iter() {
            assert!(a.cdf > 0.0 && a.cdf <= 1.0);
        }
        assert_eq!(r.slack_events, 0);
    }
}
