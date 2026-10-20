//! 해안 염해 환경 · 부식 — 크레인이 바다와 어떻게 놓였는지(해안 거리 · 바다 방위 · 해안 형태)와 시간별 날씨로
//! 비래염분 · 젖음을 만들고, ISO 9223 용량–반응식 → ISO 9224 장기식으로 강재 · 아연 부식을 표면(노출 · 차폐 · 밀폐)마다
//! 누적한다. 도장 · 아연 도금 · 모터 · 전장 · 감속기 · 로프 · 레일 클램프가 염해 방지 조치에 따라 얼마나 빨리 상하는지도 낸다.
//!
//! 층 (위에서 아래로):
//! 1. 연평균 염화물 침착량 Sd(d) = κ · 60.7 · S100 · (d/100)^−b  (mg Cl⁻/m²·d)
//!    — S100 · b: 국내 해안 27개 지역 100 m 비래염분, 6개 지점 거리 감쇠 (이종석 외, 한국콘크리트학회 2004 · 2006)
//! 2. 시간 변화: 바다 쪽(±67.5°)에서 3 m/s 넘게 부는 바람의 '바다 바람 거리'에 비례 (Morcillo, ALC17 §4.2–4.3),
//!    기준 해의 평균이 연평균과 같도록 정규화
//! 3. 날마다 ISO 9223 용량–반응식(일평균 기온 · 습도 · Sd · SO₂) → 연간 식에 맞춘 정규화 → 표면별 등가 시간
//!    → ISO 9224 D = r·τ^b (20년 뒤 직선)
//! 4. 표면: 노출(비에 씻김, 용량–반응식 기준) · 차폐(염 누적, ISO 9223 주석 · GAA 경고) · 밀폐(기계실 · 전기실)
//! 5. 부품: 도장(ISO 12944 내구성 · 환경 불일치) → 녹 면적(ISO 4628-3) → 강재 국부 두께 손실, 통로 · 난간 아연(GAA),
//!    모터 결로 · 오염 → 절연저항(IEEE 43), 전장(ISA-71.04 등급 · 습도 규칙), 감속기 수분(Cantley 1977),
//!    로프(재윤활 · 아연 도금), 레일 클램프(부식 · 작동 시험)
//!
//! 근거가 없는 값은 `가정` 이라고 적었다 (docs/plan-stage2.md · docs/research/stage2).

use crate::weather::HourWeather;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------- 근거 상수

/// 건식 거즈 NaCl mdd (mg/100 cm²·d) → mg Cl⁻/m²·d (Cl/NaCl 질량비 35.45/58.44 × 100)
pub const MDD_NACL_TO_CL: f64 = 60.7;
/// ISO 9223 용량–반응식 적용 범위 (Ohara et al. 2018 표 3)
pub const DRF_T: (f64, f64) = (-17.1, 28.7);
pub const DRF_RH: (f64, f64) = (34.0, 93.0);
pub const DRF_PD: (f64, f64) = (0.7, 150.4);
pub const DRF_SD: (f64, f64) = (0.4, 760.5);
/// ISO 9223 첫해 부식 범주 상한 (µm/년): 탄소강 · 아연 (C1 … C5, CX 는 그 위)
pub const STEEL_LIMITS: [f64; 5] = [1.3, 25.0, 50.0, 80.0, 200.0];
pub const ZINC_LIMITS: [f64; 5] = [0.1, 0.7, 2.1, 4.2, 8.4];
pub const CATEGORIES: [&str; 6] = ["C1", "C2", "C3", "C4", "C5", "CX"];
/// ISO 9224:2012 표 2 시간 지수 B1 (평균): 탄소강 0.523, 아연 0.813
pub const B_STEEL: f64 = 0.523;
pub const B_ZINC: f64 = 0.813;
/// 해안 장기 노출 평균 지수 (Alcántara et al. 2017 표 6: 해안 0.73, 범위 0.37–0.98) — 고염분(S2 이상)에서 쓴다
pub const B_STEEL_MARINE: f64 = 0.73;
/// 비에 안 씻기는 차폐면의 장기 지수 — 보호 녹이 안 생겨 거의 직선 (NIMS 초시 · 미야코지마 10년: 차폐 b ≈ 1.07–1.12,
/// 노출 0.61–0.88 — Yan et al. 2020 그림 1 에서 읽은 값)
pub const B_STEEL_SHELTERED: f64 = 1.0;
/// 제습한 밀폐면 / 바깥 부식 속도 (영종대교 상자 거더 안 < 0.0004 vs 바깥 ~0.03 mm/년 ≈ 1/75, RH 40–50 % 유지 — Li 2011)
pub const DEHUMIDIFIED_RATIO: f64 = 1.0 / 75.0;
/// 건식 거즈 → 습식 캔들 환산 (JIS Z 2382: Cl_습식캔들 = 2 × Cl_건식거즈, Li 2011 식 1 인용)
pub const DRY_GAUZE_TO_WET_CANDLE: f64 = 2.0;
/// 바다 바람 판정: 바다 방위 ±67.5° · 3 m/s 초과 (ALC17 §4.2 "3–5 m/s", Morcillo et al.)
pub const ONSHORE_HALF_ANGLE: f64 = 67.5;
pub const ONSHORE_MIN_WIND: f64 = 3.0;
/// ISO 4628-3 녹 등급 면적 (%) Ri0–Ri5 — 표준 값은 기억값 (리서치 R2 U1, 미확인)
pub const RI_AREA_PCT: [f64; 6] = [0.0, 0.05, 0.5, 1.0, 8.0, 40.0];
/// ISO 12944-1:2017 내구성 (년): L ≤7, M 7–15, H 15–25, VH > 25 (VH 위 끝 35 는 가정)
pub const DURABILITY_YEARS: [(&str, f64, f64); 4] = [("L", 2.0, 7.0), ("M", 7.0, 15.0), ("H", 15.0, 25.0), ("VH", 25.0, 35.0)];
/// IEEE 43 최소 절연저항 (40 °C 환산): 저압 · 랜덤 권선 5 MΩ, PI ≥ 2.0
pub const IR_MIN_MOHM: f64 = 5.0;
/// ISA-71.04 SO₂ 등급 경계 (ppb): G1 < 10, G2 < 50, G3 < 300
pub const ISA_SO2_PPB: [f64; 3] = [10.0, 50.0, 300.0];
/// SO₂ 1 ppb ≈ 2.62 µg/m³ (25 °C, 1 atm)
pub const SO2_UG_PER_PPB: f64 = 2.62;
/// ISO 9223 SO₂ 침착량 ≈ 0.67 × 농도 (Panchenko & Marshakov 2017 이 인용한 ISO 9223 환산)
pub const SO2_DEPOSITION_PER_UG: f64 = 0.67;

// ---------------------------------------------------------------- 가정 상수 (docs 에 표시)

/// 차폐면(비에 안 씻기는 아랫면 · 상자 안)의 염 누적 배수 — 보정 대상 (보정 전 1.6, fieldcal 이 NIMS 차폐/노출 첫해 비 1.3–1.5 에 맞춘다)
pub const SHELTER_SALT_FACTOR: f64 = 1.6;
/// 차폐면 염이 저절로 빠지는 시간 (이슬 · 흘러내림, 일) — 가정
pub const SHELTER_RESIDENCE_D: f64 = 45.0;
/// 고압 담수 세척이 표면 염을 지우는 비율 (PennDOT 2013: 10–16 MPa 세척 40–93 %, 정상 압력 5회 평균 약 0.75; 저압 17–29 %)
pub const WASH_EFFICIENCY: f64 = 0.75;
/// 높이에 따른 비래염분: 지면 근처(쇄파) 성분은 지수 감소 (Meira et al. 2017: 처음 20–30 m 에서 크게 감소),
/// 바다 전체 배경 성분은 높이와 무관 — 감쇠 길이 · 배경 비율은 가정 (크레인에 두 높이 습식 캔들로 확인 권장)
pub const SALT_HEIGHT_DECAY_M: f64 = 10.0;
/// 비래염분 측정 높이 (K3 수집기 2 m — 이종석 외 2004)
pub const SALT_MEASURE_HEIGHT_M: f64 = 2.0;
/// 도장 수명의 환경 불일치 지수: T = T_사양 · (r_사양 / r_현장)^m — 가정 (KSCRANE 해안 C3 도장 3–5년 일화에 맞춘 값)
pub const COATING_MISMATCH_EXP: f64 = 0.75;
/// 설계보다 순한 환경에서 늘어나는 도장 수명의 상한 배수 — 가정 (자외선 · 결합제 노화는 부식성과 상관없이 진행)
pub const COATING_MISMATCH_MAX: f64 = 1.5;
/// 녹 면적이 Ri1 (0.05 %) 이 되면 그 자리 강재 부식이 시작된다
pub const SPOT_START_COAT: f64 = 0.64;

// ---------------------------------------------------------------- 용량–반응식 · 범주

/// ISO 9223:2012 탄소강 첫해 부식 속도 (µm/년). pd · sd: mg/m²·d, rh: %, t: °C
pub fn drf_steel(pd: f64, sd: f64, rh: f64, t: f64) -> f64 {
    let f = if t <= 10.0 { 0.150 * (t - 10.0) } else { -0.054 * (t - 10.0) };
    1.77 * pd.max(0.0).powf(0.52) * (0.020 * rh + f).exp() + 0.102 * sd.max(0.0).powf(0.62) * (0.033 * rh + 0.040 * t).exp()
}

/// ISO 9223:2012 아연 첫해 부식 속도 (µm/년)
pub fn drf_zinc(pd: f64, sd: f64, rh: f64, t: f64) -> f64 {
    let f = if t <= 10.0 { 0.038 * (t - 10.0) } else { -0.071 * (t - 10.0) };
    0.0129 * pd.max(0.0).powf(0.44) * (0.046 * rh + f).exp() + 0.0175 * sd.max(0.0).powf(0.57) * (0.008 * rh + 0.085 * t).exp()
}

/// 연평균 값으로 쓸 때는 적용 범위 안으로 자른다
pub fn drf_steel_annual(pd: f64, sd: f64, rh: f64, t: f64) -> f64 {
    drf_steel(pd.clamp(0.0, DRF_PD.1), sd.clamp(0.0, DRF_SD.1), rh.clamp(DRF_RH.0, DRF_RH.1), t.clamp(DRF_T.0, DRF_T.1))
}

pub fn drf_zinc_annual(pd: f64, sd: f64, rh: f64, t: f64) -> f64 {
    drf_zinc(pd.clamp(0.0, DRF_PD.1), sd.clamp(0.0, DRF_SD.1), rh.clamp(DRF_RH.0, DRF_RH.1), t.clamp(DRF_T.0, DRF_T.1))
}

fn category_of(r: f64, limits: &[f64; 5]) -> &'static str {
    for (k, l) in limits.iter().enumerate() {
        if r <= *l {
            return CATEGORIES[k];
        }
    }
    "CX"
}

pub fn category_steel(r: f64) -> &'static str {
    category_of(r, &STEEL_LIMITS)
}

pub fn category_zinc(r: f64) -> &'static str {
    category_of(r, &ZINC_LIMITS)
}

/// 범주 대표 탄소강 속도 (범주 경계의 기하 평균, µm/년)
pub fn category_mid_steel(c: &str) -> f64 {
    match c {
        "C1" => 0.7,
        "C2" => (1.3f64 * 25.0).sqrt(),
        "C3" => (25.0f64 * 50.0).sqrt(),
        "C4" => (50.0f64 * 80.0).sqrt(),
        "C5" => (80.0f64 * 200.0).sqrt(),
        _ => (200.0f64 * 700.0).sqrt(),
    }
}

/// 염화물 침착 등급 S0–S3 (mg/m²·d: ≤3, 3–60, 60–300, 300–1500)
pub fn chloride_class(sd: f64) -> &'static str {
    if sd <= 3.0 {
        "S0"
    } else if sd <= 60.0 {
        "S1"
    } else if sd <= 300.0 {
        "S2"
    } else {
        "S3"
    }
}

/// ISO 9224 장기 부식 깊이 (µm): D = r·t^b, 20년 뒤는 20년 기울기로 직선 (식 3)
pub fn iso9224_loss(r: f64, t_years: f64, b: f64) -> f64 {
    let t = t_years.max(0.0);
    if t <= 20.0 {
        r * t.powf(b)
    } else {
        r * (20f64.powf(b) + b * 20f64.powf(b - 1.0) * (t - 20.0))
    }
}

/// 깊이 D 에 해당하는 등가 시간 (역함수)
pub fn iso9224_time(r: f64, d: f64, b: f64) -> f64 {
    if r <= 0.0 || d <= 0.0 {
        return 0.0;
    }
    let d20 = r * 20f64.powf(b);
    if d <= d20 {
        (d / r).powf(1.0 / b)
    } else {
        20.0 + (d - d20) / (r * b * 20f64.powf(b - 1.0))
    }
}

/// 녹 등급 (Ri0–Ri5) — 녹 면적 %
pub fn ri_grade(area_pct: f64) -> u8 {
    let mut g = 0u8;
    for (k, a) in RI_AREA_PCT.iter().enumerate().skip(1) {
        if area_pct >= *a {
            g = k as u8;
        }
    }
    g
}

/// 도장 열화 c (0 신품 → 1 첫 대보수 = Ri3 1 %) → 녹 면적 % : 1 %·exp(k(c − 1)), Ri4 8 % 가 c = 1.25
pub fn rust_area_pct(c: f64) -> f64 {
    if c <= 0.0 {
        return 0.0;
    }
    let k = (8.0f64).ln() / 0.25;
    (1.0 * (k * (c - 1.0)).exp()).min(60.0)
}

/// 바다 바람 가중 (0 · 1 사이): 바다 방위 ±67.5° 안이면 1, 경계 22.5° 는 코사인으로 줄임
pub fn onshore_factor(wind_from_deg: f64, sea_bearing_deg: f64) -> f64 {
    let d = ((wind_from_deg - sea_bearing_deg + 540.0).rem_euclid(360.0) - 180.0).abs();
    let edge = 22.5;
    if d <= ONSHORE_HALF_ANGLE - edge {
        1.0
    } else if d >= ONSHORE_HALF_ANGLE + edge {
        0.0
    } else {
        0.5 * (1.0 + (std::f64::consts::PI * (d - (ONSHORE_HALF_ANGLE - edge)) / (2.0 * edge)).cos())
    }
}

// ---------------------------------------------------------------- 국내 해안 비래염분 자료

/// (지역, 해안, 100 m 지점 비래염분 mdd NaCl) — 이종석 · 최원성 · 안기홍 · 이장화, 한국콘크리트학회 2006 가을 표 1 (3년 월별)
pub const KR_SALT_100M: [(&str, &str, f64); 27] = [
    ("goseong", "east", 4.3),
    ("sokcho", "east", 2.4),
    ("jumunjin", "east", 7.7),
    ("samcheok", "east", 6.7),
    ("uljin", "east", 5.1),
    ("yeongdeok", "east", 3.0),
    ("pohang", "east", 6.2),
    ("ulsan", "east", 7.6),
    ("busan", "east", 2.2),
    ("ganghwa1", "west", 1.0),
    ("ganghwa2", "west", 0.8),
    ("incheon", "west", 1.1),
    ("asan_bay", "west", 1.0),
    ("taean", "west", 16.0),
    ("cheonsu_bay", "west", 0.9),
    ("boryeong", "west", 4.7),
    ("byeonsan", "west", 2.2),
    ("gochang", "west", 11.4),
    ("hampyeong", "west", 4.2),
    ("mokpo", "west", 1.0),
    ("haenam", "south", 2.0),
    ("jangheung", "south", 1.2),
    ("goheung", "south", 0.7),
    ("gwangyang", "south", 1.2),
    ("sacheon", "south", 1.6),
    ("geoje", "south", 3.5),
    ("masan", "south", 1.1),
];

/// 해안별 거리 감쇠 지수 b (y = a·x^−b): 동 (속초 0.30 · 울진 0.84), 서 (고창 0.51 · 변산 0.37), 남 (장흥 0.10 · 사천 0.40)
/// — 이종석 · 최원성 · 문한영, 한국콘크리트학회 2004 봄 (6개 지점, 1년 월별)
pub const KR_DECAY: [(&str, f64, f64); 3] = [("east", 0.30, 0.84), ("west", 0.51, 0.37), ("south", 0.10, 0.40)];

pub fn kr_decay_mean(region: &str) -> Option<f64> {
    KR_DECAY.iter().find(|(r, ..)| *r == region).map(|(_, a, b)| 0.5 * (a + b))
}

pub fn kr_salt_100m(area: &str) -> Option<(f64, &'static str)> {
    KR_SALT_100M.iter().find(|(a, ..)| *a == area).map(|(_, r, v)| (*v, *r))
}

/// 높이가 올라가도 남는 배경 염분 비율 (해안 형태별, 가정): 쇄파가 큰 외해는 지면 성분이 커 배경 비율이 작다
pub fn background_share(coast_type: &str) -> f64 {
    match coast_type {
        "open" => 0.3,
        "tidal" => 0.4,
        "strait" => 0.5,
        "bay" | "harbor" => 0.6,
        _ => 1.0,
    }
}

/// 구조물 평균 높이 `h` (m) 에서의 비래염분 배수 (측정 높이 2 m = 1)
pub fn height_factor(coast_type: &str, h: f64) -> f64 {
    let bg = background_share(coast_type);
    bg + (1.0 - bg) * (-(h - SALT_MEASURE_HEIGHT_M).max(0.0) / SALT_HEIGHT_DECAY_M).exp()
}

/// 해안 형태별 기본값 (100 m 비래염분 mdd, 감쇠 지수) — 국내 자료의 해안별 대표값으로 정한 가정
/// (외해 = 동해안 평균 5, 만 · 항내 = 남해안 1.2–1.5, 갯벌 = 서해 만 1.0, 강 항구는 하구에서 멀어 0.1)
pub fn coast_defaults(coast_type: &str) -> (f64, f64) {
    match coast_type {
        "open" => (5.0, kr_decay_mean("east").unwrap_or(0.57)),
        "bay" => (1.2, kr_decay_mean("south").unwrap_or(0.25)),
        "harbor" => (1.5, kr_decay_mean("south").unwrap_or(0.25)),
        "tidal" => (1.0, kr_decay_mean("west").unwrap_or(0.44)),
        "strait" => (2.0, 0.40),
        "river" => (0.1, 0.30),
        _ => (1.5, 0.30),
    }
}

// ---------------------------------------------------------------- 설정

/// 크레인의 해안 노출 (현장 프리셋 `environment.coastal`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoastalExposure {
    /// 크레인에서 해안선(바닷물)까지 거리 (m). 안벽 크레인 10–60 m, 야드 100–600 m
    pub distance_to_sea_m: f64,
    /// 현장에서 바다를 보는 방위 (°, 북 0 시계 방향)
    pub sea_bearing_deg: f64,
    /// harbor · bay · open · tidal · strait · river
    pub coast_type: String,
    /// 국내 비래염분 자료의 가까운 지역 (`KR_SALT_100M` 이름) — 있으면 그 100 m 값과 해안별 감쇠를 쓴다
    #[serde(default)]
    pub salt_area: Option<String>,
    /// 100 m 지점 비래염분 (mdd NaCl) · 감쇠 지수 — 직접 주면 위보다 먼저
    #[serde(default)]
    pub salt_100m_mdd: Option<f64>,
    #[serde(default)]
    pub decay_b: Option<f64>,
    /// 실측 염화물 침착량 (mg Cl⁻/m²·d, ISO 9225 습식 캔들) — 주면 거리 모델 대신 쓴다
    #[serde(default)]
    pub chloride_mg_m2_d: Option<f64>,
    /// SO₂ 농도 (µg/m³). 없으면 항만 통계(CAMS) → 없으면 3 µg/m³ (가정)
    #[serde(default)]
    pub so2_ug_m3: Option<f64>,
    /// ERA5 통계 위치 (`ports`) — SO₂ · 파고
    #[serde(default)]
    pub port_id: Option<String>,
}

impl CoastalExposure {
    pub fn validate(&self) -> Result<(), String> {
        if !(self.distance_to_sea_m.is_finite() && self.distance_to_sea_m >= 0.0 && self.distance_to_sea_m <= 200_000.0) {
            return Err(format!("해안 거리 {} m 가 범위 밖입니다 (0–200 km)", self.distance_to_sea_m));
        }
        if !["harbor", "bay", "open", "tidal", "strait", "river"].contains(&self.coast_type.as_str()) {
            return Err(format!("해안 형태 '{}' 를 모릅니다 (harbor · bay · open · tidal · strait · river)", self.coast_type));
        }
        if let Some(a) = &self.salt_area {
            if kr_salt_100m(a).is_none() {
                return Err(format!("비래염분 지역 '{a}' 이 자료에 없습니다"));
            }
        }
        Ok(())
    }

    /// (100 m 비래염분 mdd, 감쇠 지수 b, 근거)
    pub fn salt_params(&self) -> (f64, f64, String) {
        let (dv, db) = coast_defaults(&self.coast_type);
        let (mut v, mut b, mut why) = (dv, db, format!("해안 형태 '{}' 기본값 (가정)", self.coast_type));
        if let Some((s, region)) = self.salt_area.as_deref().and_then(kr_salt_100m) {
            v = s;
            b = kr_decay_mean(region).unwrap_or(db);
            why = format!("국내 비래염분 {} 100 m {:.1} mdd · {} 해안 감쇠 (이종석 외 2004 · 2006)", self.salt_area.clone().unwrap_or_default(), s, region);
        }
        if let Some(s) = self.salt_100m_mdd {
            v = s;
            why = format!("입력한 100 m 비래염분 {s:.1} mdd");
        }
        if let Some(x) = self.decay_b {
            b = x;
        }
        (v, b, why)
    }

    /// 지면(2 m) 연평균 염화물 침착량 (mg Cl⁻/m²·d, 습식 캔들 환산) — `kappa` = 추가 보정 배수 (기본 1)
    pub fn chloride_annual(&self, kappa: f64) -> f64 {
        if let Some(s) = self.chloride_mg_m2_d {
            return s.max(0.0);
        }
        let (s100, b, _) = self.salt_params();
        let d = self.distance_to_sea_m.max(5.0);
        kappa * DRY_GAUZE_TO_WET_CANDLE * MDD_NACL_TO_CL * s100 * (d / 100.0).powf(-b)
    }
}

/// 염해 방지 조치 묶음 (프리셋) — 현장 프리셋 `environment.protection`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarineProtection {
    pub id: String,
    pub name: String,
    /// 구조 도장: ISO 12944 설계 범주(C3 · C4 · C5 · CX) · 내구성(L · M · H · VH)
    pub coating_category: String,
    pub coating_durability: String,
    /// 도장 아래 용융아연 두께 (µm, 0 = 없음) — 이중 계통
    #[serde(default)]
    pub duplex_hdg_um: f64,
    /// 통로 · 난간 · 계단 용융아연 두께 (µm, 0 = 도장만)
    #[serde(default)]
    pub access_hdg_um: f64,
    /// 모터 외함 (IP55 · IP56 · IP66) · 정지 중 결로 방지 히터
    pub motor_ip: String,
    pub motor_heaters: bool,
    /// 전기실 공조 · 양압 (습도 50 % 아래 유지, 염분 걸러 냄)
    pub eroom_conditioned: bool,
    /// 와이어로프 아연 도금 · 재윤활 간격 (일, 0 = 안 함)
    pub rope_galvanized: bool,
    pub rope_relube_days: f64,
    /// 감속기 흡습 브리더
    pub desiccant_breathers: bool,
    /// 담수 세척 간격 (일, 0 = 안 함)
    pub washing_days: f64,
    /// 도장 점검 간격 (일)
    pub coating_inspection_days: f64,
    /// 레일 클램프 · 스톰핀 작동 시험 간격 (일, 0 = 안 함)
    pub clamp_test_days: f64,
    /// 연간 비용 (원/년, 가정 — 결정 카드 비교용)
    #[serde(default)]
    pub annual_cost_krw: f64,
    #[serde(default)]
    pub sources: Vec<String>,
}

impl MarineProtection {
    pub fn validate(&self) -> Result<(), String> {
        if !["C3", "C4", "C5", "CX"].contains(&self.coating_category.as_str()) {
            return Err(format!("도장 설계 범주 '{}' (C3 · C4 · C5 · CX)", self.coating_category));
        }
        if !DURABILITY_YEARS.iter().any(|(k, ..)| *k == self.coating_durability) {
            return Err(format!("도장 내구성 '{}' (L · M · H · VH)", self.coating_durability));
        }
        if !["IP55", "IP56", "IP66"].contains(&self.motor_ip.as_str()) {
            return Err(format!("모터 외함 '{}' (IP55 · IP56 · IP66)", self.motor_ip));
        }
        for (k, v) in [("rope_relube_days", self.rope_relube_days), ("washing_days", self.washing_days), ("coating_inspection_days", self.coating_inspection_days), ("clamp_test_days", self.clamp_test_days), ("duplex_hdg_um", self.duplex_hdg_um), ("access_hdg_um", self.access_hdg_um)] {
            if !(v.is_finite() && v >= 0.0) {
                return Err(format!("{k} = {v} 는 0 이상이어야 합니다"));
            }
        }
        Ok(())
    }

    /// ISO 12944 내구성 범위의 중앙 (년)
    pub fn durability_mid_years(&self) -> f64 {
        DURABILITY_YEARS.iter().find(|(k, ..)| *k == self.coating_durability).map(|(_, a, b)| 0.5 * (a + b)).unwrap_or(11.0)
    }
}

pub fn protection_presets() -> Vec<MarineProtection> {
    let src_common = vec![
        "ISO 12944-1:2017 내구성 L ≤7 · M 7–15 · H 15–25 · VH >25 년".to_string(),
        "ISO 12944-5:2018 C5: H 3코트 260 µm · VH 3코트 320 µm (아연 프라이머)".to_string(),
    ];
    vec![
        MarineProtection {
            id: "none".into(),
            name: "내륙 기본 사양 (C3 · M)".into(),
            coating_category: "C3".into(),
            coating_durability: "M".into(),
            duplex_hdg_um: 0.0,
            access_hdg_um: 0.0,
            motor_ip: "IP55".into(),
            motor_heaters: false,
            eroom_conditioned: false,
            rope_galvanized: false,
            rope_relube_days: 0.0,
            desiccant_breathers: false,
            washing_days: 0.0,
            coating_inspection_days: 365.0,
            clamp_test_days: 0.0,
            annual_cost_krw: 0.0,
            sources: src_common.clone(),
        },
        MarineProtection {
            id: "port_standard".into(),
            name: "항만 표준 (C5 · H, 일반 로프 · 월 재윤활)".into(),
            coating_category: "C5".into(),
            coating_durability: "H".into(),
            duplex_hdg_um: 0.0,
            access_hdg_um: 85.0,
            motor_ip: "IP55".into(),
            motor_heaters: true,
            eroom_conditioned: true,
            rope_galvanized: false,
            rope_relube_days: 30.0,
            desiccant_breathers: false,
            washing_days: 0.0,
            coating_inspection_days: 365.0,
            clamp_test_days: 30.0,
            annual_cost_krw: 12_000_000.0,
            sources: [src_common.clone(), vec!["GAA 수명표: 85 µm 용융아연 C5 10–20년".into(), "EC&M 1996: 히터로 권선을 주변보다 5–10 °C 높게".into()]].concat(),
        },
        MarineProtection {
            id: "marine_c5vh".into(),
            name: "염해 방지 강화 (C5 · VH, 아연 로프 · 세척 · 흡습 브리더)".into(),
            coating_category: "C5".into(),
            coating_durability: "VH".into(),
            duplex_hdg_um: 0.0,
            access_hdg_um: 85.0,
            motor_ip: "IP56".into(),
            motor_heaters: true,
            eroom_conditioned: true,
            rope_galvanized: true,
            rope_relube_days: 30.0,
            desiccant_breathers: true,
            washing_days: 30.0,
            coating_inspection_days: 180.0,
            clamp_test_days: 7.0,
            annual_cost_krw: 30_000_000.0,
            sources: [
                src_common.clone(),
                vec![
                    "Koper 항만 크레인 사양서 2018: 도막 250 µm 이상 · 아연 도금 로프 · 전기실 100 % 공조 2대 · 통로 용융아연".into(),
                    "Takeuchi 1990: 해수에서 아연 도금이 로프 피로 강도를 크게 높임".into(),
                    "Cantley 1977: 베어링 수명 (100/ppm)^0.6".into(),
                ],
            ]
            .concat(),
        },
        MarineProtection {
            id: "offshore_cx".into(),
            name: "쇄파 해안 · 해상 (CX, 이중 계통)".into(),
            coating_category: "CX".into(),
            coating_durability: "H".into(),
            duplex_hdg_um: 85.0,
            access_hdg_um: 125.0,
            motor_ip: "IP66".into(),
            motor_heaters: true,
            eroom_conditioned: true,
            rope_galvanized: true,
            rope_relube_days: 14.0,
            desiccant_breathers: true,
            washing_days: 14.0,
            coating_inspection_days: 90.0,
            clamp_test_days: 7.0,
            annual_cost_krw: 55_000_000.0,
            sources: [src_common, vec!["ISO 12944-9: CX 순환 노화 4200 h".into(), "AGA: 이중 계통 수명 = 배수 × (아연 + 도장), 극한 해양 1.4–1.6".into()]].concat(),
        },
    ]
}

pub fn protection_preset(id: &str) -> Option<MarineProtection> {
    protection_presets().into_iter().find(|p| p.id == id)
}

/// 현장 프리셋의 염해 방지 지정 — 프리셋 이름 또는 전체 값
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtectionRef {
    Preset(String),
    Data(Box<MarineProtection>),
}

impl ProtectionRef {
    pub fn resolve(&self) -> Result<MarineProtection, String> {
        match self {
            ProtectionRef::Preset(n) => protection_preset(n).ok_or_else(|| {
                let names: Vec<String> = protection_presets().into_iter().map(|p| p.id).collect();
                format!("염해 방지 프리셋 '{n}' 이 없습니다 (사용 가능: {})", names.join(", "))
            }),
            ProtectionRef::Data(p) => {
                p.validate()?;
                Ok((**p).clone())
            }
        }
    }
}

// ---------------------------------------------------------------- 표면 · 상태

/// 표면 0 노출 · 1 차폐 · 2 밀폐
pub const SURFACES: [&str; 3] = ["노출", "차폐", "밀폐"];
/// 거더 · 다리 강재의 표면 비율 (노출 · 차폐) — 가정 (상자 거더 아랫면 · 안쪽 40 %)
pub const GIRDER_SHARE: [f64; 2] = [0.6, 0.4];

/// 원장에 이어 저장하는 염해 상태
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarineState {
    /// 표면별 도장 열화 (0 신품 → 1 첫 대보수)
    pub coat: [f64; 3],
    /// 표면별 녹 자리 강재의 ISO 9224 등가 시간 (년) · 도장 보수 때 굳힌 손실 (µm)
    pub tau: [f64; 3],
    pub loss_frozen_um: [f64; 3],
    /// 차폐면 표면 염 (mg Cl⁻/m², 정상 상태 대비 비율로 씀)
    pub sheltered_load: f64,
    /// 통로 · 난간 아연 소모 (µm) · 도장만이면 도장 열화
    pub access_zinc_used_um: f64,
    pub access_coat: f64,
    /// 모터 권선 오염 지수 (0 깨끗)
    pub motor_contam: [f64; 3],
    /// 감속기 오일 수분 (ppm)
    pub gear_ppm: f64,
    /// 누적 (요약용)
    pub days: f64,
    pub sd_sum: f64,
    pub tow_sum: f64,
    pub steel_um_sum: f64,
    pub last_wash_day: f64,
    pub washes: u32,
    pub repaints: u32,
    /// 마지막 도장 점검 · 고염분 알림 날
    #[serde(default)]
    pub last_coating_check: f64,
    #[serde(default)]
    pub last_salt_alert: f64,
    #[serde(default)]
    pub touch_ups: u32,
}

/// 하루 결과 — 원장 · 화면
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarineDay {
    /// 노출면 염화물 (mg Cl⁻/m²·d) · 차폐면 실효값
    pub sd: f64,
    pub sd_sheltered: f64,
    /// 젖음 시간 (h, RH > 80 % · T > 0 °C) · 결로 위험 시간 (RH ≥ 95 %)
    pub tow_h: f64,
    pub condense_h: f64,
    /// 그날 조건의 연환산 탄소강 · 아연 속도 (µm/년, 노출면)
    pub steel_rate: f64,
    pub zinc_rate: f64,
    /// 표면별 도장 열화 증가
    pub coat_inc: [f64; 3],
    /// 하루 뒤 표면별 도장 열화 (누적) · 통로 수명 사용 (누적)
    pub coat_now: [f64; 3],
    pub access_used: f64,
    /// 강재 국부 두께 손실 (µm, 가장 나쁜 표면) · 녹 면적 % (가장 나쁜 표면)
    pub local_loss_um: f64,
    pub rust_area_pct: f64,
    /// 통로 · 난간 수명 증가 (비율)
    pub access_inc: f64,
    /// 로프 수명 배수 (1 = 보정 기준 조건)
    pub rope_mult: f64,
    /// 모터 결로 · 오염으로 더해지는 절연 열화 (등급 온도 환산 h) · 추정 절연저항 (MΩ)
    pub motor_aging_h: [f64; 3],
    pub motor_ir_mohm: [f64; 3],
    /// 감속기 수분 (ppm) · 수명 소비 배수
    pub gear_ppm: f64,
    pub gear_mult: f64,
    /// 전장 ISA-71.04 등급 (0 G1 … 3 GX) · 수명 증가
    pub isa_level: u8,
    pub electrical_inc: f64,
    /// 레일 클램프 부식 수명 증가
    pub clamp_inc: f64,
    pub washed: bool,
}

/// 현장마다 한 번 계산하는 기준값
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarineSetup {
    pub exposure_note: String,
    /// 연평균: 염화물 (노출 · 차폐 정상 상태) · SO₂ 침착 · 기온 · 습도
    pub sd_annual: f64,
    pub sd_sheltered_annual: f64,
    pub pd: f64,
    pub so2_ug_m3: f64,
    pub t_annual: f64,
    pub rh_annual: f64,
    /// 연간 용량–반응식 속도 (µm/년): 표면별 탄소강 · 아연
    pub r_steel: [f64; 3],
    pub r_zinc: f64,
    pub category: String,
    pub category_zinc: String,
    pub chloride_class: String,
    /// 탄소강 장기 지수 b (노출) · 차폐면 b
    pub b_steel: f64,
    #[serde(default)]
    pub b_sheltered: f64,
    /// 구조물 평균 높이의 비래염분 배수 (지면 2 m = 1)
    #[serde(default)]
    pub height_factor: f64,
    /// 차폐면 염 누적 배수 (보정값)
    #[serde(default)]
    pub shelter_factor: f64,
    /// 날마다 계산한 속도를 연간 식에 맞추는 배수 · 바다 바람 가중 평균
    pub daily_norm: f64,
    pub wind_norm: f64,
    /// 연 젖음 시간 (h)
    pub tow_annual_h: f64,
    /// 고염분 날 문턱 (기준 해 하루 염화물의 상위 2 %, mg/m²·d)
    #[serde(default)]
    pub sd_event: f64,
    /// 표면별 도장 수명 (년) · 통로 수명 (년)
    pub coat_life_years: [f64; 3],
    pub access_life_years: f64,
    pub kappa: f64,
    pub notes: Vec<String>,
}

impl MarineSetup {
    /// 표면 k 의 장기 지수 (차폐면은 `b_sheltered`)
    pub fn b_for(&self, k: usize) -> f64 {
        if k == 1 && self.b_sheltered > 0.0 {
            self.b_sheltered
        } else {
            self.b_steel
        }
    }
}

/// 밀폐면(기계실 · 전기실) 조건: 염화물 · 습도 — 공조 · 제습은 속도에 `DEHUMIDIFIED_RATIO` 를 곱하고(아래 `surface_rate`),
/// 아니면 염 30 % · 바깥 RH − 5 (가정)
fn enclosed(sd: f64, rh: f64, conditioned: bool) -> (f64, f64) {
    if conditioned {
        (sd, rh)
    } else {
        (0.30 * sd, (rh - 5.0).max(20.0))
    }
}

/// 표면 k 의 탄소강 속도 (용량–반응식 × 제습 배수)
fn surface_rate(k: usize, pd: f64, sd: f64, rh: f64, t: f64, k_sh: f64, indoor: bool, conditioned: bool, annual: bool) -> f64 {
    let (s, h) = surface_condition(k, sd, rh, k_sh, indoor, conditioned);
    let r = if annual { drf_steel_annual(pd, s, h, t) } else { drf_steel(pd, s, h, t) };
    if k == 2 && conditioned {
        r * DEHUMIDIFIED_RATIO
    } else {
        r
    }
}

/// 표면 k 의 (염화물, 습도): 0 노출 · 1 차폐(염 누적 배수 k_sh) · 2 밀폐. 실내 크레인은 노출 · 차폐도 건물 안 조건
fn surface_condition(k: usize, sd: f64, rh: f64, k_sh: f64, indoor: bool, conditioned: bool) -> (f64, f64) {
    match k {
        0 if indoor => enclosed(sd, rh, false),
        0 => (sd, rh),
        1 if indoor => {
            let (s, r) = enclosed(sd, rh, false);
            (s * k_sh, r)
        }
        1 => (sd * k_sh, rh),
        _ => enclosed(sd, rh, conditioned),
    }
}

/// 한 시간의 바다 바람 가중 (정규화 전)
pub fn salt_weight(w: &HourWeather, sea_bearing_deg: f64, height_m: f64) -> f64 {
    let from = w.wind_from_deg.unwrap_or((w.wind_dir_deg + 180.0).rem_euclid(360.0));
    // 시뮬레이션 풍속은 크레인 높이 값 → 10 m 로 되돌린다 (α 0.14)
    let u10 = w.wind_mean_mps / (height_m.max(2.0) / 10.0).powf(0.14);
    // Meira et al. 2017 · ALC17: '3 m/s 넘는 바다 바람의 풍속 × 시간' (바다 바람 거리)이 염분을 가장 잘 설명한다
    let run = if u10 > ONSHORE_MIN_WIND { u10 } else { 0.0 };
    onshore_factor(from, sea_bearing_deg) * run + 0.1
}

fn day_means(hours: &[HourWeather]) -> (f64, f64, f64, f64) {
    let n = hours.len().max(1) as f64;
    let t = hours.iter().map(|h| h.temp_c).sum::<f64>() / n;
    let rh = hours.iter().map(|h| h.humidity_pct).sum::<f64>() / n;
    let tow = hours.iter().filter(|h| h.humidity_pct > 80.0 && h.temp_c > 0.0).count() as f64;
    let cond = hours.iter().filter(|h| h.humidity_pct >= 95.0).count() as f64;
    (t, rh, tow, cond)
}

/// 도장 수명 (년): 설계 내구성 × 환경 불일치 배수 (현장 속도가 설계 범주보다 크면 짧다), 0.2–3배로 자름.
/// 이중 계통(도장 아래 용융아연)은 `setup` 이 현장 아연 속도로 더한다
pub fn coating_life_years(p: &MarineProtection, r_site: f64) -> f64 {
    let base = p.durability_mid_years();
    let r_spec = category_mid_steel(&p.coating_category);
    base * (r_spec / r_site.max(0.5)).powf(COATING_MISMATCH_EXP).clamp(0.2, COATING_MISMATCH_MAX)
}

/// 이중 계통 배수 (AGA: 극한 해양 1.4–1.6, 산업 해양 1.8–2.0, 비공격 2.0–2.3)
pub fn duplex_factor(category: &str) -> f64 {
    match category {
        "CX" => 1.5,
        "C5" => 1.7,
        "C4" => 1.9,
        _ => 2.15,
    }
}

/// 현장 기준값을 만든다. `reference` 는 1년 이상의 시간별 날씨 (정규화용, 합성 또는 ERA5)
#[allow(clippy::too_many_arguments)]
pub fn setup(exp: &CoastalExposure, p: &MarineProtection, reference: &[HourWeather], so2_ug_m3: f64, height_m: f64, kappa: f64, indoor: bool) -> MarineSetup {
    setup_with(exp, p, reference, so2_ug_m3, height_m, kappa, indoor, crate::field_constants::SHELTER_FACTOR)
}

/// `setup` 과 같되 차폐면 염 누적 배수를 직접 준다 (보정용)
#[allow(clippy::too_many_arguments)]
pub fn setup_with(exp: &CoastalExposure, p: &MarineProtection, reference: &[HourWeather], so2_ug_m3: f64, height_m: f64, kappa: f64, indoor: bool, shelter: f64) -> MarineSetup {
    let hf = height_factor(&exp.coast_type, 0.5 * height_m.max(SALT_MEASURE_HEIGHT_M));
    let sd_ann = exp.chloride_annual(kappa) * hf;
    let n = reference.len().max(1) as f64;
    let t_ann = reference.iter().map(|h| h.temp_c).sum::<f64>() / n;
    let rh_ann = reference.iter().map(|h| h.humidity_pct).sum::<f64>() / n;
    let tow = reference.iter().filter(|h| h.humidity_pct > 80.0 && h.temp_c > 0.0).count() as f64 * 8760.0 / n;
    let pd = (SO2_DEPOSITION_PER_UG * so2_ug_m3).max(0.0);
    let w_norm = reference.iter().map(|h| salt_weight(h, exp.sea_bearing_deg, height_m)).sum::<f64>() / n;
    // 차폐면 정상 상태 (세척 없을 때 배수 shelter)
    let sd_sh = sd_ann * shelter;
    let r = [0usize, 1, 2].map(|k| surface_rate(k, pd, sd_ann, rh_ann, t_ann, shelter, indoor, p.eroom_conditioned, true));
    let (sz, hz) = surface_condition(0, sd_ann, rh_ann, shelter, indoor, p.eroom_conditioned);
    let rz = drf_zinc_annual(pd, sz, hz, t_ann);
    // 날마다 계산한 속도의 평균이 연간 식과 같도록
    let mut daily_sum = 0.0;
    let mut days = 0.0;
    for d in reference.chunks(24) {
        if d.len() < 24 {
            break;
        }
        let (t, rh, _, _) = day_means(d);
        let sd_d = sd_ann * d.iter().map(|h| salt_weight(h, exp.sea_bearing_deg, height_m)).sum::<f64>() / 24.0 / w_norm.max(1e-9);
        daily_sum += surface_rate(0, pd, sd_d, rh.clamp(20.0, 100.0), t, 1.0, indoor, p.eroom_conditioned, false);
        days += 1.0;
    }
    let daily_norm = if days > 0.0 && daily_sum > 0.0 { r[0] / (daily_sum / days) } else { 1.0 };
    // 고염분 날: 기준 해 하루 염화물 분포의 상위 2 % (연 약 7일) — 세척 판단 문턱
    let mut daily_sd: Vec<f64> = reference.chunks(24).filter(|d| d.len() == 24).map(|d| sd_ann * d.iter().map(|h| salt_weight(h, exp.sea_bearing_deg, height_m)).sum::<f64>() / 24.0 / w_norm.max(1e-9)).collect();
    daily_sd.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let sd_event = daily_sd.get(((daily_sd.len() as f64) * 0.98) as usize).cloned().unwrap_or(3.0 * sd_ann).max(sd_ann);
    let category = category_steel(r[0]).to_string();
    let b = if sd_ann > 60.0 { B_STEEL_MARINE } else { B_STEEL };
    // 염분이 있는 곳(S1 이상)의 차폐면은 보호 녹이 안 생긴다
    let b_sh = if sd_ann > 3.0 && !indoor { B_STEEL_SHELTERED } else { b };
    let coat = [coating_life_years(p, r[0]), coating_life_years(p, r[1]), coating_life_years(p, r[2]).max(coating_life_years(p, r[0]))];
    let mut coat_years = coat;
    if p.duplex_hdg_um > 0.0 {
        // 이중 계통: 배수 × (아연 수명 + 도장 수명), 아연 수명 = 두께 / 첫해 속도 (GAA 처럼 직선, 보수적)
        let f = duplex_factor(&category);
        for (k, y) in coat_years.iter_mut().enumerate() {
            let zr = if k == 0 { rz } else { rz * (r[k] / r[0].max(1e-9)) };
            *y = f * (*y + p.duplex_hdg_um / zr.max(0.05));
        }
    }
    let access = if p.access_hdg_um > 0.0 { p.access_hdg_um / rz.max(0.05) } else { coat_years[0] };
    let (s100, bdec, why) = exp.salt_params();
    let mut notes = vec![
        format!("비래염분: {why} → 거리 {:.0} m, 감쇠 b {:.2}, 환산 κ {:.2} → 연평균 {:.0} mg Cl⁻/m²·d ({})", exp.distance_to_sea_m, bdec, kappa, sd_ann, chloride_class(sd_ann)),
        format!("ISO 9223: 기온 {:.1} °C · 습도 {:.0} % · SO₂ {:.1} µg/m³ (침착 {:.1}) → 탄소강 {:.0} µm/년 ({}) · 아연 {:.2} µm/년 ({})", t_ann, rh_ann, so2_ug_m3, pd, r[0], category, rz, category_zinc(rz)),
        format!("젖음 시간 {:.0} h/년 (ISO 9223 τ: RH > 80 %, T > 0 °C)", tow),
        format!("높이 배수 {:.2} (구조물 평균 높이 {:.0} m, 배경 비율 {:.1} · 감쇠 {:.0} m — 가정), 차폐면 염 배수 {:.2}", hf, 0.5 * height_m, background_share(&exp.coast_type), SALT_HEIGHT_DECAY_M, shelter),
    ];
    if indoor {
        notes.push("실내 크레인: 노출 · 차폐면도 건물 안 조건 (염 30 % · 습도 −5, 가정)".into());
    }
    if exp.chloride_mg_m2_d.is_none() {
        notes.push(format!("100 m 값 {s100:.1} mdd (건식 거즈) — 습식 캔들(ISO 9225)과 직접 같지 않아 κ 로 맞춘다"));
    }
    MarineSetup {
        exposure_note: why,
        sd_annual: sd_ann,
        sd_sheltered_annual: sd_sh,
        pd,
        so2_ug_m3,
        t_annual: t_ann,
        rh_annual: rh_ann,
        r_steel: r,
        r_zinc: rz,
        category,
        category_zinc: category_zinc(rz).to_string(),
        chloride_class: chloride_class(sd_ann).to_string(),
        b_steel: b,
        b_sheltered: b_sh,
        height_factor: hf,
        shelter_factor: shelter,
        daily_norm,
        wind_norm: w_norm,
        tow_annual_h: tow,
        sd_event,
        coat_life_years: coat_years,
        access_life_years: access,
        kappa,
        notes,
    }
}

/// 로프 환경 배수 (1 = 보정 기준: 해안 C5 · 월 재윤활 · 일반 로프) — 재윤활 2–3배(Chevron, 판매사) ·
/// 아연 도금은 해수에서 피로 강도를 크게 높임(Takeuchi 1990, 크기 없음 → 1.2 가정) · 범주 배수는 기존 가정
pub fn rope_multiplier(category: &str, p: &MarineProtection) -> f64 {
    let env = match category {
        "C1" | "C2" => 1.0 / 0.6,
        "C3" => 0.9 / 0.6,
        "C4" => 0.75 / 0.6,
        "C5" => 1.0,
        _ => 0.45 / 0.6,
    };
    // 재윤활: 30일 이내 1, 안 하면 1/2.5, 그 사이 직선
    let lube = if p.rope_relube_days <= 0.0 { 0.4 } else if p.rope_relube_days <= 30.0 { 1.0 } else { (1.0 - 0.6 * ((p.rope_relube_days - 30.0) / 150.0).min(1.0)).max(0.4) };
    let galv = if p.rope_galvanized {
        match category {
            "C1" | "C2" => 0.95,
            "C3" => 1.0,
            _ => 1.2,
        }
    } else {
        1.0
    };
    env * lube * galv
}

/// ISA-71.04 등급 (0 G1 … 3 GX): SO₂ 경계 + 습도 50 % 위 10 % 마다 한 단계 + 염화물(S2 +1, S3 +2, 가정)
pub fn isa_level(so2_ug_m3: f64, rh: f64, sd: f64, conditioned: bool) -> u8 {
    let ppb = so2_ug_m3 / SO2_UG_PER_PPB;
    let mut g = ISA_SO2_PPB.iter().filter(|b| ppb >= **b).count() as i32;
    let rh_in = if conditioned { rh.min(45.0) } else { rh };
    g += ((rh_in - 50.0) / 10.0).floor().max(0.0) as i32;
    if !conditioned {
        g += match chloride_class(sd) {
            "S2" => 1,
            "S3" => 2,
            _ => 0,
        };
    }
    g.clamp(0, 3) as u8
}

/// ISA 등급별 전장 고장률 배수 — 가정 (ISA: G1 '부식이 요인 아님' · G2 '측정 가능' · G3 '공격 확률 높음' · GX '특수 설계만 견딤')
pub const ISA_MULT: [f64; 4] = [1.0, 1.5, 3.0, 6.0];
/// 전장(드라이브 · PLC · 제어반) 기준 수명 (년, G1) — 가정
pub const ELECTRICAL_LIFE_YEARS: f64 = 15.0;

/// Cantley (1977) 베어링 상대 수명 L = (100/ppm)^0.6 (25–400 ppm)
pub fn cantley(ppm: f64) -> f64 {
    (100.0 / ppm.clamp(25.0, 400.0)).powf(0.6)
}

/// 감속기 오일 수분 평형값 (ppm) — 가정: 100 + 300 × (일평균 RH − 60)/40, 흡습 브리더는 60 % 줄임
pub fn gear_ppm_equilibrium(rh: f64, breathers: bool) -> f64 {
    let x = 100.0 + 300.0 * ((rh - 60.0) / 40.0).clamp(0.0, 1.0);
    let x = if breathers { 100.0 + 0.4 * (x - 100.0) } else { x };
    x.clamp(25.0, 400.0)
}

/// 하루 진행. `op_frac` = 그날 운전 시간 비율 (운전 중 모터는 따뜻해 결로가 없다)
#[allow(clippy::too_many_arguments)]
pub fn day_step(s: &MarineSetup, exp: &CoastalExposure, p: &MarineProtection, st: &mut MarineState, hours: &[HourWeather], height_m: f64, abs_day: f64, op_frac: f64, insulation_used: [f64; 3], indoor: bool) -> MarineDay {
    let (t, rh, tow, cond) = day_means(hours);
    let w = hours.iter().map(|h| salt_weight(h, exp.sea_bearing_deg, height_m)).sum::<f64>() / hours.len().max(1) as f64;
    let sd = s.sd_annual * w / s.wind_norm.max(1e-9);
    // 차폐면 염: 들어옴 − 저절로 빠짐 − 세척
    let washed = p.washing_days > 0.0 && abs_day - st.last_wash_day >= p.washing_days;
    let resid = SHELTER_RESIDENCE_D;
    st.sheltered_load += sd - st.sheltered_load / resid;
    if washed {
        st.sheltered_load *= 1.0 - WASH_EFFICIENCY;
        st.last_wash_day = abs_day;
        st.washes += 1;
    }
    let load_ss = (s.sd_annual * resid).max(1e-9);
    let shelter = if s.shelter_factor > 0.0 { s.shelter_factor } else { SHELTER_SALT_FACTOR };
    let k_sh = 1.0 + (shelter - 1.0) * (st.sheltered_load / load_ss).clamp(0.0, 3.0);
    let rhc = rh.clamp(20.0, 100.0);
    let sd_sh = surface_condition(1, sd, rhc, k_sh, indoor, p.eroom_conditioned).0;
    let rates = [0usize, 1, 2].map(|k| s.daily_norm * surface_rate(k, s.pd, sd, rhc, t, k_sh, indoor, p.eroom_conditioned, false));
    let c0 = surface_condition(0, sd, rhc, k_sh, indoor, p.eroom_conditioned);
    let zr = s.daily_norm * drf_zinc(s.pd, c0.0, c0.1, t);
    let mut coat_inc = [0.0; 3];
    let mut worst_loss: f64 = 0.0;
    let mut worst_area: f64 = 0.0;
    for k in 0..3 {
        let rel = rates[k] / s.r_steel[k].max(1e-9);
        coat_inc[k] = rel / (365.0 * s.coat_life_years[k].max(0.2));
        st.coat[k] += coat_inc[k];
        if st.coat[k] >= SPOT_START_COAT {
            st.tau[k] += rel / 365.0;
        }
        let loss = st.loss_frozen_um[k] + iso9224_loss(s.r_steel[k], st.tau[k], s.b_for(k));
        if k < 2 {
            worst_loss = worst_loss.max(loss);
            worst_area = worst_area.max(rust_area_pct(st.coat[k]));
        }
    }
    // 통로 · 난간: 아연이면 소모량, 도장만이면 노출면 도장처럼
    let access_inc = if p.access_hdg_um > 0.0 {
        st.access_zinc_used_um += zr / 365.0;
        zr / 365.0 / p.access_hdg_um
    } else {
        st.access_coat += coat_inc[0];
        coat_inc[0]
    };
    // 모터: 정지 중 결로 시간 × 염 × 외함 (히터면 결로 없음) — 오염은 정기 점검 때 줄인다
    let ip = match p.motor_ip.as_str() {
        "IP66" => 0.3,
        "IP56" => 0.7,
        _ => 1.0,
    };
    let idle_cond = if p.motor_heaters { 0.0 } else { cond * (1.0 - op_frac).clamp(0.0, 1.0) };
    let mut motor_aging = [0.0; 3];
    let mut motor_ir = [0.0; 3];
    for k in 0..3 {
        // 가정: 결로 1시간 × 염 지수 (Sd/100)^0.5 × 외함 배수 × 0.002 만큼 오염이 쌓이고, 오염이 절연 열화를 0.5 h × (1 + 오염) 더한다
        st.motor_contam[k] += idle_cond * (sd / 100.0).max(0.0).sqrt() * ip * 0.002;
        motor_aging[k] = idle_cond * 0.5 * (1.0 + st.motor_contam[k]);
        // 절연저항 (MΩ): 새것 1,000 MΩ, 열 열화 · 오염 · 그날 결로로 떨어진다 (가정, 기준선 IEEE 43 5 MΩ)
        let log_ir = 3.0 - 2.0 * insulation_used[k].clamp(0.0, 1.5) - 1.5 * st.motor_contam[k].min(2.0) - if idle_cond >= 4.0 { 0.5 } else { 0.0 };
        motor_ir[k] = 10f64.powf(log_ir);
    }
    // 감속기 수분 (30일 시정수로 평형값에 다가감)
    let eq = gear_ppm_equilibrium(rh, p.desiccant_breathers);
    if st.gear_ppm <= 0.0 {
        st.gear_ppm = eq;
    }
    st.gear_ppm += (eq - st.gear_ppm) / 30.0;
    let gear_mult = cantley(150.0) / cantley(st.gear_ppm);
    // 전장
    let lvl = isa_level(s.so2_ug_m3, rh, sd, p.eroom_conditioned);
    let electrical_inc = ISA_MULT[lvl as usize] / (ELECTRICAL_LIFE_YEARS * 365.0);
    // 레일 클램프: C3 대표 속도 대비 부식 (기존 15년 기준), 작동 시험이 드물면 고착 가속 ×1.5 (가정)
    let test = if p.clamp_test_days > 0.0 && p.clamp_test_days <= 30.0 { 1.0 } else { 1.5 };
    let clamp_inc = (rates[0] / category_mid_steel("C3")).clamp(0.05, 10.0) * test / (15.0 * 365.0);
    st.days += 1.0;
    st.sd_sum += sd;
    st.tow_sum += tow;
    st.steel_um_sum += rates[0] / 365.0;
    MarineDay {
        sd,
        sd_sheltered: sd_sh,
        tow_h: tow,
        condense_h: cond,
        steel_rate: rates[0],
        zinc_rate: zr,
        coat_inc,
        coat_now: st.coat,
        access_used: if p.access_hdg_um > 0.0 { st.access_zinc_used_um / p.access_hdg_um } else { st.access_coat },
        local_loss_um: worst_loss,
        rust_area_pct: worst_area,
        access_inc,
        rope_mult: rope_multiplier(&s.category, p),
        motor_aging_h: motor_aging,
        motor_ir_mohm: motor_ir,
        gear_ppm: st.gear_ppm,
        gear_mult,
        isa_level: lvl,
        electrical_inc,
        clamp_inc,
        washed,
    }
}

impl MarineState {
    /// 도장 전체 재도장: 녹 자리 강재 손실은 남기고(굳힘) 도장 · 녹 자리 시간은 새로
    pub fn repaint(&mut self, s: &MarineSetup) {
        for k in 0..3 {
            self.loss_frozen_um[k] += iso9224_loss(s.r_steel[k], self.tau[k], s.b_for(k));
            self.tau[k] = 0.0;
            self.coat[k] = 0.0;
        }
        self.repaints += 1;
    }

    /// 부분 보수(터치업): 가장 나쁜 표면의 녹 자리를 손봐 도장 열화를 `amount` 만큼 되돌림
    pub fn touch_up(&mut self, s: &MarineSetup, amount: f64) {
        self.touch_ups += 1;
        for k in 0..2 {
            if self.coat[k] >= SPOT_START_COAT {
                self.loss_frozen_um[k] += iso9224_loss(s.r_steel[k], self.tau[k], s.b_for(k));
                self.tau[k] = 0.0;
            }
            self.coat[k] = (self.coat[k] - amount).max(0.0);
        }
    }

    /// 모터 청소 · 건조 (점검 때) — 오염을 `keep` 비율만 남긴다
    pub fn clean_motors(&mut self, keep: f64) {
        for c in self.motor_contam.iter_mut() {
            *c *= keep.clamp(0.0, 1.0);
        }
    }

    /// 담수 세척 (차폐면 염 제거)
    pub fn wash(&mut self, day: f64) {
        self.sheltered_load *= 1.0 - WASH_EFFICIENCY;
        self.last_wash_day = day;
        self.washes += 1;
    }

    pub fn local_loss_um(&self, s: &MarineSetup) -> f64 {
        (0..2).map(|k| self.loss_frozen_um[k] + iso9224_loss(s.r_steel[k], self.tau[k], s.b_for(k))).fold(0.0, f64::max)
    }
}

/// 요약 (화면 · CLI)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarineSummary {
    pub setup: MarineSetup,
    pub protection: String,
    pub days: f64,
    pub mean_sd: f64,
    pub tow_h_per_year: f64,
    pub steel_um_per_year: f64,
    pub coat: [f64; 3],
    pub rust_area_pct: f64,
    pub ri: u8,
    pub local_loss_um: f64,
    pub gear_ppm: f64,
    pub washes: u32,
    pub repaints: u32,
}

pub fn summarize(s: &MarineSetup, p: &MarineProtection, st: &MarineState) -> MarineSummary {
    let n = st.days.max(1.0);
    let area = rust_area_pct(st.coat[0].max(st.coat[1]));
    MarineSummary {
        setup: s.clone(),
        protection: p.id.clone(),
        days: st.days,
        mean_sd: st.sd_sum / n,
        tow_h_per_year: st.tow_sum / n * 365.0,
        steel_um_per_year: st.steel_um_sum / n * 365.0,
        coat: st.coat,
        rust_area_pct: area,
        ri: ri_grade(area),
        local_loss_um: st.local_loss_um(s),
        gear_ppm: st.gear_ppm,
        washes: st.washes,
        repaints: st.repaints,
    }
}

/// 노출의 SO₂ 농도: 입력값 → 항만 통계(CAMS 월평균) → 기본값
pub fn so2_for(exp: &CoastalExposure) -> f64 {
    let port = exp.port_id.as_deref().and_then(crate::ports::stats).and_then(|s| {
        let v: Vec<f64> = s.months.iter().filter_map(|m| m.so2_ug_m3).collect();
        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
    });
    exp.so2_ug_m3.or(port).unwrap_or(crate::field_constants::DEFAULT_SO2_UG_M3)
}

/// 현장의 염해 기준값 (기준 해 합성 날씨, 고정 시드)
pub fn setup_for_site(site: &crate::site::SitePreset, prot: &MarineProtection) -> Result<MarineSetup, String> {
    let exp = site.environment.coastal.clone().ok_or("해안 노출(environment.coastal) 설정이 없는 현장입니다")?;
    let climate = site.climate.resolve()?;
    let h = site.environment.crane_height_m;
    let reference = crate::weather::generate(&climate, &crate::weather::WeatherOptions { start_doy: 0, days: 365, height_m: h, seed: 0x5A17_0001 });
    Ok(setup(&exp, prot, &reference, so2_for(&exp), h, crate::field_constants::SALT_KAPPA, site.environment.indoor))
}

/// 여러 해 진행 표의 한 줄
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct YearRow {
    pub year: u32,
    pub sd_mean: f64,
    pub tow_h: f64,
    pub steel_um_y: f64,
    pub coat_exposed: f64,
    pub coat_sheltered: f64,
    pub rust_area_pct: f64,
    pub ri: u8,
    pub local_loss_um: f64,
    pub access_used: f64,
    pub gear_ppm: f64,
    pub isa_level: u8,
    pub washes: u32,
    pub touch_ups: u32,
    pub repaints: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projection {
    pub site: String,
    pub protection: MarineProtection,
    pub setup: MarineSetup,
    pub maintained: bool,
    pub rows: Vec<YearRow>,
}

/// 크레인 운전과 상관없이 염해만 `years` 년 진행한다. `maintain` 이면 도장 점검에서 Ri2 이상은 터치업, 열화 1 이상은 재도장
pub fn project(site: &crate::site::SitePreset, prot: &MarineProtection, years: u32, seed: u64, maintain: bool) -> Result<Projection, String> {
    let exp = site.environment.coastal.clone().ok_or("해안 노출(environment.coastal) 설정이 없는 현장입니다")?;
    let s = setup_for_site(site, prot)?;
    let climate = site.climate.resolve()?;
    let h = site.environment.crane_height_m;
    let indoor = site.environment.indoor;
    let w = crate::weather::generate(&climate, &crate::weather::WeatherOptions { start_doy: 0, days: 365 * years.max(1), height_m: h, seed });
    let mut st = MarineState::default();
    let mut rows = Vec::new();
    let mut acc = YearRow::default();
    let mut last: MarineDay;
    for (d, day) in w.chunks(24).enumerate() {
        if day.len() < 24 {
            break;
        }
        let abs = d as f64 + 1.0;
        last = day_step(&s, &exp, prot, &mut st, day, h, abs, 0.5, [0.3; 3], indoor);
        acc.sd_mean += last.sd;
        acc.tow_h += last.tow_h;
        acc.steel_um_y += last.steel_rate / 365.0;
        if maintain && prot.coating_inspection_days > 0.0 && abs - st.last_coating_check >= prot.coating_inspection_days {
            st.last_coating_check = abs;
            if st.coat[0].max(st.coat[1]) >= 1.0 {
                st.repaint(&s);
            } else if ri_grade(rust_area_pct(st.coat[0].max(st.coat[1]))) >= 2 {
                st.touch_up(&s, crate::field_constants::TOUCH_UP_AMOUNT);
            }
        }
        if (d + 1) % 365 == 0 {
            let area = rust_area_pct(st.coat[0].max(st.coat[1]));
            rows.push(YearRow {
                year: ((d + 1) / 365) as u32,
                sd_mean: acc.sd_mean / 365.0,
                tow_h: acc.tow_h,
                steel_um_y: acc.steel_um_y,
                coat_exposed: st.coat[0],
                coat_sheltered: st.coat[1],
                rust_area_pct: area,
                ri: ri_grade(area),
                local_loss_um: st.local_loss_um(&s),
                access_used: last.access_used,
                gear_ppm: st.gear_ppm,
                isa_level: last.isa_level,
                washes: st.washes,
                touch_ups: st.touch_ups,
                repaints: st.repaints,
            });
            acc = YearRow::default();
        }
    }
    Ok(Projection { site: site.id.clone(), protection: prot.clone(), setup: s, maintained: maintain, rows })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drf_matches_iso_examples_and_categories() {
        // 염분 없는 내륙 (Pd 5, Sd 0, RH 70, T 12): SO₂ 항만 → 약 13 µm (C2)
        let r = drf_steel(5.0, 0.0, 70.0, 12.0);
        assert!((r - 1.77 * 5f64.powf(0.52) * (0.020f64 * 70.0 - 0.054 * 2.0).exp()).abs() < 1e-9);
        assert_eq!(category_steel(r), "C2");
        // 해안 (Sd 300) 은 C4–C5
        let m = drf_steel(5.0, 300.0, 75.0, 15.0);
        assert!(m > 50.0 && m < 200.0, "{m}");
        assert_eq!(category_steel(250.0), "CX");
        assert_eq!(category_zinc(5.0), "C5");
        assert_eq!(chloride_class(61.0), "S2");
    }

    #[test]
    fn iso9224_is_continuous_and_invertible() {
        let r = 60.0;
        let a = iso9224_loss(r, 20.0 - 1e-9, B_STEEL);
        let b = iso9224_loss(r, 20.0 + 1e-9, B_STEEL);
        assert!((a - b).abs() < 1e-6);
        // ISO 9224 표 3: 탄소강 B1 t=10 → 3.334, t=20 → 4.791 (r=1)
        assert!((iso9224_loss(1.0, 10.0, B_STEEL) - 3.334).abs() < 0.002);
        assert!((iso9224_loss(1.0, 20.0, B_STEEL) - 4.791).abs() < 0.002);
        for t in [0.5, 3.0, 19.0, 35.0] {
            let d = iso9224_loss(r, t, B_STEEL);
            assert!((iso9224_time(r, d, B_STEEL) - t).abs() < 1e-6, "{t}");
        }
    }

    #[test]
    fn salt_decays_with_distance_and_follows_korean_data() {
        let mut e = CoastalExposure { distance_to_sea_m: 100.0, sea_bearing_deg: 90.0, coast_type: "open".into(), salt_area: Some("ulsan".into()), salt_100m_mdd: None, decay_b: None, chloride_mg_m2_d: None, so2_ug_m3: None, port_id: None };
        e.validate().unwrap();
        let s100 = e.chloride_annual(1.0);
        assert!((s100 - 7.6 * MDD_NACL_TO_CL * DRY_GAUZE_TO_WET_CANDLE).abs() < 1e-9);
        e.distance_to_sea_m = 1000.0;
        let s1k = e.chloride_annual(1.0);
        // 동해안 감쇠 0.57 → 10배 멀면 10^-0.57 ≈ 0.27
        assert!((s1k / s100 - 10f64.powf(-0.57)).abs() < 1e-6);
        e.salt_area = Some("nowhere".into());
        assert!(e.validate().is_err());
    }

    #[test]
    fn onshore_wind_is_directional() {
        assert_eq!(onshore_factor(180.0, 180.0), 1.0);
        assert_eq!(onshore_factor(0.0, 180.0), 0.0);
        let edge = onshore_factor(180.0 + ONSHORE_HALF_ANGLE, 180.0);
        assert!((edge - 0.5).abs() < 1e-9);
    }

    #[test]
    fn protection_presets_validate_and_rank() {
        let ps = protection_presets();
        assert!(ps.len() >= 4);
        for p in ps.iter() {
            p.validate().unwrap_or_else(|e| panic!("{}: {e}", p.id));
        }
        // 같은 C5 현장에서 VH 가 H 보다, H 가 내륙 C3 · M 보다 오래간다
        let r = 120.0;
        let life = |id: &str| coating_life_years(&protection_preset(id).unwrap(), r);
        assert!(life("marine_c5vh") > life("port_standard") && life("port_standard") > life("none"));
        // 내륙 C3 · M 사양을 C5 해안에 쓰면 3–6년 (KSCRANE 해안 일화 3–5년과 같은 범위)
        assert!(life("none") > 2.5 && life("none") < 6.5, "{}", life("none"));
        // 로프: 재윤활 안 하면 2.5배 짧다 (Chevron 2–3배)
        let none = rope_multiplier("C5", &protection_preset("none").unwrap());
        let std = rope_multiplier("C5", &protection_preset("port_standard").unwrap());
        assert!((std / none - 2.5).abs() < 1e-9);
        assert!((cantley(100.0) - 1.0).abs() < 1e-12 && (cantley(400.0) - 0.435).abs() < 0.01);
        assert_eq!(isa_level(2.0, 45.0, 0.0, true), 0);
        assert_eq!(isa_level(2.0, 75.0, 100.0, false), 3);
        assert_eq!(ri_grade(rust_area_pct(1.0)), 3);
    }

    #[test]
    fn coastal_sites_corrode_faster_and_protection_helps() {
        let sites = crate::site::builtin();
        let busan = sites.iter().find(|s| s.id == "busan_newport_rmg").unwrap();
        let ulsan = sites.iter().find(|s| s.id == "ulsan_shipyard_goliath").unwrap();
        let dangjin = sites.iter().find(|s| s.id == "dangjin_precast_yard").unwrap();
        let std = protection_preset("port_standard").unwrap();
        let su = setup_for_site(ulsan, &std).unwrap();
        let sb = setup_for_site(busan, &std).unwrap();
        let sd = setup_for_site(dangjin, &std).unwrap();
        // 동해안 도크 60 m (울산) > 남해안 만 250 m (부산 신항) > 아산만 1.5 km (당진)
        assert!(su.sd_annual > sb.sd_annual && sb.sd_annual > sd.sd_annual, "{} {} {}", su.sd_annual, sb.sd_annual, sd.sd_annual);
        // 탄소강 속도는 염분만이 아니라 SO₂ · 습도도 본다 — 당진(제철 · 발전소 SO₂)은 1.5 km 떨어져도 부산 신항과 비슷하다
        assert!(su.r_steel[0] > sb.r_steel[0] && su.r_steel[0] > sd.r_steel[0], "{} {} {}", su.r_steel[0], sb.r_steel[0], sd.r_steel[0]);
        assert!(sd.so2_ug_m3 > sb.so2_ug_m3, "{} {}", sd.so2_ug_m3, sb.so2_ug_m3);
        // 차폐면이 노출면보다 빨리 부식한다
        assert!(su.r_steel[1] > su.r_steel[0]);
        // 날마다 계산의 평균이 연간 식과 같게 맞춰진다
        let p = project(ulsan, &std, 2, 7, false).unwrap();
        let mean_rate = p.rows.iter().map(|r| r.steel_um_y).sum::<f64>() / p.rows.len() as f64;
        assert!((mean_rate / su.r_steel[0] - 1.0).abs() < 0.15, "{mean_rate} vs {}", su.r_steel[0]);
        // 같은 현장에서 염해 방지 강화가 도장 열화를 늦춘다
        let none = project(ulsan, &protection_preset("none").unwrap(), 3, 7, false).unwrap();
        let strong = project(ulsan, &protection_preset("marine_c5vh").unwrap(), 3, 7, false).unwrap();
        assert!(strong.rows[2].coat_sheltered < none.rows[2].coat_sheltered);
        assert!(strong.rows[2].gear_ppm < none.rows[2].gear_ppm);
    }
}
