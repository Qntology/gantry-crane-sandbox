//! 실측 로그로 모델 파라미터를 맞추는 보정 도구 (엔드밀 `physcal.rs` · SDS 보정 통로의 자리).
//!
//! - `fit_s3`: 모터 열 회로(권선 시간상수 · 정지 냉각 비율 · 프레임 비율)를 S3 출력 배수표(SEW 등)에 맞춘다.
//! - `fit_pendulum`: 흔들림 각 기록 → 감쇠 진동수 · 감쇠비 · 등가 로프 길이.
//! - `fit_winding`: 전류 · 속도 · 냉각 기준 온도 · 권선 온도 기록 → 2단 열 모델 (Kaggle Electric Motor Temperature 형식).
//! - `fit_first_order`: 계단 응답 y(t) = y∞ + (y0 − y∞)·e^(−t/τ).
//! - `fit_coastdown`: 구동 없이 굴러갈 때 감속 → 구름 저항 계수 · 점성 계수.
//!
//! 보정 결과는 기록만 하고 기본은 적용하지 않는다(`validated = false`). 적용 여부는 사용자가 정한다.

use crate::crane_profile::G;
use crate::motor::{self, MotorParams};
use serde::{Deserialize, Serialize};

/// Nelder–Mead 단체법 (경계는 목적 함수 안에서 처리)
pub fn nelder_mead(f: &dyn Fn(&[f64]) -> f64, x0: &[f64], step: &[f64], iters: usize, tol: f64) -> (Vec<f64>, f64) {
    let n = x0.len();
    let mut simplex: Vec<Vec<f64>> = vec![x0.to_vec()];
    for i in 0..n {
        let mut p = x0.to_vec();
        p[i] += step[i];
        simplex.push(p);
    }
    let mut vals: Vec<f64> = simplex.iter().map(|p| f(p)).collect();
    for _ in 0..iters {
        let mut idx: Vec<usize> = (0..=n).collect();
        idx.sort_by(|a, b| vals[*a].partial_cmp(&vals[*b]).unwrap_or(std::cmp::Ordering::Equal));
        simplex = idx.iter().map(|i| simplex[*i].clone()).collect();
        vals = idx.iter().map(|i| vals[*i]).collect();
        if (vals[n] - vals[0]).abs() <= tol * (1.0 + vals[0].abs()) {
            break;
        }
        let centroid: Vec<f64> = (0..n).map(|j| simplex[..n].iter().map(|p| p[j]).sum::<f64>() / n as f64).collect();
        let at = |c: f64| -> Vec<f64> { (0..n).map(|j| centroid[j] + c * (simplex[n][j] - centroid[j])).collect() };
        let xr = at(-1.0);
        let fr = f(&xr);
        if fr < vals[0] {
            let xe = at(-2.0);
            let fe = f(&xe);
            if fe < fr {
                simplex[n] = xe;
                vals[n] = fe;
            } else {
                simplex[n] = xr;
                vals[n] = fr;
            }
        } else if fr < vals[n - 1] {
            simplex[n] = xr;
            vals[n] = fr;
        } else {
            let xc = if fr < vals[n] { at(-0.5) } else { at(0.5) };
            let fc = f(&xc);
            if fc < vals[n].min(fr) {
                simplex[n] = xc;
                vals[n] = fc;
            } else {
                for i in 1..=n {
                    for j in 0..n {
                        simplex[i][j] = simplex[0][j] + 0.5 * (simplex[i][j] - simplex[0][j]);
                    }
                    vals[i] = f(&simplex[i]);
                }
            }
        }
    }
    let best = (0..=n).min_by(|a, b| vals[*a].partial_cmp(&vals[*b]).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0);
    (simplex[best].clone(), vals[best])
}

/// S3 보정에서 움직일 열 파라미터
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalKnob {
    TauWinding,
    TauFrame,
    Standstill,
    FrameShare,
}

impl ThermalKnob {
    fn bounds(&self) -> (f64, f64) {
        match self {
            ThermalKnob::TauWinding => (30.0, 3600.0),
            ThermalKnob::TauFrame => (300.0, 20_000.0),
            ThermalKnob::Standstill => (0.05, 1.0),
            ThermalKnob::FrameShare => (0.2, 0.9),
        }
    }
    fn get(&self, m: &MotorParams) -> f64 {
        match self {
            ThermalKnob::TauWinding => m.thermal.tau_winding_s,
            ThermalKnob::TauFrame => m.thermal.tau_frame_s,
            ThermalKnob::Standstill => m.thermal.standstill_cooling,
            ThermalKnob::FrameShare => m.thermal.frame_share,
        }
    }
    fn set(&self, m: &mut MotorParams, v: f64) {
        let (lo, hi) = self.bounds();
        let v = v.clamp(lo, hi);
        match self {
            ThermalKnob::TauWinding => m.thermal.tau_winding_s = v,
            ThermalKnob::TauFrame => m.thermal.tau_frame_s = v,
            ThermalKnob::Standstill => m.thermal.standstill_cooling = v,
            ThermalKnob::FrameShare => m.thermal.frame_share = v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Fit {
    pub motor: String,
    pub knobs: Vec<ThermalKnob>,
    pub tau_winding_s: f64,
    pub tau_frame_s: f64,
    pub standstill_cooling: f64,
    pub frame_share: f64,
    /// (CDF, 목표 배수, 모델 배수)
    pub points: Vec<(f64, f64, f64)>,
    pub rms_error: f64,
    pub cycle_s: f64,
    pub validated: bool,
}

/// S3 출력 배수표에 맞춰 `knobs` 로 고른 열 파라미터만 움직인다. 나머지는 `m` 의 값을 그대로 쓴다.
pub fn fit_s3(m: &MotorParams, targets: &[(f64, f64)], cycle_s: f64, knobs: &[ThermalKnob], iters: usize) -> S3Fit {
    let eval = |p: &[f64]| -> (MotorParams, f64) {
        let mut mm = m.clone();
        let mut pen = 0.0;
        for (k, v) in knobs.iter().zip(p.iter()) {
            let (lo, hi) = k.bounds();
            pen += ((v - v.clamp(lo, hi)) / (hi - lo)).powi(2) * 1e3;
            k.set(&mut mm, *v);
        }
        let mut se = 0.0;
        for (cdf, k) in targets {
            let e = motor::s3_power_factor(&mm, *cdf, cycle_s) - k;
            se += e * e;
        }
        (mm, se / targets.len().max(1) as f64 + pen)
    };
    let f = |p: &[f64]| eval(p).1;
    let x0: Vec<f64> = knobs.iter().map(|k| k.get(m)).collect();
    let step: Vec<f64> = knobs
        .iter()
        .zip(x0.iter())
        .map(|(k, v)| {
            let (lo, hi) = k.bounds();
            (0.3 * v.abs()).max(0.05 * (hi - lo))
        })
        .collect();
    let (x, _) = if knobs.is_empty() { (vec![], 0.0) } else { nelder_mead(&f, &x0, &step, iters, 1e-9) };
    let (mm, _) = eval(&x);
    let points: Vec<(f64, f64, f64)> = targets.iter().map(|(c, k)| (*c, *k, motor::s3_power_factor(&mm, *c, cycle_s))).collect();
    let rms = (points.iter().map(|(_, k, mk)| (mk - k).powi(2)).sum::<f64>() / points.len().max(1) as f64).sqrt();
    S3Fit {
        motor: m.name.clone(),
        knobs: knobs.to_vec(),
        tau_winding_s: mm.thermal.tau_winding_s,
        tau_frame_s: mm.thermal.tau_frame_s,
        standstill_cooling: mm.thermal.standstill_cooling,
        frame_share: mm.thermal.frame_share,
        points,
        rms_error: rms,
        cycle_s,
        validated: false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendulumFit {
    pub damped_freq_hz: f64,
    pub natural_freq_hz: f64,
    pub damping_ratio: f64,
    /// g/ω_n² — 하중 무게중심까지의 등가 길이 (m)
    pub equivalent_length_m: f64,
    pub cycles_used: usize,
    pub validated: bool,
}

/// 흔들림 각(임의 단위) 기록에서 감쇠 진동수 · 감쇠비 · 등가 길이를 구한다
pub fn fit_pendulum(t: &[f64], theta: &[f64]) -> Result<PendulumFit, String> {
    if t.len() != theta.len() || t.len() < 20 {
        return Err("시간과 각도 길이가 같고 20점 이상이어야 합니다".into());
    }
    let mean = theta.iter().sum::<f64>() / theta.len() as f64;
    let y: Vec<f64> = theta.iter().map(|v| v - mean).collect();
    let mut up = Vec::new();
    for i in 1..y.len() {
        if y[i - 1] < 0.0 && y[i] >= 0.0 {
            let w = -y[i - 1] / (y[i] - y[i - 1]);
            up.push(t[i - 1] + w * (t[i] - t[i - 1]));
        }
    }
    if up.len() < 3 {
        return Err("진동 주기를 2개 이상 담은 기록이 필요합니다".into());
    }
    let td = (up[up.len() - 1] - up[0]) / (up.len() - 1) as f64;
    // 주기마다 최대값 → 대수 감쇠율
    let mut peaks = Vec::new();
    for w in up.windows(2) {
        let mut pk = f64::MIN;
        for i in 0..t.len() {
            if t[i] >= w[0] && t[i] < w[1] {
                pk = pk.max(y[i]);
            }
        }
        if pk > 0.0 {
            peaks.push(pk);
        }
    }
    let mut decs = Vec::new();
    for w in peaks.windows(2) {
        if w[0] > 0.0 && w[1] > 0.0 {
            decs.push((w[0] / w[1]).ln());
        }
    }
    let delta = if decs.is_empty() { 0.0 } else { decs.iter().sum::<f64>() / decs.len() as f64 };
    let zeta = (delta / (4.0 * std::f64::consts::PI * std::f64::consts::PI + delta * delta).sqrt()).max(0.0);
    let fd = 1.0 / td;
    let fnat = fd / (1.0 - zeta * zeta).max(1e-9).sqrt();
    let wn = 2.0 * std::f64::consts::PI * fnat;
    Ok(PendulumFit { damped_freq_hz: fd, natural_freq_hz: fnat, damping_ratio: zeta, equivalent_length_m: G / (wn * wn), cycles_used: up.len() - 1, validated: false })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstOrderFit {
    pub y0: f64,
    pub y_inf: f64,
    pub tau_s: f64,
    pub rms_error: f64,
}

/// y(t) = y∞ + (y0 − y∞)·e^(−(t − t0)/τ) — τ 는 로그 격자 + 황금분할, (y0, y∞) 는 선형 최소제곱
pub fn fit_first_order(t: &[f64], y: &[f64]) -> Result<FirstOrderFit, String> {
    if t.len() != y.len() || t.len() < 5 {
        return Err("5점 이상 필요합니다".into());
    }
    let t0 = t[0];
    let span = (t[t.len() - 1] - t0).max(1e-9);
    let solve = |tau: f64| -> (f64, f64, f64) {
        // y = a·e + b·(1 − e), e = exp(−(t−t0)/τ)
        let (mut see, mut sef, mut sff, mut sye, mut syf) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for i in 0..t.len() {
            let e = (-(t[i] - t0) / tau).exp();
            let fz = 1.0 - e;
            see += e * e;
            sef += e * fz;
            sff += fz * fz;
            sye += y[i] * e;
            syf += y[i] * fz;
        }
        let det = see * sff - sef * sef;
        if det.abs() < 1e-18 {
            return (y[0], y[y.len() - 1], f64::MAX);
        }
        let a = (sye * sff - syf * sef) / det;
        let b = (syf * see - sye * sef) / det;
        let mut sse = 0.0;
        for i in 0..t.len() {
            let e = (-(t[i] - t0) / tau).exp();
            sse += (y[i] - a * e - b * (1.0 - e)).powi(2);
        }
        (a, b, sse)
    };
    let mut best = (span, f64::MAX);
    for k in 0..200 {
        let tau = span * 10f64.powf(-3.0 + 4.0 * k as f64 / 199.0);
        let (_, _, sse) = solve(tau);
        if sse < best.1 {
            best = (tau, sse);
        }
    }
    let (mut lo, mut hi) = (best.0 / 1.2, best.0 * 1.2);
    let gr = 0.618_033_988_75;
    for _ in 0..80 {
        let a = hi - gr * (hi - lo);
        let b = lo + gr * (hi - lo);
        if solve(a).2 < solve(b).2 {
            hi = b;
        } else {
            lo = a;
        }
    }
    let tau = 0.5 * (lo + hi);
    let (a, b, sse) = solve(tau);
    Ok(FirstOrderFit { y0: a, y_inf: b, tau_s: tau, rms_error: (sse / t.len() as f64).sqrt() })
}

/// 2단 권선 열 모델: 프레임 상승 ΔT_f' = (a_f·I² + b_f·u − ΔT_f)/τ_f, 권선–프레임 ΔT_wf' = (a_w·I² − ΔT_wf)/τ_w,
/// T_w = T_ref + ΔT_f + ΔT_wf  (I² = 전류 제곱, u = |속도|^1.5 를 정규화한 값, T_ref = 냉각수 또는 주변 온도)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindingFit {
    pub a_w_k_per_a2: f64,
    pub a_f_k_per_a2: f64,
    pub b_f_k: f64,
    pub tau_w_s: f64,
    pub tau_f_s: f64,
    pub speed_ref: f64,
    pub rms_error_k: f64,
    pub samples: usize,
    pub validated: bool,
}

impl WindingFit {
    /// 같은 입력으로 권선 온도를 다시 계산한다
    pub fn simulate(&self, t: &[f64], i2: &[f64], speed: &[f64], t_ref: &[f64], t_w0: f64) -> Vec<f64> {
        simulate_winding(&[self.a_w_k_per_a2, self.a_f_k_per_a2, self.b_f_k, self.tau_w_s, self.tau_f_s], self.speed_ref, t, i2, speed, t_ref, t_w0)
    }

    /// 정격 전류 · 정격 속도에서의 권선 상승 (K) — `MotorParams.thermal.rated_rise_k` 후보
    pub fn rise_at(&self, current_a: f64, speed: f64) -> f64 {
        let u = (speed.abs() / self.speed_ref.max(1e-9)).powf(1.5);
        (self.a_w_k_per_a2 + self.a_f_k_per_a2) * current_a * current_a + self.b_f_k * u
    }
}

fn simulate_winding(p: &[f64], speed_ref: f64, t: &[f64], i2: &[f64], speed: &[f64], t_ref: &[f64], t_w0: f64) -> Vec<f64> {
    let (aw, af, bf, tw, tf) = (p[0], p[1], p[2], p[3].max(1e-3), p[4].max(1e-3));
    let mut out = Vec::with_capacity(t.len());
    // 시작: 권선 초기값 차이를 프레임 쪽에 몰아 둔다
    let mut d_f = (t_w0 - t_ref[0]).max(0.0);
    let mut d_wf = 0.0;
    out.push(t_ref[0] + d_f + d_wf);
    for k in 1..t.len() {
        let dt = (t[k] - t[k - 1]).max(0.0);
        let u = (speed[k - 1].abs() / speed_ref.max(1e-9)).powf(1.5);
        let ef = 1.0 - (-dt / tf).exp();
        let ew = 1.0 - (-dt / tw).exp();
        d_f += ef * (af * i2[k - 1] + bf * u - d_f);
        d_wf += ew * (aw * i2[k - 1] - d_wf);
        out.push(t_ref[k] + d_f + d_wf);
    }
    out
}

/// 권선 온도 기록에 2단 열 모델을 맞춘다
pub fn fit_winding(t: &[f64], current_a: &[f64], speed: &[f64], t_ref: &[f64], t_winding: &[f64], iters: usize) -> Result<WindingFit, String> {
    let n = t.len();
    if [current_a.len(), speed.len(), t_ref.len(), t_winding.len()].iter().any(|l| *l != n) || n < 20 {
        return Err("모든 열의 길이가 같고 20점 이상이어야 합니다".into());
    }
    let i2: Vec<f64> = current_a.iter().map(|i| i * i).collect();
    let speed_ref = speed.iter().fold(0.0f64, |a, b| a.max(b.abs())).max(1e-9);
    let i2_max = i2.iter().cloned().fold(0.0, f64::max).max(1e-9);
    let rise_max = t_winding.iter().zip(t_ref.iter()).map(|(w, r)| w - r).fold(1.0f64, f64::max);
    let span = (t[n - 1] - t[0]).max(1.0);
    let obj = |p: &[f64]| -> f64 {
        if p.iter().take(3).any(|v| *v < 0.0) || p[3] <= 1.0 || p[4] <= 1.0 {
            return 1e12;
        }
        let sim = simulate_winding(p, speed_ref, t, &i2, speed, t_ref, t_winding[0]);
        sim.iter().zip(t_winding.iter()).map(|(a, b)| (a - b).powi(2)).sum::<f64>() / n as f64
    };
    let x0 = [0.4 * rise_max / i2_max, 0.6 * rise_max / i2_max, 0.1 * rise_max, span / 50.0, span / 8.0];
    let step: Vec<f64> = x0.iter().map(|v| 0.5 * v.abs().max(1e-6)).collect();
    let (mut x, mut fx) = nelder_mead(&obj, &x0, &step, iters, 1e-10);
    // 재시작 한 번 (단체법이 납작해졌을 때)
    let step2: Vec<f64> = x.iter().map(|v| 0.3 * v.abs().max(1e-6)).collect();
    let (x2, f2) = nelder_mead(&obj, &x, &step2, iters, 1e-12);
    if f2 < fx {
        x = x2;
        fx = f2;
    }
    Ok(WindingFit {
        a_w_k_per_a2: x[0],
        a_f_k_per_a2: x[1],
        b_f_k: x[2],
        tau_w_s: x[3],
        tau_f_s: x[4],
        speed_ref,
        rms_error_k: fx.max(0.0).sqrt(),
        samples: n,
        validated: false,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoastdownFit {
    pub rolling_resistance: f64,
    pub viscous_n_s_per_m: f64,
    pub samples: usize,
}

/// 구동 없이 굴러가는 구간의 속도 기록 → 감속 a = −(μ·g + (c/M)·v)
pub fn fit_coastdown(t: &[f64], v: &[f64], moving_mass_kg: f64) -> Result<CoastdownFit, String> {
    if t.len() != v.len() || t.len() < 5 {
        return Err("5점 이상 필요합니다".into());
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for i in 1..t.len() - 1 {
        let dt = t[i + 1] - t[i - 1];
        if dt <= 0.0 || v[i].abs() < 1e-3 {
            continue;
        }
        let a = (v[i + 1] - v[i - 1]) / dt * v[i].signum();
        xs.push(v[i].abs());
        ys.push(a);
    }
    if xs.len() < 3 {
        return Err("움직이는 구간이 너무 짧습니다".into());
    }
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let sxx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    let sxy: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| (x - mx) * (y - my)).sum();
    let slope = if sxx > 1e-12 { sxy / sxx } else { 0.0 };
    let icpt = my - slope * mx;
    Ok(CoastdownFit { rolling_resistance: (-icpt / G).max(0.0), viscous_n_s_per_m: (-slope * moving_mass_kg).max(0.0), samples: xs.len() })
}

/// 정밀 로프 해석기(crane_mbd)와 앱의 단진자 모델 비교 (엔드밀 `physcal` 의 자리)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCheck {
    pub crane: String,
    pub rope_length_m: f64,
    pub sling_length_m: f64,
    pub load_kg: f64,
    /// 앱 모델: 걸이점 → 하중 중심까지 길이의 단진자
    pub analytic_freq_hz: f64,
    /// 정밀 해석기: 로프 탄성 · 로프 질량 · 훅–하중 이중 진자를 포함한 1차 흔들림
    pub mbd_freq_hz: f64,
    pub freq_ratio: f64,
    /// 슬링이 있을 때 이중 진자 선형 모드 (Hz)
    pub double_pendulum_modes_hz: Option<(f64, f64)>,
    /// 하중 상하 진동 (로프 탄성) 해석값 (Hz)
    pub bounce_freq_hz: f64,
    pub static_stretch_mm: f64,
    /// 셰이퍼 설계 진동수 보정 제안 (`antisway.freq_error` 에 넣을 값)
    pub suggested_freq_error: f64,
    pub validated: bool,
    pub apply: bool,
    pub notes: Vec<String>,
}

/// 로프 대수 감쇠율 0.359 (Uzunov 2010, 500 kg 호이스트 실측) → 감쇠비 δ/√(4π² + δ²)
pub fn rope_axial_damping_ratio() -> f64 {
    let d: f64 = 0.359;
    d / (4.0 * std::f64::consts::PI * std::f64::consts::PI + d * d).sqrt()
}

pub fn crosscheck(spec: &crate::crane_profile::CraneSpec, rope_length_m: f64, load_kg: f64, sling_length_m: f64) -> CrossCheck {
    use crane_mbd as mbd;
    let falls = spec.hoist.reeving_falls.max(1.0);
    let hook = spec.structure.hook_mass_kg;
    let p = mbd::MbdParams {
        rope_length_m,
        segments: 8,
        rope_mass_per_m: spec.rope.mass_kg_per_m * falls,
        ea_n: spec.rope.axial_stiffness_ea_n * falls,
        axial_damping_ratio: rope_axial_damping_ratio(),
        hook_mass_kg: hook,
        sling_length_m: sling_length_m.max(0.0),
        sling_ea_n: spec.rope.axial_stiffness_ea_n * 2.0,
        load_mass_kg: load_kg.max(1.0),
        load_cd_area_m2: 0.0,
        air_density: 1.2,
    };
    let path = mbd::Path::default();
    let total_l = rope_length_m + sling_length_m.max(0.0);
    let analytic = (G / total_l.max(1e-6)).sqrt() / (2.0 * std::f64::consts::PI);
    let mut notes = Vec::new();
    let (start, modes) = if sling_length_m > 0.0 {
        let (w1, w2) = mbd::double_pendulum_modes(hook.max(1.0), load_kg.max(1.0), rope_length_m, sling_length_m);
        let ratio = mbd::double_pendulum_shape(hook.max(1.0), load_kg.max(1.0), rope_length_m, sling_length_m, w1);
        let tau = 2.0 * std::f64::consts::PI;
        (mbd::Start::Angles { rope_rad: 0.02, sling_rad: 0.02 * ratio }, Some((w1 / tau, w2 / tau)))
    } else {
        (mbd::Start::Tilted { angle_rad: 0.02 }, None)
    };
    let dur = (8.0 / analytic).clamp(20.0, 120.0);
    let r = mbd::run(&p, &path, &start, dur, 0.005);
    let per = mbd::mean_period(&r.t, &mbd::sway_x(&r, &path, true));
    let mbd_f = per.map(|p| 1.0 / p).unwrap_or(f64::NAN);
    if per.is_none() {
        notes.push("정밀 해석에서 진동 주기를 읽지 못했습니다".into());
    }
    let m_total = load_kg.max(1.0) + hook;
    let bounce = (p.ea_n / (rope_length_m.max(1e-6) * m_total)).sqrt() / (2.0 * std::f64::consts::PI);
    let ratio = mbd_f / analytic;
    if let Some((f1, _)) = modes {
        notes.push(format!("훅–하중 이중 진자 1차 모드 {:.4} Hz — 단진자 가정 {:.4} Hz", f1, analytic));
    }
    CrossCheck {
        crane: spec.name.clone(),
        rope_length_m,
        sling_length_m,
        load_kg,
        analytic_freq_hz: analytic,
        mbd_freq_hz: mbd_f,
        freq_ratio: ratio,
        double_pendulum_modes_hz: modes,
        bounce_freq_hz: bounce,
        static_stretch_mm: r.static_stretch_m * 1000.0,
        suggested_freq_error: if ratio.is_finite() { ratio - 1.0 } else { 0.0 },
        validated: false,
        apply: false,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nelder_mead_finds_rosenbrock_minimum() {
        let f = |p: &[f64]| (1.0 - p[0]).powi(2) + 100.0 * (p[1] - p[0] * p[0]).powi(2);
        let (x, fx) = nelder_mead(&f, &[-1.2, 1.0], &[0.5, 0.5], 5000, 1e-14);
        assert!(fx < 1e-8, "{fx}");
        assert!((x[0] - 1.0).abs() < 1e-3 && (x[1] - 1.0).abs() < 1e-3);
    }

    #[test]
    fn pendulum_fit_recovers_frequency_and_damping() {
        let (f, z) = (0.18, 0.02);
        let wn = 2.0 * std::f64::consts::PI * f;
        let wd = wn * (1.0f64 - z * z).sqrt();
        let t: Vec<f64> = (0..6000).map(|i| i as f64 * 0.01).collect();
        let th: Vec<f64> = t.iter().map(|t| 0.03 * (-z * wn * t).exp() * (wd * t).cos()).collect();
        let r = fit_pendulum(&t, &th).unwrap();
        assert!((r.natural_freq_hz - f).abs() < 0.002, "{}", r.natural_freq_hz);
        assert!((r.damping_ratio - z).abs() < 0.004, "{}", r.damping_ratio);
        assert!((r.equivalent_length_m - G / (wn * wn)).abs() < 0.25);
    }

    #[test]
    fn first_order_fit() {
        let t: Vec<f64> = (0..300).map(|i| i as f64 * 10.0).collect();
        let y: Vec<f64> = t.iter().map(|t| 90.0 - 60.0 * (-t / 700.0).exp()).collect();
        let r = fit_first_order(&t, &y).unwrap();
        assert!((r.tau_s - 700.0).abs() < 1.0, "{}", r.tau_s);
        assert!((r.y_inf - 90.0).abs() < 0.01 && (r.y0 - 30.0).abs() < 0.01);
    }

    #[test]
    fn winding_fit_recovers_synthetic_model() {
        let truth = [0.004, 0.006, 8.0, 300.0, 2400.0];
        let t: Vec<f64> = (0..14400).map(|i| i as f64 * 1.0).collect();
        let cur: Vec<f64> = t.iter().map(|t| if ((t / 600.0) as i64) % 2 == 0 { 80.0 } else { 20.0 }).collect();
        let sp: Vec<f64> = t.iter().map(|t| if ((t / 900.0) as i64) % 3 == 0 { 0.0 } else { 3000.0 }).collect();
        let tr: Vec<f64> = vec![40.0; t.len()];
        let i2: Vec<f64> = cur.iter().map(|i| i * i).collect();
        let tw = simulate_winding(&truth, 3000.0, &t, &i2, &sp, &tr, 40.0);
        let fit = fit_winding(&t, &cur, &sp, &tr, &tw, 4000).unwrap();
        assert!(fit.rms_error_k < 0.3, "rms {}", fit.rms_error_k);
        assert!((fit.rise_at(80.0, 3000.0) - (0.01 * 6400.0 + 8.0)).abs() < 3.0, "{}", fit.rise_at(80.0, 3000.0));
    }

    #[test]
    fn crosscheck_single_and_double_pendulum() {
        let spec = crate::crane_profile::gantry_10t_20m();
        let c = crosscheck(&spec, 8.0, 6000.0, 0.0);
        // 단진자: 로프 늘음만큼 조금 느리다 (1% 안)
        assert!((c.freq_ratio - 1.0).abs() < 0.01, "{:?}", c);
        let d = crosscheck(&spec, 6.0, 6000.0, 2.0);
        let (f1, _) = d.double_pendulum_modes_hz.unwrap();
        assert!((d.mbd_freq_hz - f1).abs() / f1 < 0.02, "{:?}", d);
    }

    #[test]
    fn coastdown_fit() {
        let (mu, c, m) = (0.006, 200.0, 30_000.0);
        let mut v = 0.6;
        let dt = 0.05;
        let mut ts = vec![];
        let mut vs = vec![];
        for i in 0..400 {
            ts.push(i as f64 * dt);
            vs.push(v);
            v -= (mu * G + c / m * v) * dt;
            if v <= 0.0 {
                break;
            }
        }
        let r = fit_coastdown(&ts, &vs, m).unwrap();
        assert!((r.rolling_resistance - mu).abs() < 2e-4, "{}", r.rolling_resistance);
        assert!((r.viscous_n_s_per_m - c).abs() < 30.0, "{}", r.viscous_n_s_per_m);
    }
}
