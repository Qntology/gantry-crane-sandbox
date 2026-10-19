//! 권상 와이어로프: 안전율, D/d, 굽힘 피로 누적 (Palmgren–Miner).
//!
//! 굽힘 피로 수명 N(폐기까지 굽힘 횟수)은 세 가지 모델 중 하나로 계산한다.
//! - `FeyrerUpdate`: IFT Stuttgart Feyrer 갱신 자료(2011) 표 3.14 의 상수 a0–a3 꼴 (기본값).
//!   lg N = a0 + (a1 + a3·lg(D/d))·(lg((S/d²)·1770/R0) − 0.4·lg(R0/1770)) + a2·lg(D/d) + lg f_d + lg f_l
//!   항 배치는 OCR 품질이 낮은 갱신 자료에서 재구성한 것이므로 Feyrer 2015 원서로 확인해야 한다.
//! - `Feyrer`: 예전 b0..b5 상수 꼴.
//!   lg N = b0 + (b1 + b4·lg(D/d))·(lg(S/d²) − 0.4·lg(R0/1770)) + b2·lg(D/d) + b3·lg d + 1/(b5 + lg(l/d))
//!   (S: 줄당 장력 N, d: 공칭 직경 mm, D: 시브 직경 mm, R0: 소선 인장강도 N/mm², l: 굽힘 길이 mm)
//!   상수 b0..b5 는 로프 구조마다 다르므로 로프 제조사·문헌 값을 넣어야 한다.
//! - `PowerLaw`: N = n_ref·(S_ref/S)^k·((D/d)/(D/d)_ref)^m — 굽힘 시험 데이터에 맞추는 단순 모델.
//!
//! 위험 구간은 권상 한 행정마다 `sheave_count` 개 시브를 지난다고 보고, 행정당 손상 n/N 을 더한다.
//! 손상 합 D ≥ 1 이면 폐기 기준(ISO 4309 검사로 확인) 도달로 본다.

use serde::{Deserialize, Serialize};

fn one() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "model", rename_all = "snake_case")]
pub enum RopeFatigue {
    FeyrerUpdate {
        a0: f64,
        a1: f64,
        a2: f64,
        a3: f64,
        /// 직경 보정 f_d(d/16) · 굽힘 길이 보정 f_l(l/d) — 갱신 자료의 유효 범위(d ≈ 16 mm, l ≥ 10d)에서 1
        #[serde(default = "one")]
        f_d: f64,
        #[serde(default = "one")]
        f_l: f64,
        source: String,
    },
    Feyrer {
        b0: f64,
        b1: f64,
        b2: f64,
        b3: f64,
        b4: f64,
        b5: f64,
        /// 한 번에 굽혀지는 로프 길이 (mm)
        bending_length_mm: f64,
        /// 이 상수의 출처 (로프 구조 · 문헌)
        source: String,
    },
    PowerLaw {
        n_ref: f64,
        s_ref_kn: f64,
        k: f64,
        dd_ref: f64,
        m: f64,
        source: String,
    },
}

impl RopeFatigue {
    /// 상수의 출처 설명
    pub fn source(&self) -> &str {
        match self {
            RopeFatigue::FeyrerUpdate { source, .. } | RopeFatigue::Feyrer { source, .. } | RopeFatigue::PowerLaw { source, .. } => source,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RopeParams {
    pub construction: String,
    pub diameter_mm: f64,
    pub min_breaking_force_kn: f64,
    pub tensile_grade_mpa: f64,
    pub sheave_diameter_mm: f64,
    pub drum_diameter_mm: f64,
    /// 위험 구간이 권상 한 행정에 지나는 시브 수
    pub sheave_count: u32,
    pub mass_kg_per_m: f64,
    /// 축 강성 EA (N) — 정밀 해석기(crane_mbd)의 로프 탄성
    pub axial_stiffness_ea_n: f64,
    /// 최소 안전율 (기계 등급별 설계 기준값으로 교체)
    pub min_safety_factor: f64,
    pub fatigue: RopeFatigue,
}

impl RopeParams {
    pub fn d_ratio_sheave(&self) -> f64 {
        self.sheave_diameter_mm / self.diameter_mm.max(1e-9)
    }

    pub fn d_ratio_drum(&self) -> f64 {
        self.drum_diameter_mm / self.diameter_mm.max(1e-9)
    }

    /// 줄당 장력 S (N) 에서의 안전율
    pub fn safety_factor(&self, s_per_fall_n: f64) -> f64 {
        if s_per_fall_n <= 0.0 {
            return f64::INFINITY;
        }
        self.min_breaking_force_kn * 1000.0 / s_per_fall_n
    }

    /// 줄당 장력 S (N) 에서 폐기까지의 굽힘 횟수
    pub fn bending_cycles_to_discard(&self, s_per_fall_n: f64) -> f64 {
        let s = s_per_fall_n.max(1.0);
        let d = self.diameter_mm.max(1e-6);
        let dd = self.d_ratio_sheave().max(1.0);
        match &self.fatigue {
            RopeFatigue::FeyrerUpdate { a0, a1, a2, a3, f_d, f_l, .. } => {
                let lg = |x: f64| x.max(1e-12).log10();
                let r0 = self.tensile_grade_mpa.max(1.0);
                let stress = lg(s / (d * d) * 1770.0 / r0) - 0.4 * lg(r0 / 1770.0);
                let lgn = a0 + (a1 + a3 * lg(dd)) * stress + a2 * lg(dd) + lg(*f_d) + lg(*f_l);
                10f64.powf(lgn)
            }
            RopeFatigue::Feyrer { b0, b1, b2, b3, b4, b5, bending_length_mm, .. } => {
                let lg = |x: f64| x.max(1e-12).log10();
                let lterm = lg(bending_length_mm / d);
                let tail = if (b5 + lterm).abs() < 1e-9 { 0.0 } else { 1.0 / (b5 + lterm) };
                let lgn = b0
                    + (b1 + b4 * lg(dd)) * (lg(s / (d * d)) - 0.4 * lg(self.tensile_grade_mpa / 1770.0))
                    + b2 * lg(dd)
                    + b3 * lg(d)
                    + tail;
                10f64.powf(lgn)
            }
            RopeFatigue::PowerLaw { n_ref, s_ref_kn, k, dd_ref, m, .. } => {
                n_ref * (s_ref_kn * 1000.0 / s).powf(*k) * (dd / dd_ref.max(1e-9)).powf(*m)
            }
        }
    }

    /// 줄당 장력 S 로 권상 한 행정을 했을 때의 손상 증분
    pub fn damage_per_stroke(&self, s_per_fall_n: f64) -> f64 {
        let n = self.bending_cycles_to_discard(s_per_fall_n);
        if !(n.is_finite() && n > 0.0) {
            return 0.0;
        }
        self.sheave_count as f64 / n
    }
}

/// 권상 행정 계수기: 방향이 바뀌거나 멈출 때 하나의 행정으로 묶는다
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RopeCounter {
    pub strokes: u64,
    pub damage: f64,
    pub max_tension_per_fall_n: f64,
    stroke_len_m: f64,
    stroke_dir: i8,
    stroke_peak_s: f64,
}

impl RopeCounter {
    /// 최소 행정 길이 (m) — 이보다 짧은 움직임은 위험 구간이 시브를 다 지나지 않는다고 본다
    pub const MIN_STROKE_M: f64 = 0.5;

    /// 권상 속도 l̇ (m/s, +는 하강), 줄당 장력 S 로 dt 를 진행한다
    pub fn step(&mut self, rope: &RopeParams, l_dot: f64, s_per_fall_n: f64, dt: f64) {
        self.max_tension_per_fall_n = self.max_tension_per_fall_n.max(s_per_fall_n);
        let dir: i8 = if l_dot > 1e-4 {
            1
        } else if l_dot < -1e-4 {
            -1
        } else {
            0
        };
        if dir != self.stroke_dir {
            self.close(rope);
            self.stroke_dir = dir;
        }
        if dir != 0 {
            self.stroke_len_m += l_dot.abs() * dt;
            self.stroke_peak_s = self.stroke_peak_s.max(s_per_fall_n);
        }
    }

    /// 진행 중인 행정을 닫는다
    pub fn close(&mut self, rope: &RopeParams) {
        if self.stroke_dir != 0 && self.stroke_len_m >= Self::MIN_STROKE_M {
            self.strokes += 1;
            self.damage += rope.damage_per_stroke(self.stroke_peak_s);
        }
        self.stroke_len_m = 0.0;
        self.stroke_peak_s = 0.0;
        self.stroke_dir = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn power_rope() -> RopeParams {
        RopeParams {
            construction: "test".into(),
            diameter_mm: 16.0,
            min_breaking_force_kn: 178.0,
            tensile_grade_mpa: 1960.0,
            sheave_diameter_mm: 320.0,
            drum_diameter_mm: 400.0,
            sheave_count: 2,
            mass_kg_per_m: 1.0,
            axial_stiffness_ea_n: 1.28e7,
            min_safety_factor: 4.5,
            fatigue: RopeFatigue::PowerLaw { n_ref: 1.0e6, s_ref_kn: 20.0, k: 3.0, dd_ref: 20.0, m: 2.0, source: "test".into() },
        }
    }

    #[test]
    fn higher_tension_shorter_life() {
        let r = power_rope();
        assert!(r.bending_cycles_to_discard(30_000.0) < r.bending_cycles_to_discard(15_000.0));
        assert!((r.bending_cycles_to_discard(20_000.0) - 1.0e6).abs() < 1e-3);
    }

    #[test]
    fn larger_sheave_longer_life() {
        let mut r = power_rope();
        let a = r.bending_cycles_to_discard(20_000.0);
        r.sheave_diameter_mm = 400.0;
        assert!(r.bending_cycles_to_discard(20_000.0) > a);
    }

    #[test]
    fn counter_counts_strokes_and_damage() {
        let r = power_rope();
        let mut c = RopeCounter::default();
        for _ in 0..100 {
            c.step(&r, -0.1, 20_000.0, 0.1); // 1 m 권상
        }
        for _ in 0..10 {
            c.step(&r, 0.0, 20_000.0, 0.1);
        }
        for _ in 0..2 {
            c.step(&r, 0.1, 20_000.0, 0.1); // 0.02 m — 너무 짧음
        }
        c.close(&r);
        assert_eq!(c.strokes, 1);
        assert!((c.damage - 2.0 / 1.0e6).abs() < 1e-15);
    }

    #[test]
    fn feyrer_update_matches_hand_calculation() {
        // 6×19 FC 파단 상수 (a0 −0.809, a1 0.875, a2 6.480, a3 −1.850), D/d 25, S/d² 100 N/mm², R0 1770
        // lg N = −0.809 + (0.875 − 1.850·lg25)·2 + 6.480·lg25 ≈ 4.83 → N ≈ 6.7×10⁴
        let mut r = power_rope();
        r.tensile_grade_mpa = 1770.0;
        r.sheave_diameter_mm = 25.0 * r.diameter_mm;
        r.fatigue = RopeFatigue::FeyrerUpdate { a0: -0.809, a1: 0.875, a2: 6.480, a3: -1.850, f_d: 1.0, f_l: 1.0, source: "test".into() };
        let n = r.bending_cycles_to_discard(100.0 * 16.0 * 16.0);
        assert!(n > 6.0e4 && n < 7.5e4, "{n}");
        // 장력이 커지면 줄고, D/d 가 커지면 는다
        assert!(r.bending_cycles_to_discard(150.0 * 256.0) < n);
        r.sheave_diameter_mm = 30.0 * r.diameter_mm;
        assert!(r.bending_cycles_to_discard(100.0 * 256.0) > n);
    }

    #[test]
    fn safety_factor() {
        let r = power_rope();
        assert!((r.safety_factor(25_000.0) - 7.12).abs() < 1e-9);
    }
}
