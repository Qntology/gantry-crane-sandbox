//! 흔들림 억제: 입력 셰이핑(ZV · ZVD · EI)과 각도 센서 기반 LQR 상태 궤환.
//!
//! 셰이퍼 (K = exp(−ζπ/√(1−ζ²)), T_d = 감쇠 주기):
//! - ZV : A = [1, K]/(1+K),            t = [0, T_d/2]
//! - ZVD: A = [1, 2K, K²]/(1+K)²,      t = [0, T_d/2, T_d]
//! - EI : A = [(1+V)/4, (1−V)/2, (1+V)/4], t = [0, T/2, T] (무감쇠 식). 감쇠가 있으면 [1, K, K²] 를 곱해 다시
//!   정규화하는 근사를 쓴다 (작은 감쇠비 가정).
//! 잔류 진동 V(ω) 는 임펄스 열이 만드는 진동 진폭을 단일 임펄스 대비로 계산한다.
//!
//! LQR: 트롤리 가속도를 입력으로 보는 선형 진자 모델 z = [x−x_ref, ẋ−v_ref, θ, θ̇],
//! θ̈ = −(g/l)·θ − a/l 을 영차 유지로 이산화해 이산 리카티 방정식을 반복으로 푼다.

use crate::crane_profile::G;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AntiSwayMode {
    #[default]
    Off,
    Zv,
    Zvd,
    Ei,
    /// 흔들림 각 센서 + LQR 상태 궤환
    ClosedLoop,
}

impl AntiSwayMode {
    pub fn key(&self) -> &'static str {
        match self {
            AntiSwayMode::Off => "off",
            AntiSwayMode::Zv => "zv",
            AntiSwayMode::Zvd => "zvd",
            AntiSwayMode::Ei => "ei",
            AntiSwayMode::ClosedLoop => "closed_loop",
        }
    }
    pub fn all() -> [AntiSwayMode; 5] {
        [AntiSwayMode::Off, AntiSwayMode::Zv, AntiSwayMode::Zvd, AntiSwayMode::Ei, AntiSwayMode::ClosedLoop]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shaper {
    pub kind: String,
    pub freq_hz: f64,
    pub zeta: f64,
    /// (시각 s, 크기) — 크기 합 1
    pub impulses: Vec<(f64, f64)>,
}

fn k_factor(zeta: f64) -> f64 {
    let z = zeta.clamp(0.0, 0.9);
    (-z * PI / (1.0 - z * z).sqrt()).exp()
}

fn damped_period(freq_hz: f64, zeta: f64) -> f64 {
    let z = zeta.clamp(0.0, 0.9);
    1.0 / (freq_hz.max(1e-9) * (1.0 - z * z).sqrt())
}

impl Shaper {
    pub fn identity() -> Self {
        Self { kind: "none".into(), freq_hz: 0.0, zeta: 0.0, impulses: vec![(0.0, 1.0)] }
    }

    pub fn zv(freq_hz: f64, zeta: f64) -> Self {
        let k = k_factor(zeta);
        let td = damped_period(freq_hz, zeta);
        let s = 1.0 + k;
        Self { kind: "zv".into(), freq_hz, zeta, impulses: vec![(0.0, 1.0 / s), (0.5 * td, k / s)] }
    }

    pub fn zvd(freq_hz: f64, zeta: f64) -> Self {
        let k = k_factor(zeta);
        let td = damped_period(freq_hz, zeta);
        let s = (1.0 + k) * (1.0 + k);
        Self { kind: "zvd".into(), freq_hz, zeta, impulses: vec![(0.0, 1.0 / s), (0.5 * td, 2.0 * k / s), (td, k * k / s)] }
    }

    pub fn ei(freq_hz: f64, zeta: f64, vtol: f64) -> Self {
        let v = vtol.clamp(0.0, 0.5);
        let k = k_factor(zeta);
        let td = damped_period(freq_hz, zeta);
        let raw = [(1.0 + v) / 4.0, (1.0 - v) / 2.0 * k, (1.0 + v) / 4.0 * k * k];
        let s: f64 = raw.iter().sum();
        Self {
            kind: "ei".into(),
            freq_hz,
            zeta,
            impulses: vec![(0.0, raw[0] / s), (0.5 * td, raw[1] / s), (td, raw[2] / s)],
        }
    }

    pub fn for_mode(mode: AntiSwayMode, freq_hz: f64, zeta: f64, vtol: f64) -> Option<Self> {
        match mode {
            AntiSwayMode::Zv => Some(Self::zv(freq_hz, zeta)),
            AntiSwayMode::Zvd => Some(Self::zvd(freq_hz, zeta)),
            AntiSwayMode::Ei => Some(Self::ei(freq_hz, zeta, vtol)),
            _ => None,
        }
    }

    pub fn duration(&self) -> f64 {
        self.impulses.iter().map(|(t, _)| *t).fold(0.0, f64::max)
    }

    /// 실제 진동수 `freq_hz`, 감쇠비 `zeta` 인 계에 대한 잔류 진동 비 (1 = 셰이핑 없음)
    pub fn residual(&self, freq_hz: f64, zeta: f64) -> f64 {
        let wn = 2.0 * PI * freq_hz;
        let z = zeta.clamp(0.0, 0.9);
        let wd = wn * (1.0 - z * z).sqrt();
        let tn = self.duration();
        let (mut c, mut s) = (0.0, 0.0);
        for (t, a) in self.impulses.iter() {
            let e = (z * wn * t).exp();
            c += a * e * (wd * t).cos();
            s += a * e * (wd * t).sin();
        }
        (-z * wn * tn).exp() * (c * c + s * s).sqrt()
    }
}

/// 설계 진동수 주변에서 잔류 진동이 `vtol` 이하인 정규화 진동수 폭 (둔감 폭)
pub fn insensitivity(sh: &Shaper, vtol: f64) -> f64 {
    let f0 = sh.freq_hz.max(1e-9);
    let n = 20_000;
    let (lo, hi) = (0.2, 2.5);
    let mut best = 0.0f64;
    let mut start: Option<f64> = None;
    for i in 0..=n {
        let r = lo + (hi - lo) * i as f64 / n as f64;
        let ok = sh.residual(f0 * r, sh.zeta) <= vtol;
        match (ok, start) {
            (true, None) => start = Some(r),
            (false, Some(s0)) => {
                if s0 <= 1.0 && r >= 1.0 {
                    best = best.max(r - s0);
                }
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s0) = start {
        if s0 <= 1.0 {
            best = best.max(hi - s0);
        }
    }
    // 둔감 영역이 설계점(r = 1)을 포함하지 않는 EI 는 r=1 근처의 연결 구간을 쓴다 → 가장 넓은 구간
    if best == 0.0 {
        let mut s0: Option<f64> = None;
        for i in 0..=n {
            let r = lo + (hi - lo) * i as f64 / n as f64;
            let ok = sh.residual(f0 * r, sh.zeta) <= vtol;
            match (ok, s0) {
                (true, None) => s0 = Some(r),
                (false, Some(a)) => {
                    best = best.max(r - a);
                    s0 = None;
                }
                _ => {}
            }
        }
    }
    best
}

pub fn pendulum_freq_hz(l: f64) -> f64 {
    (G / l.max(1e-6)).sqrt() / (2.0 * PI)
}

type M4 = [[f64; 4]; 4];

fn mm(a: &M4, b: &M4) -> M4 {
    let mut o = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                o[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    o
}

fn mv(a: &M4, v: &[f64; 4]) -> [f64; 4] {
    let mut o = [0.0; 4];
    for i in 0..4 {
        for k in 0..4 {
            o[i] += a[i][k] * v[k];
        }
    }
    o
}

fn transpose(a: &M4) -> M4 {
    let mut o = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            o[i][j] = a[j][i];
        }
    }
    o
}

/// 이산 모델 (A_d, B_d) — 급수 전개 영차 유지
pub fn discrete_model(l: f64, dt: f64) -> (M4, [f64; 4]) {
    let w2 = G / l.max(1e-3);
    let a: M4 = [[0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0], [0.0, 0.0, -w2, 0.0]];
    let b = [0.0, 1.0, 0.0, -1.0 / l.max(1e-3)];
    let mut ad: M4 = [[0.0; 4]; 4];
    let mut ib: M4 = [[0.0; 4]; 4]; // Σ A^k dt^(k+1)/(k+1)!
    let mut term: M4 = [[0.0; 4]; 4];
    for i in 0..4 {
        term[i][i] = 1.0;
    }
    let mut fact = 1.0;
    for k in 0..20 {
        // term = A^k
        let c1 = dt.powi(k as i32) / fact;
        let c2 = dt.powi(k as i32 + 1) / (fact * (k as f64 + 1.0));
        for i in 0..4 {
            for j in 0..4 {
                ad[i][j] += term[i][j] * c1;
                ib[i][j] += term[i][j] * c2;
            }
        }
        term = mm(&term, &a);
        fact *= k as f64 + 1.0;
    }
    (ad, mv(&ib, &b))
}

/// 이산 LQR 이득 K (u = −K·z)
pub fn lqr_gain(l: f64, q: [f64; 4], r: f64, dt: f64) -> [f64; 4] {
    let (a, b) = discrete_model(l, dt);
    let at = transpose(&a);
    let mut p: M4 = [[0.0; 4]; 4];
    for i in 0..4 {
        p[i][i] = q[i];
    }
    let mut k = [0.0; 4];
    for _ in 0..200_000 {
        let pb = mv(&p, &b);
        let s = r + b.iter().zip(pb.iter()).map(|(x, y)| x * y).sum::<f64>();
        // B'PA (행벡터) = (A'PB)'
        let pa = mm(&p, &a);
        let mut bpa = [0.0; 4];
        for j in 0..4 {
            for i in 0..4 {
                bpa[j] += b[i] * pa[i][j];
            }
        }
        for j in 0..4 {
            k[j] = bpa[j] / s;
        }
        // P' = Q + A'PA − (A'PB) K
        let apa = mm(&at, &pa);
        let apb = mv(&at, &pb);
        let mut pn: M4 = [[0.0; 4]; 4];
        let mut diff = 0.0f64;
        for i in 0..4 {
            for j in 0..4 {
                pn[i][j] = apa[i][j] - apb[i] * k[j] + if i == j { q[i] } else { 0.0 };
                diff = diff.max((pn[i][j] - p[i][j]).abs());
            }
        }
        p = pn;
        if diff < 1e-10 * (1.0 + p[0][0].abs()) {
            break;
        }
    }
    k
}

/// 폐루프 극의 크기(스펙트럼 반경) — 1 미만이면 안정
pub fn closed_loop_radius(l: f64, k: &[f64; 4], dt: f64) -> f64 {
    let (a, b) = discrete_model(l, dt);
    let mut acl = a;
    for i in 0..4 {
        for j in 0..4 {
            acl[i][j] -= b[i] * k[j];
        }
    }
    // ‖A^n‖^(1/n) 로 스펙트럼 반경 추정 — 제곱할 때마다 크기를 따로 떼어 넘침을 막는다
    let mut m = acl;
    let mut log_c = 0.0f64;
    let mut n = 1.0f64;
    for _ in 0..20 {
        m = mm(&m, &m);
        n *= 2.0;
        let s = m.iter().flat_map(|r| r.iter()).fold(0.0f64, |a, b| a.max(b.abs()));
        if s <= 0.0 {
            return 0.0;
        }
        for row in m.iter_mut() {
            for x in row.iter_mut() {
                *x /= s;
            }
        }
        log_c = 2.0 * log_c + s.ln();
    }
    let norm: f64 = m.iter().flat_map(|r| r.iter()).map(|x| x * x).sum::<f64>().sqrt();
    ((log_c + norm.ln()) / n).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapers_cancel_at_design_frequency() {
        for sh in [Shaper::zv(0.2, 0.0), Shaper::zvd(0.2, 0.0), Shaper::zv(0.2, 0.05), Shaper::zvd(0.2, 0.05)] {
            let s: f64 = sh.impulses.iter().map(|x| x.1).sum();
            assert!((s - 1.0).abs() < 1e-12);
            assert!(sh.residual(0.2, sh.zeta) < 1e-9, "{} {}", sh.kind, sh.residual(0.2, sh.zeta));
        }
        let ei = Shaper::ei(0.2, 0.0, 0.05);
        assert!((ei.residual(0.2, 0.0) - 0.05).abs() < 1e-9);
    }

    #[test]
    fn insensitivity_ordering_matches_literature() {
        // ZV 5% 둔감 폭 ≈ 0.06, ZVD ≈ 0.28 (Robertson, ACC 2008), EI 가 가장 넓다
        let zv = insensitivity(&Shaper::zv(0.2, 0.0), 0.05);
        let zvd = insensitivity(&Shaper::zvd(0.2, 0.0), 0.05);
        let ei = insensitivity(&Shaper::ei(0.2, 0.0, 0.05), 0.05);
        assert!((zv - 0.064).abs() < 0.005, "zv {zv}");
        assert!((zvd - 0.287).abs() < 0.01, "zvd {zvd}");
        assert!(ei > zvd, "ei {ei} zvd {zvd}");
    }

    #[test]
    fn lqr_stabilizes_pendulum() {
        let l = 8.0;
        for dt in [0.002, 0.01] {
            let k = lqr_gain(l, [10.0, 1.0, 1000.0, 100.0], 1.0, dt);
            let r = closed_loop_radius(l, &k, dt);
            assert!(r < 1.0, "dt {dt}: radius {r}");
        }
        let dt = 0.01;
        // 열린 루프 진자는 감쇠가 없어 반경 1
        let r0 = closed_loop_radius(l, &[0.0; 4], dt);
        assert!(r0 >= 0.999, "{r0}");
    }
}
