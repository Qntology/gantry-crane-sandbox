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

/// ERA5(Open-Meteo era5_seamless) → 기상청 지상 관측 기준 보정 계수 — `fieldcal` 이 기상청 6개 지점과 비교해 맞춘다
pub fn era5_adjust() -> crate::weather_api::Era5Adjust {
    crate::weather_api::Era5Adjust::default()
}

/// 결정 카드: '진행' 선택의 기대 사고가 가장 안전한 선택의 이 배수 이상이면 '위험' 으로 표시한다 (정책 값, 가정)
pub const DANGER_RISK_RATIO: f64 = 2.0;

/// 보정 결과 (fieldcal, 2026-10-20) — 로프: 보정 전 1.0 (수명 202일) → 1.337 (270일 = 9개월).
/// 브레이크: 보정 전 2.0×10⁻⁵ mm (권상 패드 75일) → 2.05×10⁻⁶ mm (730일 = 2년)
pub const ROPE_FIELD_FACTOR: f64 = 1.337;
pub const BRAKE_WEAR_PER_SET_MM: f64 = 2.05e-6;

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
pub fn apply_climate_calibration(c: &mut Climate) {
    // (지점, 와이블 k, 겨울 기온 이상치 표준편차 °C, 눈발 확률, 강풍 사건 빈도 배수)
    // 기상청 폭풍일 · 한파일 · 눈 일수 · 순위값 10년 순간풍속에 맞춘 값 (fieldcal, 2026-10-20)
    let fit: &[(&str, f64, f64, f64, f64)] = &[
        ("busan", 2.211, 3.5, 0.068, 0.708),
        ("incheon", 2.287, 4.45, 0.179, 0.802),
        ("ulsan", 2.394, 3.5, 0.0, 1.075),
        ("pohang", 2.761, 3.5, 0.047, 1.038),
        // 여수 · 서산: 폭풍일에 맞춘 배경 바람 꼬리가 이미 극값을 만들어 사건 빈도 배수가 하한(0.1)에 닿는다
        ("yeosu", 2.051, 3.5, 0.291, 0.10),
        ("seosan", 1.571, 4.0, 0.138, 0.10),
    ];
    if let Some((_, k, sdw, flurry, rate)) = fit.iter().find(|(id, ..)| *id == c.id) {
        c.weibull_k = *k;
        c.temp_anomaly_sd_winter_c = *sdw;
        c.snow_flurry_prob = *flurry;
        if let Some(t) = c.typhoon.as_mut() {
            t.per_year *= *rate;
        }
    }
}
