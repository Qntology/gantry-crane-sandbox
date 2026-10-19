//! 축별 속도 기준 프로파일: 사다리꼴(가속 한계) → 저크 평활(상자 필터) → 입력 셰이핑(임펄스 열).
//!
//! 세 단계 모두 '면적 1 인 커널과의 합성곱' 이라 이동 거리는 그대로이고, 길이만 커널 길이만큼 늘어난다.
//! 표본 간격은 시뮬레이션 dt 와 같게 두며, 처음과 끝 표본은 0 이다.

use crate::antisway::Shaper;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProfileLimits {
    pub v_max: f64,
    pub a_max: f64,
    /// 0 이면 저크 제한 없음
    pub jerk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub dt: f64,
    /// 부호 있는 속도 표본 (m/s), v[0] = t=0
    pub v: Vec<f64>,
}

impl Profile {
    pub fn zero(dt: f64) -> Self {
        Self { dt, v: vec![0.0, 0.0] }
    }

    pub fn duration(&self) -> f64 {
        self.v.len().saturating_sub(1) as f64 * self.dt
    }

    pub fn distance(&self) -> f64 {
        self.v.windows(2).map(|w| 0.5 * (w[0] + w[1]) * self.dt).sum()
    }

    pub fn peak_speed(&self) -> f64 {
        self.v.iter().fold(0.0f64, |a, b| a.max(b.abs()))
    }

    pub fn peak_accel(&self) -> f64 {
        self.v.windows(2).map(|w| ((w[1] - w[0]) / self.dt).abs()).fold(0.0, f64::max)
    }

    pub fn velocity_at(&self, t: f64) -> f64 {
        if t <= 0.0 || self.v.is_empty() {
            return 0.0;
        }
        let x = t / self.dt;
        let i = x.floor() as usize;
        if i + 1 >= self.v.len() {
            return 0.0;
        }
        let w = x - i as f64;
        self.v[i] * (1.0 - w) + self.v[i + 1] * w
    }

    pub fn accel_at(&self, t: f64) -> f64 {
        if t < 0.0 {
            return 0.0;
        }
        let i = (t / self.dt).floor() as usize;
        if i + 1 >= self.v.len() {
            return 0.0;
        }
        (self.v[i + 1] - self.v[i]) / self.dt
    }

    /// 표본점까지의 누적 이동 거리 (cum[i] = 0..i·dt)
    pub fn cumulative(&self) -> Vec<f64> {
        let mut c = Vec::with_capacity(self.v.len());
        let mut acc = 0.0;
        c.push(0.0);
        for w in self.v.windows(2) {
            acc += 0.5 * (w[0] + w[1]) * self.dt;
            c.push(acc);
        }
        c
    }

    /// 0..t 구간 이동 거리 — `cum` 은 `cumulative()` 결과 (O(1))
    pub fn position_with(&self, cum: &[f64], t: f64) -> f64 {
        if t <= 0.0 || cum.is_empty() {
            return 0.0;
        }
        let n = (t / self.dt).floor() as usize;
        if n + 1 >= self.v.len() {
            return *cum.last().unwrap();
        }
        let rem = t - n as f64 * self.dt;
        cum[n] + 0.5 * (self.v[n] + self.velocity_at(t)) * rem
    }

    /// 0..t 구간 이동 거리 (한 번만 쓸 때)
    pub fn position_at(&self, t: f64) -> f64 {
        self.position_with(&self.cumulative(), t)
    }

    /// 표본 수를 줄여 내보낸다 (간격 `step` s)
    pub fn downsample(&self, step: f64) -> Vec<f64> {
        let k = ((step / self.dt).round() as usize).max(1);
        self.v.iter().step_by(k).cloned().collect()
    }
}

/// 사다리꼴(또는 삼각) 시간: (가속 시간, 등속 시간, 최고 속도)
pub fn trapezoid_times(distance: f64, v_max: f64, a_max: f64) -> (f64, f64, f64) {
    let d = distance.abs();
    if d <= 1e-12 || v_max <= 0.0 || a_max <= 0.0 {
        return (0.0, 0.0, 0.0);
    }
    if d >= v_max * v_max / a_max {
        (v_max / a_max, (d - v_max * v_max / a_max) / v_max, v_max)
    } else {
        let vp = (d * a_max).sqrt();
        (vp / a_max, 0.0, vp)
    }
}

/// 사다리꼴 속도 프로파일을 표본화한다. 표본화 오차는 크기 보정으로 없애 이동 거리를 정확히 맞춘다.
pub fn sample_trapezoid(distance: f64, lim: &ProfileLimits, dt: f64) -> Profile {
    let (ta, tf, vp) = trapezoid_times(distance, lim.v_max, lim.a_max);
    if vp == 0.0 {
        return Profile::zero(dt);
    }
    let total = 2.0 * ta + tf;
    let n = (total / dt).ceil() as usize + 1;
    let sign = distance.signum();
    let mut v: Vec<f64> = (0..=n)
        .map(|i| {
            let t = i as f64 * dt;
            let s = if t < ta {
                lim.a_max * t
            } else if t < ta + tf {
                vp
            } else if t < total {
                vp - lim.a_max * (t - ta - tf)
            } else {
                0.0
            };
            sign * s.max(0.0)
        })
        .collect();
    if let Some(last) = v.last_mut() {
        *last = 0.0;
    }
    let p = Profile { dt, v: v.clone() };
    let got = p.distance();
    if got.abs() > 1e-12 {
        let k = distance / got;
        for x in v.iter_mut() {
            *x *= k;
        }
    }
    Profile { dt, v }
}

/// 상자 필터(이동 평균) 합성곱 — 가속도가 폭 동안 선형으로 오르므로 저크 = a/폭
pub fn smooth_box(p: &Profile, width_s: f64) -> Profile {
    let k = (width_s / p.dt).round() as usize;
    if k <= 1 {
        return p.clone();
    }
    let n = p.v.len();
    let mut out = vec![0.0; n + k - 1];
    let mut acc = 0.0;
    for i in 0..out.len() {
        if i < n {
            acc += p.v[i];
        }
        if i >= k && i - k < n {
            acc -= p.v[i - k];
        }
        out[i] = acc / k as f64;
    }
    if let Some(last) = out.last_mut() {
        *last = 0.0;
    }
    Profile { dt: p.dt, v: out }
}

/// 입력 셰이퍼 합성곱: out(t) = Σ A_i·v(t − t_i)
pub fn shape(p: &Profile, shaper: &Shaper) -> Profile {
    if shaper.impulses.len() <= 1 {
        return p.clone();
    }
    let shifts: Vec<(usize, f64)> = shaper.impulses.iter().map(|(t, a)| ((t / p.dt).round() as usize, *a)).collect();
    let max_shift = shifts.iter().map(|s| s.0).max().unwrap_or(0);
    let n = p.v.len();
    let mut out = vec![0.0; n + max_shift];
    for (sh, a) in shifts.iter() {
        for i in 0..n {
            out[i + sh] += a * p.v[i];
        }
    }
    Profile { dt: p.dt, v: out }
}

/// 거리 + 한계 + (선택) 셰이퍼로 축 기준 프로파일을 만든다
pub fn build(distance: f64, lim: &ProfileLimits, shaper: Option<&Shaper>, dt: f64) -> Profile {
    let mut p = sample_trapezoid(distance, lim, dt);
    if lim.jerk > 0.0 && lim.a_max > 0.0 {
        p = smooth_box(&p, lim.a_max / lim.jerk);
    }
    if let Some(s) = shaper {
        p = shape(&p, s);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::antisway::Shaper;

    #[test]
    fn trapezoid_hits_distance_and_limits() {
        let lim = ProfileLimits { v_max: 0.5, a_max: 0.2, jerk: 0.0 };
        for d in [0.3, 2.0, 15.0, -7.5] {
            let p = sample_trapezoid(d, &lim, 0.002);
            assert!((p.distance() - d).abs() < 1e-9, "{d} {}", p.distance());
            assert!(p.peak_speed() <= 0.5 * (1.0 + 1e-3));
            assert!(p.peak_accel() <= 0.2 * 1.01);
            assert_eq!(p.v[0], 0.0);
            assert_eq!(*p.v.last().unwrap(), 0.0);
        }
    }

    #[test]
    fn smoothing_and_shaping_keep_distance() {
        let lim = ProfileLimits { v_max: 0.6, a_max: 0.25, jerk: 0.5 };
        let sh = Shaper::zvd(0.2, 0.0);
        let p = build(12.0, &lim, Some(&sh), 0.002);
        assert!((p.distance() - 12.0).abs() < 1e-6, "{}", p.distance());
        // 저크 평활로 가속 한계를 넘지 않는다
        assert!(p.peak_accel() <= 0.25 * 1.01);
        // 셰이퍼 길이만큼 늘어난다
        let plain = build(12.0, &ProfileLimits { jerk: 0.0, ..lim }, None, 0.002);
        assert!(p.duration() > plain.duration() + sh.duration() - 0.01);
    }

    #[test]
    fn position_integrates_velocity() {
        let lim = ProfileLimits { v_max: 0.5, a_max: 0.2, jerk: 0.0 };
        let p = sample_trapezoid(5.0, &lim, 0.002);
        assert!((p.position_at(p.duration() + 1.0) - 5.0).abs() < 1e-9);
        assert!((p.position_at(p.duration() * 0.5) - 2.5).abs() < 1e-3);
        let cum = p.cumulative();
        for t in [0.0, 0.37, 3.3, 7.77, p.duration()] {
            let brute: f64 = {
                let n = (t / p.dt).floor() as usize;
                let mut s = 0.0;
                for i in 0..n.min(p.v.len() - 1) {
                    s += 0.5 * (p.v[i] + p.v[i + 1]) * p.dt;
                }
                if n + 1 < p.v.len() {
                    s += 0.5 * (p.v[n] + p.velocity_at(t)) * (t - n as f64 * p.dt);
                }
                s
            };
            assert!((p.position_with(&cum, t) - brute).abs() < 1e-12);
        }
    }
}
