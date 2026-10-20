//! 화물 시나리오 — 크레인이 옮기는 화물을 층으로 나눠 프리셋으로 만든다.
//!
//! 1. 화물 종류 (`CargoType`): 무게 분포 · 바람 받는 면적 · 인양구 · 추가 취급 시간 · 위험물 · 특수 절차
//! 2. 혼합 (`CargoScenario.mix`): 종류별 이동 비율
//! 3. 물동 패턴: 월 · 요일 배수, 선박 집중일(묶음 도착)
//! 4. 절차 규칙: 화물별 허용 풍속(풍압 면적 / 질량), 위험물 직반출 · 낙뢰 중지, 특수 화물 시험 인양, 신고 무게 오차
//!
//! 근거 (docs/research/stage2/R3_cargo.md):
//! - 컨테이너 무게: Interpave 표 16 (20ft · 40ft 열), 자중: Hapag-Lloyd 사양 (20' 2.3 t, 40' 3.7–3.9 t, 40' 냉동 4.3–4.6 t,
//!   40' 평판 5.8–6.0 t), 빈 컨테이너 20 % (Drewry/UNCTAD, 세계 항만), 냉동 8.2 % (앤트워프)
//! - 신고 무게 오차: VICT 2017 수출 530개 중 ±0.5 t 20.6 % · ±1 t 8.5 % · ±2 t 3.6 %, 필리핀 항만청 ±1.5 t 6.3–9.0 %
//! - 허용 풍속: EN 13000 · Liebherr v = v_표 · √(1.2 m²/t · m_H / A_W), A_W = A_P · c_D (상한 v_표) — 이동식 크레인
//!   기준이라 야드 · 안벽 크레인에는 풍압 큰 화물의 비례 지표로만 쓴다
//! - 시험 인양: KOSHA A-G-18-2026 컨테이너 20 cm 정지, 특수 화물 20–30 cm + 줄걸이 작업자 앞뒤
//! - 위험물: KOSHA A-G-18-2026 Class 1 · 2 · 6.2 · 7 직선적 · 직반출, 기상 악화 하역 중지는 관리청 재량 (선박입출항법 제34조)
//! - 코일: Standard Club 2–30 t, 저장 크레인 집기 · 놓기 35 s (Weckenborg et al. 2025)
//! - 프리캐스트: 중공 슬래브 276–482 kg/m², 더블티 939–1,234 lb/ft (PCI), 패널 바람 10 m/s (Qld 실무 규정)
//!
//! 출처가 없는 값(특수 화물 비율 · 위험물 비율 · 추가 시간 일부 · 계절 배수)은 `가정` 으로 적었다.

use crate::duty::LoadBin;
use serde::{Deserialize, Serialize};

/// 화물 한 종류
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoType {
    pub id: String,
    pub name: String,
    /// container · reefer · empty · oog · dg · coil · precast · block · general
    pub category: String,
    /// 무게 분포 (t, 비율) — 화물만 (스프레더 · 인양구 제외)
    pub weight_bins: Vec<(f64, f64)>,
    /// 바람 받는 투영 면적 (m²) · 항력 계수 c_D (A_W = A_P · c_D)
    pub area_m2: f64,
    pub drag_cd: f64,
    /// 화물마다 따로 다는 인양구 (kg) — 크레인 고정 부속(현장 attachment)에 더해진다
    #[serde(default)]
    pub rigging_kg: f64,
    /// 현장 기준 취급 시간에 더하는 시간 (s/이동): 줄걸이 · 시험 인양 정지 · 플러그 확인 등
    #[serde(default)]
    pub extra_handling_s: f64,
    /// 시험 인양 높이 (m, 0 = 없음) — KOSHA 20 cm · 특수 20–30 cm
    #[serde(default)]
    pub test_lift_m: f64,
    /// 위험물 IMDG 등급 (없으면 빈칸) · 직선적 · 직반출 대상 (Class 1 · 2 · 6.2 · 7)
    #[serde(default)]
    pub imdg_class: String,
    #[serde(default)]
    pub direct_delivery: bool,
    /// 신고 무게 오차: 표준편차 (t) — 0 이면 계근한 화물
    #[serde(default)]
    pub declared_error_sd_t: f64,
    /// 줄걸이 작업자 앞뒤 · 유도 로프가 필요한 화물
    #[serde(default)]
    pub needs_riggers: bool,
    #[serde(default)]
    pub sources: Vec<String>,
}

impl CargoType {
    pub fn mean_t(&self) -> f64 {
        let s: f64 = self.weight_bins.iter().map(|b| b.1.max(0.0)).sum::<f64>().max(1e-12);
        self.weight_bins.iter().map(|b| b.0 * b.1.max(0.0)).sum::<f64>() / s
    }

    /// 바람 받는 면적 / 질량 (m²/t) — 인양구 · 훅 질량 `hook_t` 을 더한 매단 질량 기준
    pub fn area_per_t(&self, hook_t: f64) -> f64 {
        self.area_m2 * self.drag_cd / (self.mean_t() + self.rigging_kg / 1000.0 + hook_t).max(0.1)
    }

    /// 허용 풍속 (크레인 운전 한계 `crane_limit` 을 넘지 않음): EN 13000 비례식 v = v_한계 · √(1.2 / (A_W/m))
    pub fn wind_limit(&self, crane_limit: f64, hook_t: f64) -> f64 {
        let r = self.area_per_t(hook_t).max(1e-6);
        (crane_limit * (1.2 / r).sqrt()).min(crane_limit)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.weight_bins.is_empty() || self.weight_bins.iter().all(|b| b.1 <= 0.0) {
            return Err(format!("화물 {}: 무게 분포가 비었습니다", self.id));
        }
        if self.weight_bins.iter().any(|b| !(b.0.is_finite() && b.0 >= 0.0)) {
            return Err(format!("화물 {}: 무게는 0 이상이어야 합니다", self.id));
        }
        if !(self.area_m2 >= 0.0 && self.drag_cd >= 0.0 && self.extra_handling_s >= 0.0) {
            return Err(format!("화물 {}: 면적 · 항력 · 추가 시간은 0 이상", self.id));
        }
        Ok(())
    }
}

/// Interpave 표 16 무게 (t) → 4 t 폭으로 묶은 분포 (4 t 아래 칸은 빈 컨테이너로 보고 뺀다 — 가정)
const CONT20_LOADED: [(f64, f64); 6] = [(5.5, 14.82), (9.5, 14.83), (13.5, 19.21), (17.5, 26.38), (21.5, 22.46), (24.5, 0.39)];
const CONT40_LOADED: [(f64, f64); 8] = [(5.5, 3.06), (9.5, 13.15), (13.5, 21.48), (17.5, 22.30), (21.5, 33.55), (25.5, 5.09), (29.5, 1.27), (33.0, 0.08)];

fn bins(v: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let s: f64 = v.iter().map(|x| x.1).sum::<f64>().max(1e-12);
    v.iter().map(|(w, p)| (*w, p / s)).collect()
}

#[allow(clippy::too_many_arguments)]
fn ct(id: &str, name: &str, cat: &str, w: &[(f64, f64)], area: f64, cd: f64, rig: f64, extra: f64, test: f64, err: f64, src: &[&str]) -> CargoType {
    CargoType {
        id: id.into(),
        name: name.into(),
        category: cat.into(),
        weight_bins: bins(w),
        area_m2: area,
        drag_cd: cd,
        rigging_kg: rig,
        extra_handling_s: extra,
        test_lift_m: test,
        imdg_class: String::new(),
        direct_delivery: false,
        declared_error_sd_t: err,
        needs_riggers: false,
        sources: src.iter().map(|s| s.to_string()).collect(),
    }
}

/// 시험 인양(20 cm 정지)에서 신고와 1 t 넘게 다르면 다시 확인하는 시간 (s) — 가정
pub const RECHECK_S: f64 = 60.0;

/// 신고 무게 오차 표준편차 (t): VICT 수출 ±1 t 넘는 비율 8.5 % → 정규 근사 σ ≈ 1 / 1.72 = 0.58 t
/// (±0.5 t 20.6 % 는 σ ≈ 0.4 t, ±2 t 3.6 % 는 σ ≈ 0.95 t — 꼬리가 두껍다. 가운데 값 0.6 t 를 쓴다)
pub const CONTAINER_DECLARED_SD_T: f64 = 0.6;

/// 내장 화물 종류
pub fn types() -> Vec<CargoType> {
    let interpave = "Interpave Heavy Duty Pavements Ed.4 표 16 (컨테이너 무게 분포)";
    let hapag = "Hapag-Lloyd Container Specification 2016 (자중 · 최대 총중량)";
    let vict = "VICT 2017 VGM 계근: ±1 t 넘는 수출 8.5 %";
    let kosha = "KOSHA GUIDE A-G-18-2026 항만하역작업 (20 cm 스토핑, 특수 화물 20–30 cm · 줄걸이 앞뒤, 냉동 플러그 확인)";
    let en13000 = "EN 13000 · Liebherr 바람 안내서: 1.2 m²/t 기준, v = v_표·√(1.2·m_H/A_W)";
    let mut v = vec![
        // 컨테이너: 40' 옆면 약 12.2 × 2.9 m = 35 m², 20' 6.1 × 2.6 = 16 m², c_D 1.2 (Liebherr 판 · 상자 1.1–2.0 범위, 컨테이너 전용 값 없음 — 가정)
        ct("cont20", "20ft 적재 컨테이너", "container", &CONT20_LOADED, 16.0, 1.2, 0.0, 0.0, 0.2, CONTAINER_DECLARED_SD_T, &[interpave, hapag, vict, kosha, en13000]),
        ct("cont40", "40ft 적재 컨테이너", "container", &CONT40_LOADED, 35.0, 1.2, 0.0, 0.0, 0.2, CONTAINER_DECLARED_SD_T, &[interpave, hapag, vict, kosha, en13000]),
        ct("empty20", "20ft 빈 컨테이너", "empty", &[(2.3, 1.0)], 16.0, 1.2, 0.0, 0.0, 0.2, 0.0, &[hapag, "Drewry/UNCTAD: 세계 항만 물동량의 20 % 가 빈 컨테이너", kosha]),
        ct("empty40", "40ft 빈 컨테이너", "empty", &[(3.9, 1.0)], 35.0, 1.2, 0.0, 0.0, 0.2, 0.0, &[hapag, "Drewry/UNCTAD: 세계 항만 물동량의 20 % 가 빈 컨테이너", kosha]),
        // 냉동: 무게는 40' 적재 분포 (냉동 전용 분포 없음 — 가정), 플러그 확인 +30 s (가정)
        ct("reefer40", "40ft 냉동 컨테이너", "reefer", &CONT40_LOADED, 35.0, 1.2, 0.0, 30.0, 0.2, CONTAINER_DECLARED_SD_T, &[hapag, "앤트워프 2021: 냉동 8.2 % (TEU)", kosha]),
        // 특수 · OOG 평판: 자중 6 t + 화물 (가정 4–30 t), 바람 면적 큼 (가정 60 m²), 줄걸이 · 시험 인양 +240 s (가정)
        ct("oog40", "40ft 평판 · 특수 화물 (OOG)", "oog", &[(10.0, 0.3), (20.0, 0.45), (32.0, 0.25)], 60.0, 1.3, 0.0, 240.0, 0.3, 0.3, &[hapag, kosha, "OOG 비율 · 화물 무게 · 줄걸이 시간은 자료 없음 (가정)"]),
        // 코일: 열연 2–30+ t (Standard Club), 한 코일 20–24 t (TIS) — 기존 광양 프리셋 10 · 16 · 22 · 28 t 분포
        ct("coil_hr", "열연 코일", "coil", &[(10.0, 0.2), (16.0, 0.3), (22.0, 0.3), (28.0, 0.2)], 3.0, 0.8, 0.0, 0.0, 0.0, 0.0, &["Standard Club Carriage of Steel Cargo 2020: 열연 2–30+ t, 냉연 2–28 t", "TIS: 한 코일 20–24 t", "Weckenborg et al. 2025: 집기 · 놓기 35 s"]),
        // 프리캐스트: 중공 슬래브 1.2 m 폭 (Mannok 276–482 kg/m²) → 4.0–9.3 t, 판 면적 1.2 × 12 m (눕혀 듦, c_D 1.2)
        ct("hollowcore", "중공 슬래브", "precast", &[(4.0, 0.4), (5.5, 0.35), (9.3, 0.25)], 14.4, 1.2, 0.0, 60.0, 0.2, 0.0, &["Mannok Hollowcore 200 · 300 · 400: 276 · 380 · 482 kg/m²", "Qld 프리캐스트 실무 규정 2026: 판 1,200 mm 폭"]),
        // 더블티 12DT30 × 60 ft ≈ 25.6 t, 15DT34 × 60 ft ≈ 33.6 t (PCI)
        ct("double_tee", "더블티", "precast", &[(25.6, 0.6), (33.6, 0.4)], 70.0, 1.2, 0.0, 120.0, 0.3, 0.0, &["PCI Design Handbook 8판: 12DT30 939 lb/ft · 15DT34 1,234 lb/ft"]),
        // 벽 패널 3 × 10 m · 10 t (Liebherr 예시 계산), 세워 들면 바람 면적 30 m²
        ct("wall_panel", "벽 패널 (세워 듦)", "precast", &[(10.0, 1.0)], 30.0, 1.2, 0.0, 120.0, 0.3, 0.0, &["Qld 규정: 큰 패널은 이동식 크레인 바람 10 m/s 보다 낮춰야", en13000]),
        // 선박 블록: 약 100 t (15 × 15 m), 대형 · 선행 탑재 300–800 t (Koh et al. 2011, 기존 울산 프리셋)
        ct("block", "선박 블록", "block", &[(100.0, 0.6), (300.0, 0.25), (500.0, 0.1), (800.0, 0.05)], 225.0, 1.2, 0.0, 1800.0, 0.3, 0.0, &["Koh et al. 2011 (블록 100 t, 대형 300–800 t)", "대한조선학회지 2018: 블록 반전 · 훅 용량은 계획 단계에서 정함"]),
    ];
    // 위험물 컨테이너 (20' 적재 분포, Class 3 기준) — 비율 · 무게 분포는 자료 없음 (가정)
    let mut dg = ct("dg20", "위험물 컨테이너 (IMDG)", "dg", &CONT20_LOADED, 16.0, 1.2, 0.0, 60.0, 0.2, CONTAINER_DECLARED_SD_T, &[kosha, "선박입출항법 제34조: 기상 악화 시 관리청이 위험물 하역 금지 · 중지"]);
    dg.imdg_class = "3".into();
    v.push(dg);
    let mut dg1 = ct("dg_direct", "직반출 위험물 (Class 1 · 2 · 6.2 · 7)", "dg", &CONT20_LOADED, 16.0, 1.2, 0.0, 120.0, 0.2, CONTAINER_DECLARED_SD_T, &[kosha]);
    dg1.imdg_class = "1".into();
    dg1.direct_delivery = true;
    v.push(dg1);
    for c in v.iter_mut() {
        if c.category == "oog" || c.category == "precast" || c.category == "block" {
            c.needs_riggers = true;
        }
    }
    v
}

pub fn cargo_type(id: &str) -> Option<CargoType> {
    types().into_iter().find(|c| c.id == id)
}

/// 화물 시나리오 (현장 프리셋 `cargo`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoScenario {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// (화물 종류 id, 이동 비율)
    pub mix: Vec<(String, f64)>,
    /// 사용자 정의 화물 종류 (내장 id 와 겹치면 이것을 쓴다)
    #[serde(default)]
    pub custom_types: Vec<CargoType>,
    /// 월(12) · 요일(7, 월요일부터) 수요 배수 — 비면 1
    #[serde(default)]
    pub monthly_factor: Vec<f64>,
    #[serde(default)]
    pub weekday_factor: Vec<f64>,
    /// 선박 집중일: 하루 확률 · 그날 수요 배수
    #[serde(default)]
    pub peak_day_prob: f64,
    #[serde(default = "one")]
    pub peak_factor: f64,
    #[serde(default)]
    pub sources: Vec<String>,
}

fn one() -> f64 {
    1.0
}

impl CargoScenario {
    pub fn resolved_types(&self) -> Result<Vec<(CargoType, f64)>, String> {
        let total: f64 = self.mix.iter().map(|m| m.1.max(0.0)).sum();
        if total <= 0.0 {
            return Err(format!("화물 시나리오 {}: 혼합 비율이 비었습니다", self.id));
        }
        let mut out = Vec::new();
        for (id, share) in self.mix.iter() {
            let t = self.custom_types.iter().find(|c| &c.id == id).cloned().or_else(|| cargo_type(id)).ok_or_else(|| format!("화물 종류 '{id}' 이 없습니다"))?;
            t.validate()?;
            out.push((t, share.max(0.0) / total));
        }
        Ok(out)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.resolved_types()?;
        for (k, v, n) in [("monthly_factor", &self.monthly_factor, 12usize), ("weekday_factor", &self.weekday_factor, 7)] {
            if !v.is_empty() && (v.len() != n || v.iter().any(|x| !(x.is_finite() && *x >= 0.0))) {
                return Err(format!("{k} 는 {n}개의 0 이상 값이어야 합니다"));
            }
        }
        if !(0.0..=1.0).contains(&self.peak_day_prob) || self.peak_factor < 0.0 {
            return Err("peak_day_prob 0–1, peak_factor ≥ 0".into());
        }
        Ok(())
    }

    /// 그날 수요 배수 (월 0–11, 요일 0 = 월요일)
    pub fn day_factor(&self, month: usize, weekday: usize, peak: bool) -> f64 {
        let m = self.monthly_factor.get(month).cloned().unwrap_or(1.0);
        let w = self.weekday_factor.get(weekday).cloned().unwrap_or(1.0);
        m * w * if peak { self.peak_factor } else { 1.0 }
    }

    /// 정격 하중 대비 하중 스펙트럼 (사이클 증분표용): 0.05 단위로 묶어 많아야 8칸
    pub fn spectrum(&self, rated_kg: f64) -> Result<Vec<LoadBin>, String> {
        let mut acc: Vec<(f64, f64)> = Vec::new();
        for (t, share) in self.resolved_types()? {
            for (w, p) in t.weight_bins.iter() {
                let f = ((w * 1000.0 + t.rigging_kg) / rated_kg.max(1.0) / 0.05).round() * 0.05;
                let f = f.clamp(0.05, 1.5);
                match acc.iter_mut().find(|x| (x.0 - f).abs() < 1e-9) {
                    Some(x) => x.1 += share * p,
                    None => acc.push((f, share * p)),
                }
            }
        }
        acc.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        // 칸이 많으면 이웃끼리 합쳐 8칸 이하로 (비율 가중 평균 무게)
        while acc.len() > 8 {
            let (mut best, mut gap) = (0, f64::MAX);
            for k in 0..acc.len() - 1 {
                let g = acc[k + 1].0 - acc[k].0;
                if g < gap {
                    gap = g;
                    best = k;
                }
            }
            let (a, b) = (acc[best], acc[best + 1]);
            let s = a.1 + b.1;
            acc[best] = (((a.0 * a.1 + b.0 * b.1) / s.max(1e-12) / 0.05).round() * 0.05, s);
            acc.remove(best + 1);
        }
        Ok(acc.into_iter().map(|(fraction, share)| LoadBin { fraction, share }).collect())
    }

    /// 정격을 넘는 들기 비율 (신고 무게 오차 포함, 정규 근사) — 정격 하중 제한 장치가 막아야 하는 들기
    pub fn overweight_share(&self, rated_kg: f64) -> Result<f64, String> {
        let mut p = 0.0;
        for (t, share) in self.resolved_types()? {
            for (w, q) in t.weight_bins.iter() {
                let over_t = rated_kg / 1000.0 - t.rigging_kg / 1000.0 - w;
                let sd = t.declared_error_sd_t;
                let pe = if sd <= 0.0 {
                    if over_t < 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    1.0 - crate::rng::norm_cdf(over_t / sd)
                };
                p += share * q * pe;
            }
        }
        Ok(p)
    }
}

pub fn scenarios() -> Vec<CargoScenario> {
    let s = |id: &str, name: &str, desc: &str, mix: &[(&str, f64)], peak_p: f64, peak_f: f64, src: &[&str]| CargoScenario {
        id: id.into(),
        name: name.into(),
        description: desc.into(),
        mix: mix.iter().map(|(a, b)| (a.to_string(), *b)).collect(),
        custom_types: Vec::new(),
        monthly_factor: Vec::new(),
        weekday_factor: Vec::new(),
        peak_day_prob: peak_p,
        peak_factor: peak_f,
        sources: src.iter().map(|x| x.to_string()).collect(),
    };
    vec![
        s(
            "busan_transship",
            "부산 신항 환적 일반",
            "빈 컨테이너 20 % (세계 항만), 냉동 8 % (앤트워프 비교값), 적재 40':20' = 60:40 (Interpave), 특수 1 % · 위험물 3 % 는 가정",
            &[("empty20", 0.08), ("empty40", 0.12), ("cont20", 0.272), ("cont40", 0.408), ("reefer40", 0.08), ("oog40", 0.01), ("dg20", 0.03)],
            0.15,
            1.4,
            &["부산항만공사 2025: 2,488만 TEU, 환적 57 %", "선박 집중일 확률 · 배수는 가정"],
        ),
        s(
            "reefer_peak",
            "냉동 성수기",
            "냉동 20 % (로테르담 '컨테이너 종류의 약 15 %' 보다 높은 성수기 가정), 빈 컨테이너 15 %",
            &[("empty20", 0.06), ("empty40", 0.09), ("cont20", 0.22), ("cont40", 0.40), ("reefer40", 0.20), ("dg20", 0.03)],
            0.2,
            1.5,
            &["IIR 2022: 앤트워프 냉동 8.2 %, 로테르담 약 15 %", "성수기 비율은 가정"],
        ),
        s(
            "oog_project",
            "특수 화물 · 프로젝트 화물이 많은 날",
            "평판 · OOG 10 %, 직반출 위험물 2 % — 줄걸이 작업자 · 시험 인양 · 바람 제한이 늘어난다",
            &[("empty40", 0.10), ("cont20", 0.25), ("cont40", 0.43), ("oog40", 0.10), ("dg20", 0.10), ("dg_direct", 0.02)],
            0.1,
            1.2,
            &["KOSHA A-G-18-2026: 특수 화물 20–30 cm 시험 인양 · 줄걸이 작업자 앞뒤, Class 1 · 2 · 6.2 · 7 직반출", "비율은 가정"],
        ),
        s("steel_coil", "열연 코일 출하", "열연 코일 10–28 t (기존 광양 코일 창고 분포)", &[("coil_hr", 1.0)], 0.0, 1.0, &["Standard Club 2020 · TIS"]),
        s(
            "precast_mix",
            "프리캐스트 (슬래브 · 더블티 · 벽 패널)",
            "중공 슬래브 70 %, 더블티 15 %, 벽 패널 15 % — 큰 패널은 바람 제한이 낮다",
            &[("hollowcore", 0.70), ("double_tee", 0.15), ("wall_panel", 0.15)],
            0.0,
            1.0,
            &["Mannok · PCI · Qld 실무 규정", "비율은 가정"],
        ),
        s("ship_blocks", "선박 블록 탑재", "블록 100 t 60 %, 300 t 25 %, 500 t 10 %, 800 t 5 % (Koh et al. 2011)", &[("block", 1.0)], 0.0, 1.0, &["Koh et al. 2011"]),
    ]
}

pub fn scenario(id: &str) -> Option<CargoScenario> {
    scenarios().into_iter().find(|s| s.id == id)
}

/// 현장 프리셋의 화물 지정
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CargoRef {
    Preset(String),
    Data(Box<CargoScenario>),
}

impl CargoRef {
    pub fn resolve(&self) -> Result<CargoScenario, String> {
        match self {
            CargoRef::Preset(n) => scenario(n).ok_or_else(|| {
                let names: Vec<String> = scenarios().into_iter().map(|s| s.id).collect();
                format!("화물 시나리오 '{n}' 이 없습니다 (사용 가능: {})", names.join(", "))
            }),
            CargoRef::Data(s) => {
                s.validate()?;
                Ok((**s).clone())
            }
        }
    }
}

/// 운영 시뮬레이션이 쓰는 화물 묶음 (종류별 비율 · 허용 풍속 · 추가 시간)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CargoLane {
    pub id: String,
    pub name: String,
    pub category: String,
    pub share: f64,
    pub mean_t: f64,
    pub area_per_t: f64,
    /// 크레인 높이 순간풍속 기준 허용값 (m/s)
    pub wind_limit_mps: f64,
    pub extra_handling_s: f64,
    pub dg: bool,
    pub direct_delivery: bool,
    pub needs_riggers: bool,
    /// 시험 인양에서 신고 무게와 1 t 넘게 달라 다시 확인하는 비율
    pub recheck_share: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CargoCtx {
    pub scenario: String,
    pub name: String,
    pub lanes: Vec<CargoLane>,
}

pub fn context(s: &CargoScenario, crane_wind_limit: f64, hook_t: f64) -> Result<CargoCtx, String> {
    let lanes = s
        .resolved_types()?
        .into_iter()
        .map(|(t, share)| {
            let recheck = if t.declared_error_sd_t > 0.0 { 2.0 * (1.0 - crate::rng::norm_cdf(1.0 / t.declared_error_sd_t)) } else { 0.0 };
            (t, share, recheck)
        })
        .map(|(t, share, recheck)| CargoLane {
            id: t.id.clone(),
            name: t.name.clone(),
            category: t.category.clone(),
            share,
            mean_t: t.mean_t(),
            area_per_t: t.area_per_t(hook_t),
            wind_limit_mps: t.wind_limit(crane_wind_limit, hook_t),
            extra_handling_s: t.extra_handling_s + recheck * RECHECK_S,
            dg: !t.imdg_class.is_empty(),
            direct_delivery: t.direct_delivery,
            needs_riggers: t.needs_riggers,
            recheck_share: recheck,
        })
        .collect();
    Ok(CargoCtx { scenario: s.id.clone(), name: s.name.clone(), lanes })
}

impl CargoCtx {
    /// 이 순간풍속에서 허용 풍속을 넘는 화물의 비율
    pub fn share_over_wind(&self, gust: f64) -> f64 {
        self.lanes.iter().filter(|l| gust >= l.wind_limit_mps).map(|l| l.share).sum()
    }

    pub fn dg_share(&self) -> f64 {
        self.lanes.iter().filter(|l| l.dg).map(|l| l.share).sum()
    }

    /// 가장 낮은 허용 풍속 (비율 `min_share` 이상인 화물 중)
    pub fn lowest_limit(&self, min_share: f64) -> Option<(&CargoLane, f64)> {
        self.lanes.iter().filter(|l| l.share >= min_share).map(|l| (l, l.wind_limit_mps)).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// 움직이는 화물(보류 제외)의 평균 추가 취급 시간 (s/이동)
    pub fn extra_s(&self, held: &dyn Fn(&CargoLane) -> bool) -> f64 {
        let moving: f64 = self.lanes.iter().filter(|l| !held(l)).map(|l| l.share).sum();
        if moving <= 0.0 {
            return 0.0;
        }
        self.lanes.iter().filter(|l| !held(l)).map(|l| l.share * l.extra_handling_s).sum::<f64>() / moving
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_distributions_match_interpave_means() {
        // Interpave 표 16 에서 낸 평균: 40ft 17.1 t, 20ft 14.4 t — 4 t 아래 칸을 뺐으니 조금 높다
        let c40 = cargo_type("cont40").unwrap().mean_t();
        let c20 = cargo_type("cont20").unwrap().mean_t();
        assert!((c40 - 17.1).abs() < 1.0, "{c40}");
        assert!((c20 - 14.6).abs() < 1.0, "{c20}");
    }

    #[test]
    fn wind_limits_follow_sail_area() {
        let crane = 22.0;
        let hook = 10.0; // 스프레더 + 훅 (t)
        let full = cargo_type("cont40").unwrap().wind_limit(crane, hook);
        let empty = cargo_type("empty40").unwrap().wind_limit(crane, hook);
        let coil = cargo_type("coil_hr").unwrap().wind_limit(crane, 0.5);
        let panel = cargo_type("wall_panel").unwrap().wind_limit(crane, 1.5);
        assert!(empty < full && full <= crane, "{empty} {full}");
        assert!((coil - crane).abs() < 1e-9, "작은 면적 화물은 크레인 한계 그대로");
        // Liebherr 예: 65 t · A_W 280 m² → 11.2 → 5.9 m/s (비 0.53)
        let liebherr = CargoType { area_m2: 200.0, drag_cd: 1.4, weight_bins: vec![(65.0, 1.0)], ..cargo_type("block").unwrap() };
        assert!((liebherr.wind_limit(11.2, 0.0) - 5.91).abs() < 0.02, "{}", liebherr.wind_limit(11.2, 0.0));
        assert!(panel < 0.75 * crane);
    }

    #[test]
    fn scenarios_validate_and_make_spectra() {
        for s in scenarios() {
            s.validate().unwrap_or_else(|e| panic!("{}: {e}", s.id));
            let sp = s.spectrum(41_000.0).unwrap();
            assert!(!sp.is_empty() && sp.len() <= 8, "{} {}", s.id, sp.len());
            let tot: f64 = sp.iter().map(|b| b.share).sum();
            assert!((tot - 1.0).abs() < 1e-9);
        }
        // 컨테이너 41 t 크레인: 정격 초과는 거의 없다, 40 t 프리캐스트 크레인에 더블티 33.6 t + 인양구는 근처
        let b = scenario("busan_transship").unwrap();
        assert!(b.overweight_share(41_000.0).unwrap() < 1e-4);
        let sp = b.spectrum(41_000.0).unwrap();
        let mean: f64 = sp.iter().map(|x| x.fraction * x.share).sum();
        assert!(mean > 0.25 && mean < 0.45, "{mean}");
    }
}
