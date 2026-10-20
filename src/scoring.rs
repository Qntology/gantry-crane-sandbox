//! 확률 예측 점수 — 남은 수명 분포 · 기간 내 고장 확률 · 분위수 예측을 재는 지표와 교차검증 도구.
//!
//! | 지표 | 식 | 출처 |
//! | --- | --- | --- |
//! | 핀볼(분위수) 손실 | L_τ(y, q) = τ(y − q) (y ≥ q), (1 − τ)(q − y) (y < q) | Koenker & Bassett 1978 |
//! | CRPS | ∫(F(x) − 1{x ≥ y})² dx = 2∫₀¹ L_τ(y, F⁻¹(τ)) dτ ≈ (2/K) Σ L_τk | Gneiting & Raftery 2007 |
//! | 구간 점수 | (u − l) + (2/α)(l − y)·1{y < l} + (2/α)(y − u)·1{y > u} (중앙 1 − α 구간) | Gneiting & Raftery 2007 |
//! | PIT · KS 거리 | u = F(y) 가 균등 분포면 보정이 맞다 | Dawid 1984 |
//! | Brier · 분해 | BS = mean (p − o)², BS = 신뢰도 − 분해능 + 불확실성 | Brier 1950, Murphy 1973 |
//! | α–λ 정확도 · 예측 지평 | 참 남은 수명의 ±α 안에 드는가 | Saxena et al. 2008 |
//! | 적응 등각 추론 | α_{t+1} = α_t + γ(α − err_t) | Gibbs & Candès 2021 |

use serde::{Deserialize, Serialize};

/// 핀볼 손실
pub fn pinball(y: f64, q: f64, tau: f64) -> f64 {
    if y >= q {
        tau * (y - q)
    } else {
        (1.0 - tau) * (q - y)
    }
}

/// 분위수 묶음 (τ 오름차순, q 오름차순)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantileForecast {
    pub taus: Vec<f64>,
    pub qs: Vec<f64>,
}

impl QuantileForecast {
    pub fn new(taus: &[f64], mut qs: Vec<f64>) -> Self {
        // 분위수 교차를 정렬로 없앤다 (재배열, Chernozhukov et al. 2010)
        qs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Self { taus: taus.to_vec(), qs }
    }

    /// τ 에서의 분위수 (선형 보간, 바깥은 끝 기울기로 늘림)
    pub fn quantile(&self, tau: f64) -> f64 {
        let n = self.taus.len();
        if n == 0 {
            return f64::NAN;
        }
        if n == 1 {
            return self.qs[0];
        }
        let k = match self.taus.iter().position(|t| *t >= tau) {
            Some(0) => 1,
            Some(k) => k,
            None => n - 1,
        };
        let (t0, t1, q0, q1) = (self.taus[k - 1], self.taus[k], self.qs[k - 1], self.qs[k]);
        if (t1 - t0).abs() < 1e-12 {
            return q0;
        }
        q0 + (q1 - q0) * (tau - t0) / (t1 - t0)
    }

    pub fn median(&self) -> f64 {
        self.quantile(0.5)
    }

    /// 예측 분포의 F(y) (분위수 사이 선형, 끝 바깥은 0 · 1 로 자름)
    pub fn cdf(&self, y: f64) -> f64 {
        let n = self.qs.len();
        if n == 0 {
            return f64::NAN;
        }
        if y <= self.qs[0] {
            // 첫 분위수 아래: 0 과 τ₀ 사이를 첫 칸 기울기로
            if n >= 2 && self.qs[1] > self.qs[0] {
                let slope = (self.taus[1] - self.taus[0]) / (self.qs[1] - self.qs[0]);
                return (self.taus[0] - slope * (self.qs[0] - y)).max(0.0);
            }
            return if y < self.qs[0] { 0.0 } else { self.taus[0] };
        }
        if y >= self.qs[n - 1] {
            if n >= 2 && self.qs[n - 1] > self.qs[n - 2] {
                let slope = (self.taus[n - 1] - self.taus[n - 2]) / (self.qs[n - 1] - self.qs[n - 2]);
                return (self.taus[n - 1] + slope * (y - self.qs[n - 1])).min(1.0);
            }
            return if y > self.qs[n - 1] { 1.0 } else { self.taus[n - 1] };
        }
        for k in 1..n {
            if y <= self.qs[k] {
                let (q0, q1) = (self.qs[k - 1], self.qs[k]);
                if q1 - q0 < 1e-12 {
                    return self.taus[k];
                }
                return self.taus[k - 1] + (self.taus[k] - self.taus[k - 1]) * (y - q0) / (q1 - q0);
            }
        }
        1.0
    }

    /// CRPS 근사: (2/K) Σ L_τk — τ 가 (k − 0.5)/K 처럼 고르게 놓였을 때 맞다
    pub fn crps(&self, y: f64) -> f64 {
        let k = self.taus.len().max(1) as f64;
        2.0 / k * self.taus.iter().zip(self.qs.iter()).map(|(t, q)| pinball(y, *q, *t)).sum::<f64>()
    }

    /// 평균 핀볼 손실
    pub fn mean_pinball(&self, y: f64) -> f64 {
        let k = self.taus.len().max(1) as f64;
        self.taus.iter().zip(self.qs.iter()).map(|(t, q)| pinball(y, *q, *t)).sum::<f64>() / k
    }
}

/// 고르게 놓인 τ 격자 (k − 0.5)/K
pub fn uniform_taus(k: usize) -> Vec<f64> {
    (0..k).map(|i| (i as f64 + 0.5) / k as f64).collect()
}

/// 구간 점수 (중앙 1 − α 구간 [l, u])
pub fn interval_score(y: f64, l: f64, u: f64, alpha: f64) -> f64 {
    let a = alpha.clamp(1e-6, 1.0);
    let mut s = u - l;
    if y < l {
        s += 2.0 / a * (l - y);
    }
    if y > u {
        s += 2.0 / a * (y - u);
    }
    s
}

/// 균등 분포와의 KS 거리
pub fn ks_uniform(u: &[f64]) -> f64 {
    let n = u.len();
    if n == 0 {
        return f64::NAN;
    }
    let mut v: Vec<f64> = u.iter().map(|x| x.clamp(0.0, 1.0)).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let nf = n as f64;
    v.iter().enumerate().map(|(i, x)| ((i as f64 + 1.0) / nf - x).max(x - i as f64 / nf)).fold(0.0, f64::max)
}

/// PIT 히스토그램 (칸별 비율)
pub fn histogram01(u: &[f64], bins: usize) -> Vec<f64> {
    let b = bins.max(1);
    let mut h = vec![0.0; b];
    for x in u {
        let k = ((x.clamp(0.0, 1.0) * b as f64) as usize).min(b - 1);
        h[k] += 1.0;
    }
    let n = u.len().max(1) as f64;
    h.iter().map(|c| c / n).collect()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityBin {
    pub p_mean: f64,
    pub observed: f64,
    pub count: usize,
}

/// Brier 점수와 Murphy 분해
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrierReport {
    pub n: usize,
    pub brier: f64,
    pub reliability: f64,
    pub resolution: f64,
    pub uncertainty: f64,
    /// 기후값(평균 발생률) 대비 기술 점수 1 − BS/BS_clim
    pub skill: f64,
    pub base_rate: f64,
    pub bins: Vec<ReliabilityBin>,
}

pub fn brier(p: &[f64], o: &[bool], bins: usize) -> BrierReport {
    let n = p.len().min(o.len());
    if n == 0 {
        return BrierReport::default();
    }
    let bs = (0..n).map(|i| (p[i] - if o[i] { 1.0 } else { 0.0 }).powi(2)).sum::<f64>() / n as f64;
    let obar = o.iter().take(n).filter(|x| **x).count() as f64 / n as f64;
    let b = bins.max(1);
    let mut acc = vec![(0.0f64, 0.0f64, 0usize); b];
    for i in 0..n {
        let k = ((p[i].clamp(0.0, 1.0) * b as f64) as usize).min(b - 1);
        acc[k].0 += p[i];
        acc[k].1 += if o[i] { 1.0 } else { 0.0 };
        acc[k].2 += 1;
    }
    let (mut rel, mut res) = (0.0, 0.0);
    let mut out = Vec::new();
    for (ps, os, c) in acc {
        if c == 0 {
            continue;
        }
        let (pm, om) = (ps / c as f64, os / c as f64);
        rel += c as f64 * (pm - om).powi(2);
        res += c as f64 * (om - obar).powi(2);
        out.push(ReliabilityBin { p_mean: pm, observed: om, count: c });
    }
    let unc = obar * (1.0 - obar);
    BrierReport {
        n,
        brier: bs,
        reliability: rel / n as f64,
        resolution: res / n as f64,
        uncertainty: unc,
        skill: if unc > 1e-12 { 1.0 - bs / unc } else { 0.0 },
        base_rate: obar,
        bins: out,
    }
}

/// 분위수 예측 묶음의 점수 요약
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantileScore {
    pub n: usize,
    pub crps: f64,
    pub pinball: f64,
    /// 중앙값 절대 오차
    pub mae_median: f64,
    /// 80% 구간(τ 0.1–0.9) 적중률 · 평균 폭 · 구간 점수
    pub coverage80: f64,
    pub width80: f64,
    pub interval80: f64,
    /// 50% 구간 적중률
    pub coverage50: f64,
    pub pit_ks: f64,
    pub pit_hist: Vec<f64>,
}

pub fn score_quantiles(fc: &[QuantileForecast], y: &[f64]) -> QuantileScore {
    let n = fc.len().min(y.len());
    if n == 0 {
        return QuantileScore::default();
    }
    let mut s = QuantileScore { n, ..Default::default() };
    let mut pits = Vec::with_capacity(n);
    let (mut c80, mut c50) = (0.0, 0.0);
    for i in 0..n {
        let f = &fc[i];
        let yy = y[i];
        s.crps += f.crps(yy);
        s.pinball += f.mean_pinball(yy);
        s.mae_median += (f.median() - yy).abs();
        let (l, u) = (f.quantile(0.1), f.quantile(0.9));
        if yy >= l && yy <= u {
            c80 += 1.0;
        }
        s.width80 += u - l;
        s.interval80 += interval_score(yy, l, u, 0.2);
        let (l5, u5) = (f.quantile(0.25), f.quantile(0.75));
        if yy >= l5 && yy <= u5 {
            c50 += 1.0;
        }
        pits.push(f.cdf(yy));
    }
    let nf = n as f64;
    s.crps /= nf;
    s.pinball /= nf;
    s.mae_median /= nf;
    s.coverage80 = c80 / nf;
    s.width80 /= nf;
    s.interval80 /= nf;
    s.coverage50 = c50 / nf;
    s.pit_ks = ks_uniform(&pits);
    s.pit_hist = histogram01(&pits, 10);
    s
}

/// Saxena et al. (2008) α–λ 정확도: 시점 t_λ = t_P + λ(EOL − t_P) 의 예측 남은 수명이 참값의 [1 − α, 1 + α] 안인가
pub fn alpha_lambda_ok(pred_rul: f64, true_rul: f64, alpha: f64) -> bool {
    let a = alpha.max(0.0);
    pred_rul >= (1.0 - a) * true_rul && pred_rul <= (1.0 + a) * true_rul
}

/// Saxena et al. (2008) 예측 지평: 그 뒤 모든 예측이 참 남은 수명 ± α·EOL 안에 드는 첫 시점부터 EOL 까지 (같은 단위)
/// `pts` = (시점, 예측 남은 수명), `eol` = 실제 고장 시점
pub fn prognostic_horizon(pts: &[(f64, f64)], eol: f64, alpha: f64) -> Option<f64> {
    let band = alpha.max(0.0) * eol.abs();
    let mut first: Option<f64> = None;
    for (t, r) in pts.iter() {
        let truth = eol - t;
        if (r - truth).abs() <= band {
            if first.is_none() {
                first = Some(*t);
            }
        } else {
            first = None;
        }
    }
    first.map(|t| eol - t)
}

/// 분할 등각 예측: 보정 집합 점수(|오차| 등)의 (1 − α)(1 + 1/n) 분위수
pub fn conformal_quantile(scores: &[f64], alpha: f64) -> f64 {
    let n = scores.len();
    if n == 0 {
        return f64::NAN;
    }
    let mut v = scores.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let level = ((1.0 - alpha) * (1.0 + 1.0 / n as f64)).clamp(0.0, 1.0);
    let k = ((level * n as f64).ceil() as usize).clamp(1, n) - 1;
    v[k]
}

/// 적응 등각 추론 한 걸음 (Gibbs & Candès 2021): 빗나가면 α_t 를 줄여 구간을 넓힌다
pub fn aci_update(alpha_t: f64, target_alpha: f64, missed: bool, gamma: f64) -> f64 {
    let err = if missed { 1.0 } else { 0.0 };
    (alpha_t + gamma * (target_alpha - err)).clamp(0.001, 0.999)
}

/// 시간 순 교차검증의 예측 시점 (학습 최소 길이 · 간격)
pub fn rolling_origins(n: usize, min_train: usize, step: usize) -> Vec<usize> {
    let s = step.max(1);
    (min_train..n).step_by(s).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    #[test]
    fn pinball_and_interval_score() {
        assert!((pinball(10.0, 8.0, 0.9) - 1.8).abs() < 1e-12);
        assert!((pinball(6.0, 8.0, 0.9) - 0.2).abs() < 1e-12);
        assert!((interval_score(5.0, 4.0, 6.0, 0.2) - 2.0).abs() < 1e-12);
        assert!((interval_score(3.0, 4.0, 6.0, 0.2) - 12.0).abs() < 1e-12);
    }

    #[test]
    fn crps_of_normal_quantiles_matches_closed_form() {
        // N(0,1) 의 CRPS at y: y(2Φ(y) − 1) + 2φ(y) − 1/√π
        let taus = uniform_taus(99);
        let qs: Vec<f64> = taus.iter().map(|t| inv_norm(*t)).collect();
        let f = QuantileForecast::new(&taus, qs);
        for y in [0.0f64, 0.7, -1.5] {
            let phi = (-0.5 * y * y).exp() / (2.0 * std::f64::consts::PI).sqrt();
            let exact = y * (2.0 * crate::rng::norm_cdf(y) - 1.0) + 2.0 * phi - 1.0 / std::f64::consts::PI.sqrt();
            assert!((f.crps(y) - exact).abs() < 0.01, "y={y}: {} vs {exact}", f.crps(y));
        }
    }

    fn inv_norm(p: f64) -> f64 {
        // 이분법 (테스트용)
        let (mut lo, mut hi) = (-10.0, 10.0);
        for _ in 0..80 {
            let m = 0.5 * (lo + hi);
            if crate::rng::norm_cdf(m) < p {
                lo = m;
            } else {
                hi = m;
            }
        }
        0.5 * (lo + hi)
    }

    #[test]
    fn calibrated_forecasts_have_uniform_pit_and_nominal_coverage() {
        let taus = uniform_taus(19);
        let mut rng = Rng::new(3);
        let mut fc = Vec::new();
        let mut ys = Vec::new();
        for _ in 0..4000 {
            let mu = rng.normal() * 3.0;
            let qs: Vec<f64> = taus.iter().map(|t| mu + inv_norm(*t)).collect();
            fc.push(QuantileForecast::new(&taus, qs));
            ys.push(mu + rng.normal());
        }
        let s = score_quantiles(&fc, &ys);
        assert!((s.coverage80 - 0.8).abs() < 0.03, "{}", s.coverage80);
        assert!((s.coverage50 - 0.5).abs() < 0.03, "{}", s.coverage50);
        assert!(s.pit_ks < 0.04, "{}", s.pit_ks);
        // 너무 좁은 예측은 적중률이 떨어지고 PIT 가 U 자가 된다
        let narrow: Vec<QuantileForecast> = fc.iter().map(|f| QuantileForecast::new(&f.taus, f.qs.iter().map(|q| f.median() + 0.3 * (q - f.median())).collect())).collect();
        let sn = score_quantiles(&narrow, &ys);
        assert!(sn.coverage80 < 0.5 && sn.pit_ks > s.pit_ks && sn.crps > s.crps);
    }

    #[test]
    fn brier_decomposition_adds_up() {
        let mut rng = Rng::new(9);
        let (mut p, mut o) = (Vec::new(), Vec::new());
        for _ in 0..5000 {
            let q = rng.uniform();
            p.push(q);
            o.push(rng.chance(q));
        }
        let b = brier(&p, &o, 10);
        assert!((b.brier - (b.reliability - b.resolution + b.uncertainty)).abs() < 0.01, "{b:?}");
        assert!(b.reliability < 0.005 && b.skill > 0.2, "{b:?}");
    }

    #[test]
    fn horizon_and_conformal() {
        // 참 EOL 100, t ≥ 40 부터 예측이 ±10 안
        let pts: Vec<(f64, f64)> = (0..10).map(|k| (k as f64 * 10.0, if k < 4 { 30.0 } else { 100.0 - k as f64 * 10.0 + 3.0 })).collect();
        assert_eq!(prognostic_horizon(&pts, 100.0, 0.1), Some(60.0));
        assert!(alpha_lambda_ok(55.0, 50.0, 0.2) && !alpha_lambda_ok(65.0, 50.0, 0.2));
        let q = conformal_quantile(&(1..=99).map(|x| x as f64).collect::<Vec<_>>(), 0.1);
        assert!((q - 90.0).abs() <= 1.0, "{q}");
        let a = aci_update(0.1, 0.1, true, 0.005);
        assert!(a < 0.1);
    }
}
