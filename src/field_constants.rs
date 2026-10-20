//! 현장 모델 상수 — 알림 문턱 · 위험 배수 · 내구도 상수 · 정비 시간 · 기후 보정값을 한곳에 모은다.
//!
//! 값마다 출처를 적었다. 출처가 없는 값은 '가정' 이라고 적었고, 실데이터 보정(`fieldcal`)으로 맞춘 값은
//! '보정' 이라고 적고 목표와 보정 전 값을 docs/site-operation.md 의 오차 표에 남겼다.

use crate::alerts::{AlertPolicy, RiskModel};
use crate::health::DurabilityModel;
use crate::weather::{Climate, HourWeather};

/// 감속 운전 속도 배율 (가정)
pub const REDUCED_SPEED: f64 = 0.7;
/// 바람 흔들림 기준 사이클의 3초 돌풍 (m/s) — EN 13001-2 '보통' 운전 풍의 평균 13.3 m/s 근처를 크레인 높이 돌풍으로
pub const WIND_REF_MPS: f64 = 15.0;

/// 바퀴–레일 점착 계수 — Zhu 2011 (KTH TRITA-MMK 2011:15, Moore 인용 철도 값): 건조 0.25–0.30, 젖음 0.18–0.20,
/// 이슬 0.09–0.15, 눈 0.10. 결빙은 강–얼음 0.03 (Engineering ToolBox) 과 눈 사이의 0.05 (가정).
pub const ADHESION_DRY: f64 = 0.25;
pub const ADHESION_WET: f64 = 0.18;
pub const ADHESION_DEW: f64 = 0.10;
pub const ADHESION_SNOW: f64 = 0.10;
pub const ADHESION_ICY: f64 = 0.05;
/// 폭풍 일주(밀림) 검토용 정지 마찰 — 젖은 · 노후 레일 설계 사례 0.081 (Takahashi et al., Mech. Eng. J. 2016)
pub const ADHESION_RUNAWAY: f64 = 0.08;

pub fn adhesion_mu(w: &HourWeather) -> f64 {
    if w.icy {
        ADHESION_ICY
    } else if w.snow {
        ADHESION_SNOW
    } else if w.rain_mm > 0.2 {
        ADHESION_WET
    } else if w.fog {
        ADHESION_DEW
    } else {
        ADHESION_DRY
    }
}

/// 정기 점검 · 법정 안전검사 · 사고 조사 · 폭풍 뒤 점검 시간 (h, 가정)
pub const PERIODIC_INSPECTION_H: f64 = 4.0;
pub const LEGAL_INSPECTION_H: f64 = 8.0;
pub const INVESTIGATION_H: f64 = 4.0;
pub const POST_STORM_INSPECTION_H: f64 = 4.0;
/// 태풍 대비: 예보 최대 순간풍속 몇 시간 전부터 작업을 끝내고 고정하는가 (가정)
pub const STORM_PREP_H: usize = 6;
/// 중량물 작업계획서 · 신호수 절차로 중량물 들기마다 더 드는 시간 (s, 가정)
pub const LIFT_PLAN_EXTRA_S: f64 = 90.0;
/// 반복 중량물 작업의 표준 작업계획서 유효 기간 (일, 가정)
pub const LIFT_PLAN_VALID_DAYS: u32 = 7;
/// 체감온도 33 °C 이상 휴식 의무 — 산업안전보건기준에 관한 규칙 제560조 ③ (2시간마다 20분 이상, 2025-07-17 시행)
pub const HEAT_REST_MANDATORY: bool = true;

/// 부품 교체 · 수리 시간 (h, 가정)
pub fn replace_hours(id: &str) -> f64 {
    match id {
        x if x.starts_with("rope") => 8.0,
        x if x.starts_with("brake") => 3.0,
        x if x.starts_with("motor") => 8.0,
        x if x.starts_with("gear") => 16.0,
        x if x.starts_with("wheel") => 8.0,
        "girder" => 72.0,
        "hook" => 4.0,
        "coating" => 24.0,
        "rail_clamp" => 6.0,
        // 가정: 드라이브 · PLC 모듈 교체 8 h, 통로 · 난간 구간 교체 16 h
        "electrical" => 8.0,
        "walkway" => 16.0,
        _ => 4.0,
    }
}

pub fn alert_policy() -> AlertPolicy {
    AlertPolicy {
        // 운전 정지 = 크레인 운전 허용 풍속(제원, EN 13001-2 기본 20 m/s · TT Club RTG/RMG 22 m/s).
        // 경고 = 정지의 0.75 (STS 사전 경보 15.6 m/s — NTSB MAB-21/18, 옥외 주행 크레인 16 m/s 에서 계류 장치까지 이동 — 안전검사 고시)
        // 주의 = 정지의 0.5 (가정)
        wind_caution_mps: 10.0,
        wind_warn_mps: 15.0,
        wind_stop_mps: 20.0,
        // 기상청 강풍경보 순간풍속 26 m/s 에서 계류 · 고정, 30 m/s 초과 예상 시 이탈방지장치 작동 의무 (산업안전보건기준에 관한 규칙 제140조)
        storm_mps: 26.0,
        post_storm_inspect_mps: 30.0,
        wind_lookahead_h: 3,
        storm_lookahead_h: 24,
        // 낙뢰 16 km (10 마일) 안이면 하중 취급 중지, 마지막 낙뢰 뒤 30분 (USACE EM 385-1-1 §16.I.03)
        lightning_km: 16.0,
        lightning_clear_min: 30.0,
        // 시정: 안개 = 1 km 미만 (기상청). 크레인 중지 기준은 찾지 못해 가정값
        visibility_caution_m: 500.0,
        visibility_stop_m: 100.0,
        // 폭염작업 31 °C (제559조 ④), 33 °C 휴식 의무 (제560조 ③), 35 °C 14–17시 옥외 중지 · 38 °C 옥외 중지 (고용노동부 2026 대책)
        heat_stages_c: vec![31.0, 33.0, 35.0, 38.0],
        // 표준 옥외 권상 장비 −10 °C 까지 (Demag DH 환경 사양), 한파주의보 −12 °C (기상청)
        cold_c: -10.0,
        fatigue_shift_h: 10.0,
        inspection_notice_days: 14.0,
        // 권상용 와이어로프 안전계수 5 이상 (산업안전보건기준에 관한 규칙 제163조)
        rope_sf_legal: 5.0,
        // 야간 작업 조도: 보통작업 150 lx 이상 (제8조)
        lighting_min_lux: 150.0,
    }
}

pub fn risk_model() -> RiskModel {
    RiskModel {
        // 가정: 보고 대상 사건(아차사고 포함) 운전 1시간당 1×10⁻³. Bird(1969) 1:10:30:600 비율과 국내 천장크레인 사망 연 약 35명
        // (KOSHA 2014, 2003–2012 평균)에서 크레인 수를 가정해 역산한 크기다. 노출(크레인 수 · 운전 시간) 자료는 확인하지 못했다.
        base_per_hour: 1.0e-3,
        // 바람: 정지 문턱에서 2배, 넘겨 운전하면 다시 3배 (가정)
        wind_a: 1.0,
        wind_p: 4.0,
        wind_over: 3.0,
        // 젖은 레일 점착 −30% (Zhu 2011) → 제동 거리 증가 (가정 배수)
        wet: 1.2,
        icy: 2.0,
        fog: 1.5,
        dense_fog: 4.0,
        // 시정 주의 문턱(500 m) 아래 · 200 m 이상 — 가정 (안개 1.5 와 맑음 1 사이)
        haze: 1.2,
        lightning: 5.0,
        heat: vec![1.2, 1.5, 2.0],
        cold: 1.3,
        // 야간 1.30 · 오후 1.18 (아침 대비), 12 h 근무 1.27 (8 h 대비) — Folkard, Lombardi & Tucker, Industrial Health 2005
        night: 1.30,
        afternoon: 1.18,
        overtime_per_h: 0.0675,
        // 입사 첫 달 3배 이상 (IWH 2012) → 첫해 평균 1.5 로 줄인 값 (가정), 숙련 0.9 (가정)
        novice: 1.5,
        expert: 0.9,
        no_signalman: 1.3,
        // 2012년 천장크레인 사망자의 50% 가 주변 작업자 (KOSHA 2014) → 출입 통제가 없으면 노출 2배
        no_exclusion: 2.0,
        maint_good: 0.85,
        maint_poor: 1.5,
        critical_unplanned: 3.0,
        critical_planned: 1.3,
        rope_overdue: 4.0,
        brake_overdue: 4.0,
        component_overdue: 1.5,
        inspection_overdue: 1.3,
        legal_overdue: 2.0,
        low_light: 1.4,
    }
}

/// 사고 심각도 비율 — Bird(1969) 1 중대 : 10 경상 : 30 재산 피해 : 600 아차사고
pub const SEVERITY_SHARES: [(&str, f64); 4] = [("아차사고", 600.0), ("재산 피해", 30.0), ("경상", 10.0), ("중대", 1.0)];

pub fn durability() -> DurabilityModel {
    DurabilityModel {
        // 보정: 컨테이너 야드 크레인 권상 로프 교체 약 9개월 (RTG, 6×29Fi IWRC Ø30) 에 맞춘 값 — fieldcal
        rope_field_factor: ROPE_FIELD_FACTOR,
        // 부식 환경 로프 수명 배수 (C1 … CX). 정량 자료를 찾지 못해 가정 — 윤활 사례 6배 차이(Chesterton CS062)가 범위의 끝
        rope_env_factor: vec![1.0, 1.0, 0.9, 0.75, 0.6, 0.45],
        // 보정: 컨테이너 크레인 패드 2년 교체 (Pintsch Bubenzer SB 28: 동적 시험이 없으면 2년마다) — fieldcal
        brake_wear_per_set_mm: BRAKE_WEAR_PER_SET_MM,
        // ISO 281 오염 계수 ηc 범주 중앙값의 비 (깨끗 0.7 · 보통 0.55 · 오염 0.2 · 심한 오염 0.05), κ ≈ 1 에서 a_ISO ∝ ηc 근사
        dust_life_factor: vec![1.27, 1.0, 0.36, 0.09],
        // IEC 60085 열 등급 시험 20,000 h, 10 K 마다 수명 반감 (Montsinger)
        insulation_life_h: 20_000.0,
        insulation_halving_k: 10.0,
        // 크레인 (하루 8–16 h) 설계 L10h 15,000–30,000 h 의 중간
        motor_bearing_l10_h: 20_000.0,
        // 가정: 정격 바퀴 하중 환산 10만 km (마모율 자료 없음). 폐기 기준은 답면 직경 3% · 플랜지 50% (안전검사 고시 · Nippon Hoist)
        wheel_life_km: 100_000.0,
        wheel_rail_factor: vec![1.0, 1.6, 3.0],
        girder_detail_mpa: 71.0,
        // 가정: 설계 이용률 0.8 (설계 사이클 · 설계 스펙트럼에서 피로 손상 0.51)
        girder_utilization: 0.8,
        girder_design_kp: 0.25,
        girder_plate_mm: 12.0,
        // 거더 부식 한계: 판 두께의 10% (Nippon Hoist 점검 기준)
        girder_corrosion_limit: 0.10,
        hook_cycle_factor: 2.0,
        // 크레인 도장을 C4 '높음(H)' 계통으로 가정: C1–C3 25년, C4 20, C5 12, CX 7년 (ISO 12944 내구성 L ≤7 · M 7–15 · H 15–25 · VH >25)
        coating_life_years: vec![25.0, 25.0, 25.0, 20.0, 12.0, 7.0],
        // ISO 9223 탄소강 첫해 부식 범위의 중앙값 (µm/년): C1 ≤1.3, C2 1.3–25, C3 25–50, C4 50–80, C5 80–200, CX 200–700
        corrosion_um_per_year: vec![0.7, 13.0, 37.5, 65.0, 140.0, 450.0],
        // 장기 부식 지수 n: 내륙 0.49 · 해안 0.73 평균 (Alcántara et al., Materials 2017) — 중간값
        corrosion_exponent: 0.6,
        rail_clamp_ops: 20_000.0,
        rail_clamp_years: 15.0,
        // 관찰 0.6 · 정비 계획 0.8 (ISO 4309 '매우 높음' 심각도) · 교체 1.0
        watch_at: 0.6,
        plan_at: 0.8,
        replace_at: 1.0,
        // 남은 수명이 정기 점검 주기보다 짧으면 정비 계획 (ISO 12482 특별 평가 기준을 점검 주기에 적용)
        plan_horizon_days: 30.0,
    }
}

/// ERA5(Open-Meteo era5_seamless) → 기상청 지상 관측 기준 보정 계수 — `fieldcal`(era5cal) 이 기상청 6개 지점
/// (1991–2020 평년값 · 2022–2023 시간 자료)과 비교해 맞춘 값 (2026-10-20). 지점 하나 빼기 교차검증에서 오차가 줄어든 항목만 쓴다:
/// 기온 0.73 → 0.54 °C · 일교차 1.65 → 1.24 K · 습도 5.6 → 2.4 % · 강수량 123 → 84 mm · 강수일 32.8 → 4.4 일 · 안개 33.7 → 10.4 일 ·
/// 뇌전 3.3 → 2.8 일. 풍속 · 순간풍속 배수는 교차검증에서 오히려 나빠져(0.52 → 0.58 m/s, 10년 순간 3.3 → 4.1 m/s) 1 로 둔다 —
/// 관측소 노출(건물 · 지형) 차이라 격자 값을 한 배수로 옮길 수 없다
pub fn era5_adjust() -> crate::weather_api::Era5Adjust {
    crate::weather_api::Era5Adjust {
        temp_offset_c: 0.56,
        rh_offset_pct: -5.5,
        range_scale: 1.22,
        wind_scale: 1.0,
        gust_scale: 1.0,
        precip_scale: 1.10,
        rain_day_mm: 0.68,
        snow_day_cm: 0.01,
        fog_rh_pct: 99.0,
        thunder_mm_h: 4.8,
        source: "기상청 6개 지점 보정 (era5cal, 2026-10-20) — 풍속 배수 제외".into(),
    }
}

/// 비래염분 추가 보정 배수 κ (건식 거즈 → 습식 캔들 ×2 는 `marine` 에 이미 들어 있다) — fieldcal 이 맞춘다 (보정 전 1.0)
pub const SALT_KAPPA: f64 = 1.0;
/// 차폐면 염 누적 배수 — 보정: NIMS 차폐/노출 첫해 부식 비(초시 1.29 · 미야코지마 1.47, 평균 1.38)에 맞춘 2.07
/// (보정 전 1.6 → 비 1.23, −11 %). fieldcal 2026-10-20
pub const SHELTER_FACTOR: f64 = 2.07;
/// SO₂ 농도를 모를 때 (µg/m³) — 가정 (국내 도시 대기 2–5 ppb 수준)
pub const DEFAULT_SO2_UG_M3: f64 = 3.0;
/// 그날 염화물이 연평균의 이 배수를 넘으면 고염분 알림 (태풍 · 강한 바다 바람) — 가정
pub const SALT_EVENT_FACTOR: f64 = 3.0;
/// 부분 보수(터치업)가 되돌리는 도장 열화 — 가정 (AGA 정비 주기: 터치업 0.55T · 보수 0.7T · 전면 T)
pub const TOUCH_UP_AMOUNT: f64 = 0.3;

/// 유도 로프 · 신호수로 풍압 큰 화물의 허용 풍속을 높여 볼 수 있는 배수 — 가정 (Qld 실무 규정 · KOSHA 는 유도 로프를 대책으로 들지만 수치는 없음)
pub const TAG_LINE_LIMIT_FACTOR: f64 = 1.15;

/// 뇌우 · 강풍 중 위험물 하역의 사고율 배수 — 가정 (선박입출항법 제34조는 기준 수치 없이 관리청 재량 중지)
pub const DG_WEATHER_RISK: f64 = 3.0;

/// v3 예측 퍼짐 보정 배수 — `crane_demo prognostics` (3개 현장 · 3년 · 현장 하나 빼기)로 맞춘 값
pub const PROG_SPREAD: f64 = 1.0;
/// v4 부품 종류별 PIT 재보정 (G⁻¹ 을 τ = 0, 0.05, …, 1 에서 잰 값) — `crane_demo prognostics` 가 모든 현장 v3 기록으로 맞추고,
/// 현장 하나 빼기에서 v3 보다 나을 때만 여기에 넣는다 (비어 있으면 현장 운영은 v3).
/// 2026-10-20 (3개 현장 · 3년 · 복제 8): CRPS 147.6 → 141.5 일 · PIT KS 0.076 → 0.027 · 80 % 적중 0.82 → 0.80 → 채택.
/// 모터 · 감속기 · 로프는 PIT 가 1 쪽(예측이 짧게 치우침), 훅 · 거더 · 전장은 위 꼬리가 넓었다
#[rustfmt::skip]
pub const PROG_RECAL: &[(&str, [f64; 21])] = &[
    ("rope", [0.000, 0.085, 0.143, 0.204, 0.247, 0.329, 0.388, 0.456, 0.496, 0.543, 0.579, 0.625, 0.664, 0.726, 0.769, 0.818, 0.862, 0.891, 0.926, 0.955, 1.000]),
    ("brake", [0.000, 0.088, 0.147, 0.207, 0.260, 0.316, 0.366, 0.412, 0.454, 0.497, 0.547, 0.594, 0.644, 0.692, 0.744, 0.790, 0.834, 0.879, 0.921, 0.953, 1.000]),
    ("motor", [0.000, 0.129, 0.202, 0.262, 0.318, 0.373, 0.429, 0.476, 0.526, 0.582, 0.637, 0.686, 0.734, 0.781, 0.829, 0.866, 0.906, 0.938, 0.974, 0.997, 1.000]),
    ("gearbox", [0.000, 0.106, 0.196, 0.262, 0.329, 0.386, 0.433, 0.483, 0.533, 0.575, 0.617, 0.661, 0.705, 0.748, 0.786, 0.836, 0.870, 0.905, 0.941, 0.996, 1.000]),
    ("wheel", [0.000, 0.025, 0.070, 0.117, 0.205, 0.262, 0.319, 0.373, 0.423, 0.466, 0.515, 0.561, 0.600, 0.639, 0.695, 0.772, 0.819, 0.873, 0.925, 0.974, 1.000]),
    ("girder", [0.000, 0.038, 0.065, 0.096, 0.122, 0.153, 0.184, 0.230, 0.280, 0.319, 0.353, 0.410, 0.496, 0.589, 0.642, 0.691, 0.732, 0.779, 0.825, 0.866, 1.000]),
    ("hook", [0.000, 0.053, 0.089, 0.132, 0.185, 0.227, 0.271, 0.310, 0.349, 0.407, 0.452, 0.493, 0.529, 0.570, 0.615, 0.662, 0.696, 0.731, 0.795, 0.866, 1.000]),
    ("coating", [0.000, 0.054, 0.088, 0.137, 0.215, 0.262, 0.304, 0.344, 0.379, 0.429, 0.462, 0.514, 0.564, 0.615, 0.668, 0.707, 0.768, 0.872, 0.915, 0.953, 1.000]),
    ("rail_clamp", [0.000, 0.070, 0.130, 0.184, 0.248, 0.299, 0.346, 0.390, 0.438, 0.483, 0.553, 0.610, 0.652, 0.690, 0.720, 0.754, 0.800, 0.836, 0.882, 0.930, 1.000]),
    ("electrical", [0.000, 0.058, 0.099, 0.156, 0.205, 0.262, 0.315, 0.361, 0.403, 0.437, 0.478, 0.524, 0.566, 0.621, 0.668, 0.718, 0.754, 0.800, 0.854, 0.888, 1.000]),
    ("walkway", [0.000, 0.065, 0.103, 0.142, 0.200, 0.257, 0.331, 0.396, 0.473, 0.540, 0.580, 0.609, 0.660, 0.713, 0.769, 0.804, 0.846, 0.906, 0.943, 0.983, 1.000]),
];

pub fn prog_recal(kind: &str) -> Option<crate::scoring::PitRecal> {
    PROG_RECAL.iter().find(|(k, _)| *k == kind).map(|(_, v)| crate::scoring::PitRecal::from_knots(v, 0))
}

/// 관측 추정으로 상태를 정하는 확률 문턱 — 정책 값 (가정): 7일 안 고장 50 % 이상 교체, 30일 20 % 이상 정비 계획, 90일 20 % 이상 관찰
pub const PROG_REPLACE_P7: f64 = 0.5;
pub const PROG_PLAN_P30: f64 = 0.2;
pub const PROG_WATCH_P90: f64 = 0.2;
/// 참 상태가 교체 기준의 이 배수를 넘으면 기능을 잃는다 (돌발 고장) — 가정
pub const BREAKDOWN_TRUTH: f64 = 1.25;

/// 결정 카드: '진행' 선택의 기대 사고가 가장 안전한 선택의 이 배수 이상이면 '위험' 으로 표시한다 (정책 값, 가정)
pub const DANGER_RISK_RATIO: f64 = 2.0;
/// … 그리고 더해지는 기대 사고가 평소 운전 이 시간치(기준 사고율 × 시간)보다 클 때 — 정책 값, 가정
pub const DANGER_ABS_HOURS: f64 = 2.0;

/// 보정 결과 (fieldcal, 2026-10-20 stage 1.6) — 로프: 보정 전 1.0 (수명 437일) → 0.618 (270일 = 9개월).
/// 화물 시나리오(부산 환적: 빈 컨테이너 · Interpave 무게 분포)로 하중 스펙트럼이 가벼워져 stage 1.5 값(1.337, 보정 전 202일)과 달라졌다.
/// 브레이크: 보정 전 2.0×10⁻⁵ mm (권상 패드 69일) → 1.96×10⁻⁶ mm (730일 = 2년, 비례 맞춤 1.90 에서 752일이 나와 한 번 더 고침).
/// 둘 다 사례 하나에 맞춘 값 (신뢰도 low)
pub const ROPE_FIELD_FACTOR: f64 = 0.618;
pub const BRAKE_WEAR_PER_SET_MM: f64 = 1.96e-6;

/// 모터 열 여유의 허용 권선 온도 (°C) — 보정: ABB 주변 온도 출력 감소표(30–60 °C)에 맞춘 123.1 °C
/// (보정 전: F종 155 − 여유 10 = 145 °C → 40 °C 에서 출력 배수 1.22 로 22% 과대). 'F종 절연 · B종 상승(80 K)' 설계와 맞는다
pub const MOTOR_ALLOW_WINDING_C: f64 = 123.1;
/// 1,000 m 위 100 m 당 온도 상승 증가 비율 — 보정: ABB 고도 출력 감소표에 맞춘 1.03% (보정 전: IEC 60034-1 의 1%)
pub const ALTITUDE_RISE_PER_100M: f64 = 0.0103;

/// 고도에 따른 온도 상승 배수 (냉각 공기 밀도 감소)
pub fn altitude_rise_factor(altitude_m: f64) -> f64 {
    if altitude_m > 1000.0 {
        1.0 / (1.0 - ALTITUDE_RISE_PER_100M * (altitude_m - 1000.0) / 100.0).max(0.3)
    } else {
        1.0
    }
}

/// 기후별 보정값 — fieldcal 이 기상청 폭풍일 · 한파일 · 눈 일수에 맞춘 와이블 형상 계수 · 겨울 기온 이상치 · 눈 기온
/// 하루 습도 이상치 표준편차 (%) — fieldcal 이 기상청 지점 좌표 ERA5 시간 자료의 젖음 시간(RH > 80 %, 기상청 평균 습도 차 보정)에
/// 그럴듯한 범위 4–14 % 안에서 맞춘 값 (2026-10-20). 부산 · 울산 · 포항 · 여수는 하한 4 % 에서도 생성 젖음 시간이 7–16 % 많다 —
/// ERA5 가 일교차를 작게 그려(전체 맞춤 배수 1.22) 목표가 낮게 치우친 탓으로 본다. 항만(ERA5 기후)은 6개 지점 평균을 쓴다
pub const RH_DAILY_SD_FIT: [(&str, f64); 6] = [("busan", 4.0), ("incheon", 12.8), ("ulsan", 4.0), ("pohang", 4.0), ("yeosu", 4.0), ("seosan", 8.6)];
pub const RH_DAILY_SD_PORTS: f64 = 6.2;

pub fn apply_climate_calibration(c: &mut Climate) {
    if let Some((_, sd)) = RH_DAILY_SD_FIT.iter().find(|(id, _)| *id == c.id) {
        c.rh_daily_sd = *sd;
    }
    // (지점, 와이블 k, 겨울 기온 이상치 표준편차 °C, 눈발 확률, 강풍 사건 빈도 배수, 돌풍 계수)
    // 기상청 폭풍일 · 한파일 · 눈 일수 · 순위값 10년 순간풍속에 맞춘 값 (fieldcal, 2026-10-20 — 비 오는 날 일교차 · 비 시간 포화를 넣은 생성기)
    let fit: &[(&str, f64, f64, f64, f64, f64)] = &[
        ("busan", 2.211, 3.5, 0.047, 0.708, 1.5),
        ("incheon", 2.287, 4.26, 0.179, 0.680, 1.5),
        ("ulsan", 2.394, 3.5, 0.0, 1.075, 1.5),
        ("pohang", 2.761, 3.5, 0.035, 1.038, 1.5),
        // 여수 · 서산: 폭풍일에 맞춘 배경 바람 꼬리가 이미 극값을 만들어 사건 빈도 배수가 하한(0.1)에 닿는다 →
        // 돌풍 계수를 10년 순간풍속에 맞춘다 (1.5 → 1.39 · 1.42, 해안 개활지 돌풍 계수 1.3–1.5 범위)
        ("yeosu", 2.051, 3.5, 0.288, 0.10, 1.394),
        ("seosan", 1.595, 3.81, 0.084, 0.10, 1.418),
    ];
    if let Some((_, k, sdw, flurry, rate, gf)) = fit.iter().find(|(id, ..)| *id == c.id) {
        c.weibull_k = *k;
        c.temp_anomaly_sd_winter_c = *sdw;
        c.snow_flurry_prob = *flurry;
        c.gust_factor = *gf;
        if let Some(t) = c.typhoon.as_mut() {
            t.per_year *= *rate;
        }
    }
}
