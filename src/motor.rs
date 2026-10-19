//! 크레인 축 구동 모터 모델 (인버터 구동 유도전동기 · PMSM).
//!
//! - 토크 한계: 기저속도 이하는 인버터 전류 한계가 정하는 정토크, 이상은 약계자(자속 ψ = ω_b/ω)와
//!   정지 토크(∝ ψ²) 한계 중 작은 값.
//! - 전류: 자속분 I_d = I0·ψ, 토크분 I_q = I_qn·(T/T_n)/ψ 의 벡터 합 (자속 기준 제어 근사).
//!   정격 토크·정격 속도에서 정확히 정격 전류가 되도록 I_qn = √(I_n² − I0²) 로 둔다.
//! - 손실: 정격 손실 P_n(1/η − 1) 을 동손 ∝ (I/I_n)², 철손 ∝ ψ²(ω/ω_b)^1.3, 기계손 ∝ (ω/ω_n)² 으로 나눈다.
//! - 열: 권선–프레임 2절점 회로. 자기 통풍 팬은 정지 시 냉각 능력이 `standstill_cooling` 비율로 떨어진다.
//!   정격 S1 연속 운전의 정상상태에서 권선 상승이 `rated_rise_k` 가 되도록 열저항을 정격 손실로부터 정한다.
//!
//! 기본 프리셋은 4극 50 Hz 범용 카탈로그 범위의 **예시 값**이며 특정 제조사 제품 값이 아니다.
//! 전류·온도 실측 로그로 `calib` 모듈에서 보정한다.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotorKind {
    Induction,
    Pmsm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalParams {
    /// S1 정격 연속 운전에서 권선 온도 상승 (K)
    pub rated_rise_k: f64,
    /// 정격에서 프레임 온도 상승이 권선 온도 상승에서 차지하는 비율 (0~1)
    pub frame_share: f64,
    /// 권선 절점 시간상수 R_wf·C_w (s)
    pub tau_winding_s: f64,
    /// 프레임 절점 시간상수 R_fa·C_f (s)
    pub tau_frame_s: f64,
    /// 정지 시 냉각 능력 / 정격 속도 냉각 능력 (자기 통풍). 강제 통풍이면 1
    pub standstill_cooling: f64,
}

/// 정격 손실을 나누는 비율 (합이 1 이 아니어도 정규화해서 쓴다)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossSplit {
    pub copper: f64,
    pub iron: f64,
    pub mech: f64,
}

impl LossSplit {
    fn normalized(&self) -> (f64, f64, f64) {
        let s = (self.copper + self.iron + self.mech).max(1e-9);
        (self.copper / s, self.iron / s, self.mech / s)
    }
}

/// IEC 60034-1 듀티 정격
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DutyRating {
    /// "S1" · "S3" · "S4" · "S5"
    pub kind: String,
    /// 정격 사이클 운전율 CDF (%). S3·S4·S5 의 표준 값은 15 · 25 · 40 · 60
    pub cdf_percent: f64,
    /// 허용 기동 횟수 (회/h)
    pub starts_per_hour: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorParams {
    pub name: String,
    pub kind: MotorKind,
    pub rated_power_kw: f64,
    pub rated_speed_rpm: f64,
    pub rated_current_a: f64,
    pub rated_voltage_v: f64,
    pub power_factor: f64,
    pub efficiency: f64,
    /// 정지(최대) 토크 / 정격 토크
    pub breakdown_ratio: f64,
    /// 무부하(자화) 전류 / 정격 전류. PMSM 은 0 근처
    pub magnetizing_fraction: f64,
    /// 회전자 관성 (kg·m²)
    pub inertia_kgm2: f64,
    /// 절연 등급 문자 (B 130 · F 155 · H 180 °C)
    pub insulation_class: String,
    pub thermal: ThermalParams,
    pub losses: LossSplit,
    pub duty: DutyRating,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MotorLosses {
    pub copper_w: f64,
    pub iron_w: f64,
    pub mech_w: f64,
}

impl MotorLosses {
    pub fn total(&self) -> f64 {
        self.copper_w + self.iron_w + self.mech_w
    }
    pub fn add(&mut self, o: &MotorLosses) {
        self.copper_w += o.copper_w;
        self.iron_w += o.iron_w;
        self.mech_w += o.mech_w;
    }
    pub fn scale(&self, k: f64) -> MotorLosses {
        MotorLosses { copper_w: self.copper_w * k, iron_w: self.iron_w * k, mech_w: self.mech_w * k }
    }
}

/// 열 회로 상수
#[derive(Debug, Clone, Copy)]
pub struct ThermalNetwork {
    pub r_wf: f64,
    pub r_fa: f64,
    pub c_w: f64,
    pub c_f: f64,
}

impl MotorParams {
    pub fn rated_speed_rad_s(&self) -> f64 {
        self.rated_speed_rpm * 2.0 * PI / 60.0
    }

    /// 인버터 기저속도 (정격 속도와 같게 둔다)
    pub fn base_speed_rad_s(&self) -> f64 {
        self.rated_speed_rad_s()
    }

    pub fn rated_torque_nm(&self) -> f64 {
        self.rated_power_kw * 1000.0 / self.rated_speed_rad_s().max(1e-9)
    }

    pub fn rated_losses_w(&self) -> f64 {
        self.rated_power_kw * 1000.0 * (1.0 / self.efficiency.clamp(0.3, 0.999) - 1.0)
    }

    pub fn insulation_limit_c(&self) -> f64 {
        match self.insulation_class.trim().to_ascii_uppercase().as_str() {
            "A" => 105.0,
            "E" => 120.0,
            "B" => 130.0,
            "F" => 155.0,
            "H" => 180.0,
            _ => 155.0,
        }
    }

    /// 자속 계수 ψ: 기저속도 이하 1, 이상 ω_b/ω (약계자)
    pub fn flux_factor(&self, omega: f64) -> f64 {
        let wb = self.base_speed_rad_s();
        let w = omega.abs();
        if w <= wb {
            1.0
        } else {
            wb / w
        }
    }

    fn i0(&self) -> f64 {
        self.magnetizing_fraction.clamp(0.0, 0.95) * self.rated_current_a
    }

    fn iqn(&self) -> f64 {
        let i0 = self.i0();
        (self.rated_current_a * self.rated_current_a - i0 * i0).max(1e-9).sqrt()
    }

    /// 토크 T (Nm), 속도 ω (rad/s) 에서의 상전류 크기 (A)
    pub fn current_a(&self, torque: f64, omega: f64) -> f64 {
        let psi = self.flux_factor(omega);
        let id = self.i0() * psi;
        let iq = self.iqn() * (torque / self.rated_torque_nm().max(1e-9)) / psi;
        (id * id + iq * iq).sqrt()
    }

    /// 속도 ω 에서 전류 한계 I_lim 으로 낼 수 있는 최대 전자기 토크 크기 (Nm)
    pub fn torque_limit_nm(&self, omega: f64, current_limit_a: f64) -> f64 {
        let psi = self.flux_factor(omega);
        let id = self.i0() * psi;
        let iq_max = (current_limit_a * current_limit_a - id * id).max(0.0).sqrt();
        let tn = self.rated_torque_nm();
        let t_i = tn * psi * iq_max / self.iqn();
        let t_bd = self.breakdown_ratio * tn * psi * psi;
        t_i.min(t_bd).max(0.0)
    }

    pub fn losses_w(&self, torque: f64, omega: f64) -> MotorLosses {
        let pl = self.rated_losses_w();
        let (kc, kf, km) = self.losses.normalized();
        let i = self.current_a(torque, omega) / self.rated_current_a.max(1e-9);
        let psi = self.flux_factor(omega);
        let wb = self.base_speed_rad_s().max(1e-9);
        let wn = self.rated_speed_rad_s().max(1e-9);
        MotorLosses {
            copper_w: kc * pl * i * i,
            iron_w: kf * pl * psi * psi * (omega.abs() / wb).powf(1.3),
            mech_w: km * pl * (omega / wn).powi(2),
        }
    }

    /// 단자 전력 (W). 양수 = 전동, 음수 = 회생(발전)
    pub fn electrical_power_w(&self, torque: f64, omega: f64) -> f64 {
        torque * omega + self.losses_w(torque, omega).total()
    }

    /// 자기 통풍 냉각 계수 (정격 속도 1, 정지 시 standstill_cooling)
    pub fn cooling_factor(&self, omega: f64) -> f64 {
        let s = self.thermal.standstill_cooling.clamp(0.01, 1.0);
        let w = (omega.abs() / self.rated_speed_rad_s().max(1e-9)).min(1.0);
        s + (1.0 - s) * w
    }

    /// 정격 손실과 정격 온도 상승으로부터 열 회로 상수를 정한다
    pub fn thermal_network(&self) -> ThermalNetwork {
        let pl = self.rated_losses_w().max(1e-6);
        let (kc, _, _) = self.losses.normalized();
        let pcu = (kc * pl).max(1e-6);
        let rise_w = self.thermal.rated_rise_k.max(1.0);
        let share = self.thermal.frame_share.clamp(0.05, 0.95);
        let rise_f = rise_w * share;
        let r_fa = rise_f / pl;
        let r_wf = (rise_w - rise_f) / pcu;
        ThermalNetwork {
            r_wf,
            r_fa,
            c_w: self.thermal.tau_winding_s.max(1.0) / r_wf,
            c_f: self.thermal.tau_frame_s.max(1.0) / r_fa,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let pos = [
            ("rated_power_kw", self.rated_power_kw),
            ("rated_speed_rpm", self.rated_speed_rpm),
            ("rated_current_a", self.rated_current_a),
            ("efficiency", self.efficiency),
            ("breakdown_ratio", self.breakdown_ratio),
            ("inertia_kgm2", self.inertia_kgm2),
            ("thermal.tau_winding_s", self.thermal.tau_winding_s),
            ("thermal.tau_frame_s", self.thermal.tau_frame_s),
        ];
        for (k, v) in pos {
            if !(v.is_finite() && v > 0.0) {
                return Err(format!("모터 {}: {} 는 양수여야 합니다 ({})", self.name, k, v));
            }
        }
        if self.efficiency >= 1.0 {
            return Err(format!("모터 {}: 효율은 1 미만이어야 합니다", self.name));
        }
        Ok(())
    }
}

/// 권선·프레임 온도 상태
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MotorThermal {
    pub winding_c: f64,
    pub frame_c: f64,
}

impl MotorThermal {
    pub fn at_ambient(ambient_c: f64) -> Self {
        Self { winding_c: ambient_c, frame_c: ambient_c }
    }

    /// 손실 `losses` 를 dt 동안 넣는다. 권선 절점에는 동손, 프레임 절점에는 철손·기계손이 들어간다.
    pub fn step(&mut self, m: &MotorParams, losses: &MotorLosses, omega: f64, ambient_c: f64, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        let net = m.thermal_network();
        let r_fa = net.r_fa / m.cooling_factor(omega).max(1e-3);
        let tau_min = m.thermal.tau_winding_s.min(m.thermal.tau_frame_s).min(net.c_f * r_fa).max(1e-3);
        let n = ((dt / (tau_min / 20.0)).ceil() as usize).clamp(1, 100_000);
        let h = dt / n as f64;
        let pw = losses.copper_w;
        let pf = losses.iron_w + losses.mech_w;
        let f = |tw: f64, tf: f64| -> (f64, f64) {
            let q_wf = (tw - tf) / net.r_wf;
            let q_fa = (tf - ambient_c) / r_fa;
            ((pw - q_wf) / net.c_w, (pf + q_wf - q_fa) / net.c_f)
        };
        for _ in 0..n {
            let (a1, b1) = f(self.winding_c, self.frame_c);
            let (a2, b2) = f(self.winding_c + h * a1, self.frame_c + h * b1);
            self.winding_c += 0.5 * h * (a1 + a2);
            self.frame_c += 0.5 * h * (b1 + b2);
        }
    }

    /// 손실이 일정하게 유지될 때 정상상태 (권선, 프레임) 온도
    pub fn steady_state(m: &MotorParams, losses: &MotorLosses, omega: f64, ambient_c: f64) -> (f64, f64) {
        let net = m.thermal_network();
        let r_fa = net.r_fa / m.cooling_factor(omega).max(1e-3);
        let tf = ambient_c + losses.total() * r_fa;
        let tw = tf + losses.copper_w * net.r_wf;
        (tw, tf)
    }
}

/// S3 간헐 주기에서 주기 정상상태 권선 최고 온도 상승 (K).
/// 주기 `cycle_s` 중 `cdf` 비율은 정격 속도·정격 토크의 k 배, 나머지는 정지·무통전.
pub fn s3_peak_rise_k(m: &MotorParams, k: f64, cdf: f64, cycle_s: f64) -> f64 {
    let cdf = cdf.clamp(0.01, 1.0);
    let w = m.rated_speed_rad_s();
    let on = cdf * cycle_s;
    let off = cycle_s - on;
    let l_on = m.losses_w(k * m.rated_torque_nm(), w);
    let zero = MotorLosses::default();
    // 평균 손실의 정상상태 근처에서 시작해 수렴을 앞당긴다
    let net = m.thermal_network();
    let k_bar = cdf * m.cooling_factor(w) + (1.0 - cdf) * m.cooling_factor(0.0);
    let frame0 = cdf * l_on.total() * net.r_fa / k_bar.max(1e-3);
    let mut th = MotorThermal { winding_c: frame0, frame_c: frame0 };
    let n_on = ((on / (cycle_s / 200.0)).round() as usize).max(1);
    let dt_on = on / n_on as f64;
    let mut last_end = f64::MAX;
    let mut peak = 0.0;
    for _ in 0..2000 {
        peak = f64::MIN;
        for _ in 0..n_on {
            th.step(m, &l_on, w, 0.0, dt_on);
            peak = peak.max(th.winding_c);
        }
        if off > 0.0 {
            th.step(m, &zero, 0.0, 0.0, off);
        }
        if (th.frame_c - last_end).abs() < 1e-4 {
            break;
        }
        last_end = th.frame_c;
    }
    peak
}

/// S3 출력 배수: 주기 정상상태 권선 최고 온도가 S1 정격 연속 운전의 권선 온도와 같아지는 k.
/// IEC 60034-1 S3 의 표준 주기는 10분 (600 s).
pub fn s3_power_factor(m: &MotorParams, cdf: f64, cycle_s: f64) -> f64 {
    let target = m.thermal.rated_rise_k;
    if s3_peak_rise_k(m, 1.0, cdf, cycle_s) >= target {
        return 1.0;
    }
    let (mut lo, mut hi) = (1.0, 4.0);
    if s3_peak_rise_k(m, hi, cdf, cycle_s) < target {
        return hi;
    }
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if s3_peak_rise_k(m, mid, cdf, cycle_s) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// SEW-EURODRIVE 문서 26859750 의 S3 출력 배수 (CDF, 배수) — 열 모델 검증 · 보정 목표
pub const SEW_S3_FACTORS: [(f64, f64); 4] = [(0.60, 1.10), (0.40, 1.15), (0.25, 1.30), (0.15, 1.40)];

fn motor(
    name: &str,
    kw: f64,
    rpm: f64,
    amps: f64,
    pf: f64,
    eta: f64,
    breakdown: f64,
    mag: f64,
    inertia: f64,
    tau_w: f64,
    tau_f: f64,
) -> MotorParams {
    MotorParams {
        name: name.into(),
        kind: MotorKind::Induction,
        rated_power_kw: kw,
        rated_speed_rpm: rpm,
        rated_current_a: amps,
        rated_voltage_v: 400.0,
        power_factor: pf,
        efficiency: eta,
        breakdown_ratio: breakdown,
        magnetizing_fraction: mag,
        inertia_kgm2: inertia,
        insulation_class: "F".into(),
        thermal: ThermalParams {
            rated_rise_k: 80.0,
            frame_share: 0.6,
            tau_winding_s: tau_w,
            tau_frame_s: tau_f,
            // 1.1 kW 전폐 외선형 모터: 권선 시간상수 팬 있음 약 8.7분 / 없음 약 39분 → 비 약 0.22 (Gedzurs 2014)
            standstill_cooling: 0.22,
        },
        losses: LossSplit { copper: 0.6, iron: 0.25, mech: 0.15 },
        // S3 40% · 240 회/h = FEM 기구 등급 M5(2m) 의 운전율 · 기동 횟수 한계 (FEM 듀티 표)
        duty: DutyRating { kind: "S3".into(), cdf_percent: 40.0, starts_per_hour: 240.0 },
    }
}

/// 범용 4극 50 Hz 유도전동기 예시 프리셋.
///
/// 정격값은 범용 카탈로그 범위의 예시다(특정 제조사 제품 값이 아님). 열 파라미터는 실측·제조사 자료로 맞췄다:
/// - 정지 냉각 비율 0.22 — Gedzurs 2014 (1.1 kW, 팬 유무 권선 시간상수 8.7분 / 39분)
/// - 소형 모터 프레임 시간상수 500–600 s — 같은 자료의 팬 있는 시간상수 약 8.7분
/// - 권선 시간상수 38–57 s — SEW-EURODRIVE 26859750 의 S3 출력 배수(60% 1.1 · 40% 1.15 · 25% 1.3 · 15% 1.4)에
///   10분 주기로 맞춘 값 (`examples/s3_fit.rs`, 배수 오차 0.06 이내)
pub fn presets() -> Vec<MotorParams> {
    vec![
        motor("im_0p75kw_4p", 0.75, 1400.0, 1.8, 0.76, 0.80, 2.4, 0.55, 0.0025, 38.0, 500.0),
        motor("im_1p5kw_4p", 1.5, 1420.0, 3.3, 0.79, 0.83, 2.6, 0.50, 0.0045, 45.0, 600.0),
        motor("im_4kw_4p", 4.0, 1440.0, 8.1, 0.82, 0.87, 2.8, 0.45, 0.012, 51.0, 1800.0),
        motor("im_7p5kw_4p", 7.5, 1450.0, 14.7, 0.83, 0.89, 2.8, 0.42, 0.040, 55.0, 2100.0),
        motor("im_11kw_4p", 11.0, 1460.0, 21.5, 0.84, 0.905, 2.8, 0.40, 0.067, 57.0, 2400.0),
    ]
}

pub fn preset(name: &str) -> Option<MotorParams> {
    presets().into_iter().find(|m| m.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m11() -> MotorParams {
        preset("im_11kw_4p").unwrap()
    }

    #[test]
    fn rated_point_draws_rated_current() {
        let m = m11();
        let i = m.current_a(m.rated_torque_nm(), m.rated_speed_rad_s());
        assert!((i - m.rated_current_a).abs() < 1e-6, "{i}");
    }

    #[test]
    fn rated_losses_match_efficiency() {
        let m = m11();
        let l = m.losses_w(m.rated_torque_nm(), m.rated_speed_rad_s());
        assert!((l.total() - m.rated_losses_w()).abs() / m.rated_losses_w() < 1e-6);
        let pin = m.electrical_power_w(m.rated_torque_nm(), m.rated_speed_rad_s());
        let eta = m.rated_power_kw * 1000.0 / pin;
        assert!((eta - m.efficiency).abs() < 1e-6, "{eta}");
    }

    #[test]
    fn torque_limit_constant_then_falls_in_field_weakening() {
        let m = m11();
        let ilim = 1.5 * m.rated_current_a;
        let t0 = m.torque_limit_nm(0.0, ilim);
        let tb = m.torque_limit_nm(m.base_speed_rad_s(), ilim);
        let t2 = m.torque_limit_nm(2.0 * m.base_speed_rad_s(), ilim);
        assert!((t0 - tb).abs() < 1e-9);
        assert!(t2 < 0.6 * tb, "field weakening must reduce torque: {t2} vs {tb}");
        // 정격 전류 한계에서는 정격 토크
        let tn = m.torque_limit_nm(m.base_speed_rad_s(), m.rated_current_a);
        assert!((tn - m.rated_torque_nm()).abs() < 1e-6);
        // 전류 한계가 커도 정지 토크를 넘지 않는다
        let big = m.torque_limit_nm(0.0, 100.0 * m.rated_current_a);
        assert!((big - m.breakdown_ratio * m.rated_torque_nm()).abs() < 1e-6);
    }

    #[test]
    fn regenerating_returns_power() {
        let m = m11();
        let p = m.electrical_power_w(-m.rated_torque_nm(), m.rated_speed_rad_s());
        assert!(p < 0.0);
        assert!(-p < m.rated_power_kw * 1000.0);
    }

    #[test]
    fn s1_rated_steady_state_reaches_rated_rise() {
        let m = m11();
        let w = m.rated_speed_rad_s();
        let l = m.losses_w(m.rated_torque_nm(), w);
        let (tw, _) = MotorThermal::steady_state(&m, &l, w, 40.0);
        assert!((tw - (40.0 + m.thermal.rated_rise_k)).abs() < 1e-6, "{tw}");
        // 충분히 오래 적분하면 같은 값으로 수렴
        let mut th = MotorThermal::at_ambient(40.0);
        for _ in 0..(6 * 3600) {
            th.step(&m, &l, w, 40.0, 1.0);
        }
        assert!((th.winding_c - tw).abs() < 0.5, "{} vs {}", th.winding_c, tw);
    }

    #[test]
    fn standstill_cooling_makes_motor_hotter() {
        let m = m11();
        let l = MotorLosses { copper_w: 300.0, iron_w: 0.0, mech_w: 0.0 };
        let (run, _) = MotorThermal::steady_state(&m, &l, m.rated_speed_rad_s(), 20.0);
        let (still, _) = MotorThermal::steady_state(&m, &l, 0.0, 20.0);
        assert!(still > run);
    }

    #[test]
    fn presets_reproduce_sew_s3_factors() {
        for m in presets() {
            for (cdf, k) in SEW_S3_FACTORS {
                let mk = s3_power_factor(&m, cdf, 600.0);
                assert!((mk - k).abs() <= 0.07, "{} CDF {cdf}: model {mk:.3} vs SEW {k}", m.name);
            }
        }
    }

    #[test]
    fn presets_are_valid() {
        for m in presets() {
            m.validate().unwrap();
        }
    }
}
