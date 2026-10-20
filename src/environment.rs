//! 운전 환경: 주변 온도, 공기 밀도, 바람(평균 + 결정론적 돌풍).
//!
//! 돌풍은 재현성을 위해 정현파 두 개의 합으로 둔다 (난수 없음). 바람 속도는 수평 (x 주행, y 횡행) 성분이다.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub ambient_c: f64,
    pub air_density_kg_m3: f64,
    /// 평균 풍속 벡터 (m/s) — [주행 x, 횡행 y]
    pub wind_mean_mps: [f64; 2],
    /// 돌풍 진폭 (m/s), 평균 풍향(평균이 0 이면 x) 방향
    pub gust_amp_mps: f64,
    /// 돌풍 주기 (s)
    pub gust_period_s: f64,
    /// 바퀴–레일 점착 계수 (젖음 · 결빙 등). 있으면 주행 · 횡행 구동력과 제동력을 μ × 제동 바퀴 하중으로 자른다
    #[serde(default)]
    pub adhesion_mu: Option<f64>,
}

impl Default for Environment {
    fn default() -> Self {
        Self { ambient_c: 25.0, air_density_kg_m3: 1.2, wind_mean_mps: [0.0, 0.0], gust_amp_mps: 0.0, gust_period_s: 8.0, adhesion_mu: None }
    }
}

impl Environment {
    pub fn wind_at(&self, t: f64) -> [f64; 2] {
        let [wx, wy] = self.wind_mean_mps;
        let m = (wx * wx + wy * wy).sqrt();
        let (dx, dy) = if m > 1e-9 { (wx / m, wy / m) } else { (1.0, 0.0) };
        let p = self.gust_period_s.max(0.1);
        let g = self.gust_amp_mps * (0.7 * (2.0 * PI * t / p).sin() + 0.3 * (2.0 * PI * t / (0.37 * p) + 1.1).sin());
        [wx + g * dx, wy + g * dy]
    }

    /// 평균 + 돌풍 진폭 (가장 센 순간 풍속의 상한)
    pub fn wind_peak_mps(&self) -> f64 {
        let [wx, wy] = self.wind_mean_mps;
        (wx * wx + wy * wy).sqrt() + self.gust_amp_mps.abs()
    }

    /// 항력 F = ½ρ·C_d·A·|v|·v (v = 바람 − 물체 속도)
    pub fn drag(&self, cd_area_m2: f64, rel: [f64; 2]) -> [f64; 2] {
        let s = (rel[0] * rel[0] + rel[1] * rel[1]).sqrt();
        let k = 0.5 * self.air_density_kg_m3 * cd_area_m2 * s;
        [k * rel[0], k * rel[1]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calm_has_no_wind() {
        let e = Environment::default();
        assert_eq!(e.wind_at(3.0), [0.0, 0.0]);
        assert_eq!(e.wind_peak_mps(), 0.0);
    }

    #[test]
    fn gust_bounded_by_peak() {
        let e = Environment { wind_mean_mps: [6.0, 8.0], gust_amp_mps: 3.0, ..Default::default() };
        for i in 0..1000 {
            let [x, y] = e.wind_at(i as f64 * 0.05);
            assert!((x * x + y * y).sqrt() <= e.wind_peak_mps() + 1e-9);
        }
    }
}
