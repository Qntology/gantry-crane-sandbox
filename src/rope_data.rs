//! 로프 피로 · 설계 기준 데이터.
//!
//! - Feyrer 굽힘 피로 상수: IFT Stuttgart, Feyrer "Wire Ropes" 갱신 자료(2011) 표 3.14
//!   <https://www.ift.uni-stuttgart.de/forschung/img/feyrer/Update-Rope.pdf>.
//!   폐기 상수 a2 · a3 (FC 8.567 / −2.894, IWRC 8.056 / −2.577)는 Ryan 승강기 논문의 표 3.14b 인용과 일치한다.
//!   식의 항 배치는 OCR 재구성이라 Feyrer 2015 원서로 확인해야 한다.
//! - 최소 설계 안전율 Zp: ISO 16625:2013 미리보기 (단층 권상, M1–M8).

use crate::rope::RopeFatigue;

/// 8×19 Seale 섬유심(FC), 보통 꼬임 sZ, 폐기 굽힘 횟수 N_A
pub fn feyrer_8x19_seale_fc_discard_sz() -> RopeFatigue {
    RopeFatigue::FeyrerUpdate {
        a0: -2.660,
        a1: 1.887,
        a2: 8.567,
        a3: -2.894,
        f_d: 1.0,
        f_l: 1.0,
        source: "Feyrer 갱신 자료 표 3.14: 8×19 Seale FC 폐기 N_A sZ (식 항 배치는 재구성 — 원서 확인 필요)".into(),
    }
}

/// 6×19 섬유심(FC), 파단 굽힘 횟수 N (폐기가 아니라 파단)
pub fn feyrer_6x19_fc_break() -> RopeFatigue {
    RopeFatigue::FeyrerUpdate {
        a0: -0.809,
        a1: 0.875,
        a2: 6.480,
        a3: -1.850,
        f_d: 1.0,
        f_l: 1.0,
        source: "Feyrer 갱신 자료 표 3.14: 6×19 FC 파단 N".into(),
    }
}

/// 기본 로프 피로 모델 — 실제 로프 구조의 상수로 바꿔 쓴다
pub fn default_fatigue() -> RopeFatigue {
    feyrer_8x19_seale_fc_discard_sz()
}

/// ISO 16625:2013 단층 권상 최소 설계 안전율 Zp (M1–M8)
pub fn iso16625_zp(mechanism_class: &str) -> Option<f64> {
    let z = [3.15, 3.35, 3.55, 4.0, 4.5, 5.6, 7.1, 9.0];
    mechanism_index(mechanism_class).map(|i| z[i])
}

/// "M1".."M8" → 0..7
pub fn mechanism_index(class: &str) -> Option<usize> {
    let c = class.trim().to_ascii_uppercase();
    let n: usize = c.strip_prefix('M')?.parse().ok()?;
    if (1..=8).contains(&n) {
        Some(n - 1)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zp_table() {
        assert_eq!(iso16625_zp("M5"), Some(4.5));
        assert_eq!(iso16625_zp("m8"), Some(9.0));
        assert_eq!(iso16625_zp("M9"), None);
    }
}
