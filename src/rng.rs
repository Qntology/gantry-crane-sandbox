//! 시드가 있는 의사 난수 (xorshift64*) — 날씨 생성 · 현장 운영 시뮬레이션이 같은 시드에서 같은 결과를 내도록.
//!
//! 표준 라이브러리만 쓴다. 정규 분포는 Box–Muller, 와이블 · 지수 · 포아송은 역변환으로 뽑는다.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
    #[serde(default)]
    spare: Option<f64>,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // 0 상태를 피하고 비슷한 시드도 멀리 흩어지게 한다 (splitmix64 한 번)
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        Self { state: z | 1, spare: None }
    }

    /// 이름이 다른 하위 흐름 (날씨 · 사고 · 하중 배분이 서로의 난수 소비에 영향을 주지 않게)
    pub fn fork(&self, salt: u64) -> Self {
        Rng::new(self.state ^ salt.wrapping_mul(0xD6E8_FEB8_6659_FD93))
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// [0, 1)
    pub fn uniform(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) / ((1u64 << 53) as f64)
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.uniform()
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.uniform() < p.clamp(0.0, 1.0)
    }

    /// 표준 정규
    pub fn normal(&mut self) -> f64 {
        if let Some(z) = self.spare.take() {
            return z;
        }
        let u1 = self.uniform().max(1e-300);
        let u2 = self.uniform();
        let r = (-2.0 * u1.ln()).sqrt();
        let a = 2.0 * std::f64::consts::PI * u2;
        self.spare = Some(r * a.sin());
        r * a.cos()
    }

    pub fn exp(&mut self, mean: f64) -> f64 {
        -mean * (1.0 - self.uniform()).max(1e-300).ln()
    }

    /// 감마 분포 (모양 `shape`, 척도 `scale`, 평균 = shape·scale) — Marsaglia & Tsang (2000), shape < 1 은 U^(1/a) 승수
    pub fn gamma(&mut self, shape: f64, scale: f64) -> f64 {
        if shape <= 0.0 || scale <= 0.0 {
            return 0.0;
        }
        if shape < 1.0 {
            let u = self.uniform().max(1e-300);
            return self.gamma(shape + 1.0, scale) * u.powf(1.0 / shape);
        }
        let d = shape - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        loop {
            let x = self.normal();
            let v = (1.0 + c * x).powi(3);
            if v <= 0.0 {
                continue;
            }
            let u = self.uniform().max(1e-300);
            if u.ln() < 0.5 * x * x + d - d * v + d * v.ln() {
                return d * v * scale;
            }
        }
    }

    pub fn poisson(&mut self, lambda: f64) -> u32 {
        if lambda <= 0.0 {
            return 0;
        }
        if lambda > 30.0 {
            return (lambda + lambda.sqrt() * self.normal()).round().max(0.0) as u32;
        }
        let l = (-lambda).exp();
        let mut k = 0u32;
        let mut p = 1.0;
        loop {
            p *= self.uniform();
            if p <= l {
                return k;
            }
            k += 1;
        }
    }
}

/// 표준 정규 누적 분포 (Abramowitz–Stegun 7.1.26, 오차 1.5×10⁻⁷)
pub fn norm_cdf(z: f64) -> f64 {
    let x = z.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let erf = 1.0 - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * (-x * x).exp();
    if z >= 0.0 {
        0.5 * (1.0 + erf)
    } else {
        0.5 * (1.0 - erf)
    }
}

/// 감마 함수 (Lanczos, x > 0)
pub fn gamma(x: f64) -> f64 {
    const G: f64 = 7.0;
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        return std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma(1.0 - x));
    }
    let x = x - 1.0;
    let mut a = C[0];
    let t = x + G + 0.5;
    for (i, c) in C.iter().enumerate().skip(1) {
        a += c / (x + i as f64);
    }
    (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream_and_moments() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut r = Rng::new(7);
        let n = 20_000;
        let xs: Vec<f64> = (0..n).map(|_| r.normal()).collect();
        let m = xs.iter().sum::<f64>() / n as f64;
        let v = xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n as f64;
        assert!(m.abs() < 0.03 && (v - 1.0).abs() < 0.04, "{m} {v}");
        let p: f64 = (0..n).map(|_| r.poisson(3.0) as f64).sum::<f64>() / n as f64;
        assert!((p - 3.0).abs() < 0.06, "{p}");
    }

    #[test]
    fn special_functions() {
        assert!((norm_cdf(0.0) - 0.5).abs() < 1e-9);
        assert!((norm_cdf(1.959_964) - 0.975).abs() < 1e-6);
        assert!((gamma(5.0) - 24.0).abs() < 1e-9);
        assert!((gamma(1.5) - 0.886_226_925).abs() < 1e-8);
    }
}
