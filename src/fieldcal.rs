//! 실데이터 대조 · 보정 — 조사한 관측 · 제조사 · 현장 값과 모델 출력을 비교하고, 맞출 수 있는 계수를 맞춘 뒤 오차를 다시 잰다.
//!
//! | 묶음 | 실데이터 | 맞추는 계수 |
//! | --- | --- | --- |
//! | 날씨 | 기상청 폭풍일 · 한파일 · 눈 일수 (지점별 1991–2020 평균) | 일평균 풍속 와이블 형상 계수 · 겨울 기온 이상치 · 눈 기온 |
//! | 날씨 (확인) | 기상청 평년 기온 · 강수량 · 안개 · 뇌전 일수, 순위값으로 맞춘 10 · 50년 순간풍속 | — |
//! | 모터 | ABB Motor Guide 주변 온도 · 고도 출력 감소표 | 허용 권선 온도 · 고도 100 m 당 상승 증가 |
//! | 모터 (확인) | SEW S3 출력 배수 | — |
//! | 처리량 | 야드 크레인 기술 능력 30 회/h (Konecranes EAS RTG), 코일 크레인 14 코일/h (Hoist magazine 2023) | 현장 취급 시간 |
//! | 내구도 | 컨테이너 크레인 권상 로프 약 9개월 (RTG 사례), 브레이크 패드 2년 (Pintsch Bubenzer SB 28) | 로프 현장 배수 · 체결당 마모 |
//! | 법규 (확인) | 권상 로프 안전계수 5 이상 (제163조) | — |
//!
//! 맞춘 값은 `field_constants` · `site_presets` 에 넣어 기본값이 되고, 이 보고서는 보정 전(초기 가정) · 보정 뒤 오차를 함께 남긴다.

use crate::climate_data::{self, ClimateTargets};
use crate::crane_profile;
use crate::health::DurabilityModel;
use crate::motor::{self, MotorParams, MotorThermal};
use crate::operation::{self, OpSim, OperationConfig};
use crate::site::{self, SitePreset};
use crate::weather::{self, Climate, WeatherOptions};
use serde::{Deserialize, Serialize};

/// 보정 전 초기 가정값 (보고서의 '보정 전' 열)
pub const UNCAL_WEIBULL_K: f64 = 2.0;
pub const UNCAL_TEMP_SD_WINTER: f64 = 3.5;
pub const UNCAL_SNOW_FLURRY: f64 = 0.0;
pub const UNCAL_MOTOR_ALLOW_C: f64 = 145.0;
pub const UNCAL_ALT_PER_100M: f64 = 0.01;
pub const UNCAL_ROPE_FIELD_FACTOR: f64 = 1.0;
pub const UNCAL_BRAKE_WEAR_PER_SET_MM: f64 = 2.0e-5;
pub const UNCAL_CONTAINER_HANDLING_S: f64 = 20.0;
pub const UNCAL_COIL_HANDLING_S: f64 = 40.0;

/// 목표 실데이터
pub const YARD_TECHNICAL_MPH: f64 = 30.0;
pub const COIL_DELIVERED_MPH: f64 = 14.0;
pub const ROPE_LIFE_DAYS: f64 = 270.0;
pub const BRAKE_PAD_LIFE_DAYS: f64 = 730.0;
/// ABB Motor Guide (2005) §4.2: 주변 온도 · 고도별 출력 배수
pub const ABB_AMBIENT: [(f64, f64); 8] = [(30.0, 1.07), (40.0, 1.00), (45.0, 0.965), (50.0, 0.93), (55.0, 0.90), (60.0, 0.865), (70.0, 0.79), (80.0, 0.70)];
/// 출력 감소 보정에 쓰는 주변 온도 범위 상한 — 크레인 현장 범위 (제철 실내 고온까지). 70 · 80 °C 는 확인만
pub const FIT_MAX_AMBIENT_C: f64 = 60.0;
pub const ABB_ALTITUDE: [(f64, f64); 7] = [(1000.0, 1.00), (1500.0, 0.96), (2000.0, 0.92), (2500.0, 0.88), (3000.0, 0.84), (3500.0, 0.80), (4000.0, 0.76)];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    pub id: String,
    /// "weather" · "motor" · "throughput" · "durability" · "rule"
    pub group: String,
    pub label: String,
    pub reference: f64,
    pub unit: String,
    /// 보정 전 (초기 가정) · 보정 뒤 모델 값
    pub before: f64,
    pub after: f64,
    pub error_before_pct: f64,
    pub error_after_pct: f64,
    /// 맞춘 계수 (없으면 확인만)
    pub knob: Option<String>,
    pub source: String,
    /// "high" · "med" · "low"
    pub confidence: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClimateFit {
    pub id: String,
    pub weibull_k: f64,
    pub temp_sd_winter_c: f64,
    pub snow_flurry_prob: f64,
    /// 강풍 사건 빈도 배수 — 배경 바람 꼬리까지 더한 10년 순간풍속이 순위값과 같도록
    pub storm_rate_scale: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fitted {
    pub climates: Vec<ClimateFit>,
    pub motor_allow_winding_c: f64,
    pub altitude_rise_per_100m: f64,
    pub rope_field_factor: f64,
    pub brake_wear_per_set_mm: f64,
    /// (현장 id, 취급 시간 s)
    pub handling_overhead_s: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldCalReport {
    pub benchmarks: Vec<Benchmark>,
    pub fitted: Fitted,
    /// 계수를 맞춘 항목의 평균 절대 오차 (%) — 보정 전 · 보정 뒤
    pub mean_abs_error_before_pct: f64,
    pub mean_abs_error_after_pct: f64,
    /// 모든 항목 (확인 포함)
    pub mean_abs_error_all_after_pct: f64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CalOptions {
    /// 기후 통계를 낼 생성 년수
    pub climate_years: u32,
    /// 이분법 반복
    pub fit_iters: usize,
    /// 운영 시뮬레이션 날 수 (로프 · 브레이크 · 처리량)
    pub sim_days: u32,
    /// 재현 기간 순간풍속을 낼 생성 년수
    pub return_years: u32,
    pub seed: u64,
}

impl Default for CalOptions {
    fn default() -> Self {
        Self { climate_years: 40, fit_iters: 16, sim_days: 90, return_years: 300, seed: 2026 }
    }
}

impl CalOptions {
    /// 테스트용 짧은 설정
    pub fn quick() -> Self {
        Self { climate_years: 4, fit_iters: 5, sim_days: 10, return_years: 20, seed: 7 }
    }
}

fn err_pct(model: f64, reference: f64) -> f64 {
    if reference.abs() < 1e-9 {
        // 기준이 0 인 일수: 1 일/년을 100% 로 본다
        return model * 100.0;
    }
    (model - reference) / reference.abs() * 100.0
}

#[allow(clippy::too_many_arguments)]
fn bench(id: &str, group: &str, label: &str, reference: f64, unit: &str, before: f64, after: f64, knob: Option<&str>, source: &str, confidence: &str) -> Benchmark {
    Benchmark {
        id: id.into(),
        group: group.into(),
        label: label.into(),
        reference,
        unit: unit.into(),
        before,
        after,
        error_before_pct: err_pct(before, reference),
        error_after_pct: err_pct(after, reference),
        knob: knob.map(String::from),
        source: source.into(),
        confidence: confidence.into(),
    }
}

/// 이분법: f 가 단조라고 보고 f(x) = target 인 x
fn bisect(f: &dyn Fn(f64) -> f64, mut lo: f64, mut hi: f64, target: f64, increasing: bool, iters: usize) -> f64 {
    for _ in 0..iters {
        let mid = 0.5 * (lo + hi);
        let v = f(mid);
        let go_up = if increasing { v < target } else { v > target };
        if go_up {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// 생성 날씨의 연 통계 (10 m 기준)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClimateYear {
    pub temp_c: f64,
    pub precip_mm: f64,
    pub wind_mps: f64,
    pub storm_days: f64,
    pub coldwave_days: f64,
    pub snow_days: f64,
    pub fog_days: f64,
    pub thunder_days: f64,
}

pub fn climate_year(c: &Climate, years: u32, seed: u64) -> ClimateYear {
    let w = weather::generate(c, &WeatherOptions { start_doy: 0, days: 365 * years.max(1), height_m: 10.0, seed });
    let st = weather::monthly_stats_with(&w, 10.0, c.alpha, c.gust_factor);
    let mut y = ClimateYear::default();
    for (m, s) in st.iter().enumerate() {
        let dim = weather::DAYS_IN_MONTH[m] as f64;
        y.temp_c += s.temp_mean_c * dim / 365.0;
        y.wind_mps += s.wind_mean_10m_mps * dim / 365.0;
        y.precip_mm += s.precip_mm;
        y.storm_days += s.storm_days;
        y.coldwave_days += s.coldwave_days;
        y.snow_days += s.snow_days;
        y.fog_days += s.fog_days;
        y.thunder_days += s.thunder_days;
    }
    y
}

fn with_knobs(c: &Climate, k: f64, sdw: f64, flurry: f64) -> Climate {
    let mut x = c.clone();
    x.weibull_k = k;
    x.temp_anomaly_sd_winter_c = sdw;
    x.snow_flurry_prob = flurry;
    x
}

fn with_rate(c: &Climate, scale: f64) -> Climate {
    let mut x = c.clone();
    if let Some(t) = x.typhoon.as_mut() {
        t.per_year *= scale;
    }
    x
}

/// 기후 생성기 계수를 기상청 일수에 맞춘다
pub fn fit_climate(raw: &Climate, t: &ClimateTargets, o: &CalOptions) -> ClimateFit {
    let yrs = o.climate_years;
    let seed = o.seed;
    let mut k = UNCAL_WEIBULL_K;
    let mut sdw = UNCAL_TEMP_SD_WINTER;
    let mut snow = UNCAL_SNOW_FLURRY;
    let mut scale = 1.0;
    let target_g10 = raw.typhoon.as_ref().map(|t| t.return_level(10.0));
    // 강풍 사건도 폭풍일을 만들고 배경 바람 꼬리도 극값을 만들므로 둘을 번갈아 맞춘다
    for _pass in 0..3 {
        let base = with_rate(raw, scale);
        k = bisect(&|x| climate_year(&with_knobs(&base, x, sdw, snow), yrs, seed).storm_days, 1.2, 5.0, t.storm_days, false, o.fit_iters);
        if t.coldwave_days >= 0.5 {
            sdw = bisect(&|x| climate_year(&with_knobs(&base, k, x, snow), yrs, seed).coldwave_days, 1.5, 7.0, t.coldwave_days, true, o.fit_iters);
        }
        snow = bisect(&|x| climate_year(&with_knobs(&base, k, sdw, x), yrs, seed).snow_days, 0.0, 0.9, t.snow_days, true, o.fit_iters);
        if let Some(g10) = target_g10 {
            let knobbed = with_knobs(raw, k, sdw, snow);
            scale = bisect(&|s| simulated_return_gust(&with_rate(&knobbed, s), o.return_years, seed, 10.0), 0.1, 1.5, g10, true, o.fit_iters.min(12));
        }
    }
    ClimateFit { id: raw.id.clone(), weibull_k: k, temp_sd_winter_c: sdw, snow_flurry_prob: snow, storm_rate_scale: scale }
}

/// 연 최대 순간풍속(10 m) 의 재현 기간 T 년 값 — 생성 날씨의 연 최대값 분위수
pub fn simulated_return_gust(c: &Climate, years: u32, seed: u64, t_years: f64) -> f64 {
    let w = weather::generate(c, &WeatherOptions { start_doy: 0, days: 365 * years.max(2), height_m: 10.0, seed });
    let mut annual: Vec<f64> = w.chunks(365 * 24).map(|y| y.iter().map(|h| h.gust_mps).fold(0.0, f64::max)).collect();
    annual.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q = 1.0 - 1.0 / t_years.max(1.01);
    let pos = q * (annual.len() as f64 - 1.0);
    let (i, f) = (pos.floor() as usize, pos - pos.floor());
    let a = annual[i.min(annual.len() - 1)];
    let b = annual[(i + 1).min(annual.len() - 1)];
    a + (b - a) * f
}

/// 정격 속도에서 정상상태 권선 온도가 `t_allow` 가 되는 출력 배수 (주변 온도 · 고도)
pub fn derating_factor(m: &MotorParams, ambient_c: f64, altitude_m: f64, t_allow: f64, alt_per_100m: f64) -> f64 {
    let w = m.rated_speed_rad_s();
    let alt = if altitude_m > 1000.0 { 1.0 / (1.0 - alt_per_100m * (altitude_m - 1000.0) / 100.0).max(0.3) } else { 1.0 };
    let temp = |k: f64| {
        let l = m.losses_w(k * m.rated_torque_nm(), w);
        let (tw, _) = MotorThermal::steady_state(m, &l, w, 0.0);
        ambient_c + tw * alt
    };
    bisect(&|k| temp(k), 0.05, 2.5, t_allow, true, 50)
}

fn derating_rms(t_allow: f64, alt: f64) -> (f64, f64) {
    let ms = motor::presets();
    let (mut sa, mut na, mut sh, mut nh) = (0.0, 0.0, 0.0, 0.0);
    for m in ms.iter() {
        for (t, k) in ABB_AMBIENT.iter().filter(|(t, _)| *t <= FIT_MAX_AMBIENT_C) {
            let e = derating_factor(m, *t, 0.0, t_allow, alt) - k;
            sa += e * e;
            na += 1.0;
        }
        for (h, k) in ABB_ALTITUDE {
            let e = derating_factor(m, 40.0, h, t_allow, alt) - k;
            sh += e * e;
            nh += 1.0;
        }
    }
    ((sa / na).sqrt(), (sh / nh).sqrt())
}

fn mean_derating(t_allow: f64, alt: f64, ambient: f64, altitude: f64) -> f64 {
    let ms = motor::presets();
    ms.iter().map(|m| derating_factor(m, ambient, altitude, t_allow, alt)).sum::<f64>() / ms.len() as f64
}

/// 운영 시뮬레이션 (사이클 증분표를 재사용)
fn op_run(site: &SitePreset, table: &operation::CycleTable, model: &DurabilityModel, days: u32, seed: u64) -> operation::OperationReport {
    let cfg = OperationConfig { asset_id: format!("cal-{}", site.id), days, start_month: Some(4), start_day: Some(1), seed, incidents: false, fresh: true, ..Default::default() };
    let mut sim = OpSim::with_table(site.clone(), cfg, None, table.clone()).expect("site");
    sim.model = model.clone();
    operation::run_to_end(&mut sim)
}

fn life_days(r: &operation::OperationReport, id: &str) -> f64 {
    let c = r.ledger.component(id);
    let used = c.map(|c| c.life_used + c.replacements as f64).unwrap_or(0.0);
    if used <= 1e-12 {
        return f64::INFINITY;
    }
    r.ledger.day / used
}

/// 저장소에 넣어 둔 전체 보정 보고서 (`crane_demo fieldcal` 기본 설정 결과)
pub const SAVED_REPORT_JSON: &str = include_str!("../data/reference/fieldcal_report.json");

pub fn saved_report() -> Option<FieldCalReport> {
    serde_json::from_str(SAVED_REPORT_JSON).ok()
}

pub fn run_all() -> FieldCalReport {
    run(&CalOptions::default())
}

pub fn run(o: &CalOptions) -> FieldCalReport {
    let mut out: Vec<Benchmark> = Vec::new();
    let mut fitted = Fitted::default();
    let mut notes = Vec::new();

    // ---------- 1. 날씨 ----------
    let raws = climate_data::raw_presets();
    let cals = climate_data::presets();
    for t in climate_data::targets() {
        let Some(raw) = raws.iter().find(|c| c.id == t.id) else { continue };
        let cal = cals.iter().find(|c| c.id == t.id).cloned().unwrap_or_else(|| raw.clone());
        let fit = fit_climate(raw, &t, o);
        let before = climate_year(&with_knobs(raw, UNCAL_WEIBULL_K, UNCAL_TEMP_SD_WINTER, UNCAL_SNOW_FLURRY), o.climate_years, o.seed);
        let after = climate_year(&cal, o.climate_years, o.seed);
        let src = format!("기상청 지점 {} 1991–2020 연 일수 평균", t.station);
        out.push(bench(&format!("storm_days_{}", t.id), "weather", &format!("{} 폭풍일 (10분 평균 ≥ 13.9 m/s)", raw.name), t.storm_days, "일/년", before.storm_days, after.storm_days, Some("weibull_k"), &src, "high"));
        out.push(bench(&format!("coldwave_days_{}", t.id), "weather", &format!("{} 한파일 (최저 ≤ −12 °C)", raw.name), t.coldwave_days, "일/년", before.coldwave_days, after.coldwave_days, if t.coldwave_days >= 0.5 { Some("temp_anomaly_sd_winter_c") } else { None }, &src, "high"));
        out.push(bench(&format!("snow_days_{}", t.id), "weather", &format!("{} 눈 일수", raw.name), t.snow_days, "일/년", before.snow_days, after.snow_days, Some("snow_flurry_prob"), &src, "high"));
        out.push(bench(&format!("fog_days_{}", t.id), "weather", &format!("{} 안개 일수", raw.name), t.fog_days, "일/년", before.fog_days, after.fog_days, None, &src, "high"));
        out.push(bench(&format!("thunder_days_{}", t.id), "weather", &format!("{} 뇌전 일수", raw.name), t.thunder_days, "일/년", before.thunder_days, after.thunder_days, None, &src, "high"));
        let t_ref = raw.annual_mean_temp();
        out.push(bench(&format!("temp_{}", t.id), "weather", &format!("{} 연평균 기온", raw.name), t_ref, "°C", before.temp_c, after.temp_c, None, "기상청 1991–2020 평년값", "high"));
        let p_ref: f64 = raw.months.iter().map(|m| m.precip_mm).sum();
        out.push(bench(&format!("precip_{}", t.id), "weather", &format!("{} 연 강수량", raw.name), p_ref, "mm", before.precip_mm, after.precip_mm, None, "기상청 1991–2020 평년값", "high"));
        if let Some(ty) = &raw.typhoon {
            let g10_ref = ty.return_level(10.0);
            let g10_b = simulated_return_gust(&with_knobs(raw, UNCAL_WEIBULL_K, UNCAL_TEMP_SD_WINTER, UNCAL_SNOW_FLURRY), o.return_years, o.seed, 10.0);
            let g10_a = simulated_return_gust(&cal, o.return_years, o.seed, 10.0);
            out.push(bench(&format!("gust10_{}", t.id), "weather", &format!("{} 10년 재현 순간풍속", raw.name), g10_ref, "m/s", g10_b, g10_a, Some("storm_rate_scale"), "기상청 최대 순간풍속 순위값 상위 10개의 굼벨 꼬리", "med"));
        }
        fitted.climates.push(fit);
    }

    // ---------- 2. 모터 출력 감소 ----------
    let t_fit = {
        let f = |t: f64| derating_rms(t, UNCAL_ALT_PER_100M).0;
        // RMS 를 최소로: 황금 분할
        let (mut a, mut b) = (100.0, 160.0);
        let g = 0.618_033_988_75;
        for _ in 0..40 {
            let c = b - g * (b - a);
            let d = a + g * (b - a);
            if f(c) < f(d) {
                b = d;
            } else {
                a = c;
            }
        }
        0.5 * (a + b)
    };
    let alt_fit = {
        let f = |c: f64| derating_rms(t_fit, c).1;
        let (mut a, mut b) = (0.002, 0.03);
        let g = 0.618_033_988_75;
        for _ in 0..40 {
            let c = b - g * (b - a);
            let d = a + g * (b - a);
            if f(c) < f(d) {
                b = d;
            } else {
                a = c;
            }
        }
        0.5 * (a + b)
    };
    fitted.motor_allow_winding_c = t_fit;
    fitted.altitude_rise_per_100m = alt_fit;
    let (t_now, alt_now) = (crate::field_constants::MOTOR_ALLOW_WINDING_C, crate::field_constants::ALTITUDE_RISE_PER_100M);
    for (t, k) in ABB_AMBIENT.iter() {
        out.push(bench(
            &format!("derate_ambient_{t:.0}"),
            "motor",
            &format!("모터 출력 배수 · 주변 {t:.0} °C"),
            *k,
            "×",
            mean_derating(UNCAL_MOTOR_ALLOW_C, UNCAL_ALT_PER_100M, *t, 0.0),
            mean_derating(t_now, alt_now, *t, 0.0),
            if *t <= FIT_MAX_AMBIENT_C { Some("motor_allow_winding_c") } else { None },
            "ABB The Motor Guide (2005) §4.2 주변 온도 출력 배수",
            "high",
        ));
    }
    for (h, k) in ABB_ALTITUDE.iter().filter(|(h, _)| *h > 1000.0) {
        out.push(bench(
            &format!("derate_altitude_{h:.0}"),
            "motor",
            &format!("모터 출력 배수 · 고도 {h:.0} m"),
            *k,
            "×",
            mean_derating(UNCAL_MOTOR_ALLOW_C, UNCAL_ALT_PER_100M, 40.0, *h),
            mean_derating(t_now, alt_now, 40.0, *h),
            Some("altitude_rise_per_100m"),
            "ABB The Motor Guide (2005) §4.2 고도 출력 배수",
            "high",
        ));
    }
    let s3: Vec<f64> = motor::presets().iter().flat_map(|m| motor::SEW_S3_FACTORS.iter().map(|(c, k)| (motor::s3_power_factor(m, *c, 600.0) - k) / k * 100.0).collect::<Vec<_>>()).collect();
    let s3_rms = (s3.iter().map(|e| e * e).sum::<f64>() / s3.len().max(1) as f64).sqrt();
    out.push(Benchmark {
        id: "s3_factors".into(),
        group: "motor".into(),
        label: "S3 출력 배수 (전 모터 · CDF 15–60%) RMS 오차".into(),
        reference: 0.0,
        unit: "%".into(),
        before: s3_rms,
        after: s3_rms,
        error_before_pct: s3_rms,
        error_after_pct: s3_rms,
        knob: None,
        source: "SEW-EURODRIVE 26859750 S3 출력 배수".into(),
        confidence: "high".into(),
    });

    // ---------- 3. 처리량 (현장 취급 시간) ----------
    let sites = site::builtin();
    if let Some(rmg) = sites.iter().find(|s| s.id == "busan_newport_rmg") {
        // 같은 크레인으로 베이 안 이동(주행 13 m · 횡행 12 m · 권상 10 m) 의 기술 능력
        let mut bay = rmg.clone();
        bay.throughput.travel_m = 13.0;
        bay.throughput.trolley_m = 12.0;
        bay.throughput.lift_m = 10.0;
        if let Ok(tb) = operation::build_table(&bay) {
            let cyc = tb.mean_cycle_s(false);
            let cap = |oh: f64| 3600.0 / (cyc + oh);
            let oh_fit = (3600.0 / YARD_TECHNICAL_MPH - cyc).max(5.0);
            fitted.handling_overhead_s.push((rmg.id.clone(), oh_fit));
            out.push(bench(
                "yard_technical_rate",
                "throughput",
                "야드 크레인 기술 처리 능력 (베이 안 이동)",
                YARD_TECHNICAL_MPH,
                "회/h",
                cap(UNCAL_CONTAINER_HANDLING_S),
                cap(rmg.throughput.handling_overhead_s),
                Some("handling_overhead_s (컨테이너)"),
                "Konecranes EAS RTG 기술 문서: 30 boxes/h",
                "med",
            ));
        }
    }
    if let Some(coil) = sites.iter().find(|s| s.id == "gwangyang_coil_warehouse") {
        if let Ok(tb) = operation::build_table(coil) {
            let m = DurabilityModel::default();
            let delivered = |oh: f64| {
                let mut s = coil.clone();
                s.throughput.handling_overhead_s = oh;
                let r = op_run(&s, &tb, &m, o.sim_days.min(30), o.seed);
                r.kpi.moves / r.kpi.scheduled_h.max(1.0)
            };
            let before = delivered(UNCAL_COIL_HANDLING_S);
            let after = delivered(coil.throughput.handling_overhead_s);
            // 처리량이 수요(14/h)를 넘을 수는 없으므로 수요의 98% 를 목표로 취급 시간을 줄인다
            let oh_fit = bisect(&|x| delivered(x), 5.0, 120.0, 0.98 * COIL_DELIVERED_MPH, false, o.fit_iters.min(10));
            fitted.handling_overhead_s.push((coil.id.clone(), oh_fit));
            out.push(bench(
                "coil_delivered_rate",
                "throughput",
                "코일 크레인 처리량 (계획 시간당)",
                COIL_DELIVERED_MPH,
                "회/h",
                before,
                after,
                Some("handling_overhead_s (코일)"),
                "Hoist magazine (2023) Running the mill: 하루 약 1,000 코일 · 크레인 3대",
                "med",
            ));
        }
    }

    // ---------- 4. 내구도 (로프 · 브레이크) ----------
    if let Some(rmg) = sites.iter().find(|s| s.id == "busan_newport_rmg") {
        if let Ok(tb) = operation::build_table(rmg) {
            let m0 = DurabilityModel { rope_field_factor: UNCAL_ROPE_FIELD_FACTOR, brake_wear_per_set_mm: UNCAL_BRAKE_WEAR_PER_SET_MM, ..DurabilityModel::default() };
            let r0 = op_run(rmg, &tb, &m0, o.sim_days, o.seed);
            let m1 = DurabilityModel::default();
            let r1 = op_run(rmg, &tb, &m1, o.sim_days, o.seed);
            let rope0 = life_days(&r0, "rope_hoist");
            let rope1 = life_days(&r1, "rope_hoist");
            let brake0 = life_days(&r0, "brake_hoist");
            let brake1 = life_days(&r1, "brake_hoist");
            // 로프 수명 ∝ 현장 배수, 브레이크 수명 ∝ 1 / 체결당 마모 (비상 정지 마찰일 마모는 거의 없다)
            fitted.rope_field_factor = UNCAL_ROPE_FIELD_FACTOR * ROPE_LIFE_DAYS / rope0.max(1e-9);
            fitted.brake_wear_per_set_mm = UNCAL_BRAKE_WEAR_PER_SET_MM * brake0 / BRAKE_PAD_LIFE_DAYS;
            out.push(bench("rope_life", "durability", "컨테이너 야드 크레인 권상 로프 교체 주기", ROPE_LIFE_DAYS, "일", rope0, rope1, Some("rope_field_factor"), "UM Surabaya RTG 로프 논문: 6×29Fi IWRC Ø30, 9개월", "low"));
            out.push(bench("brake_pad_life", "durability", "컨테이너 크레인 권상 브레이크 패드 교체 주기", BRAKE_PAD_LIFE_DAYS, "일", brake0, brake1, Some("brake_wear_per_set_mm"), "Pintsch Bubenzer SB 28: 동적 시험이 없으면 2년마다 패드 교체", "low"));
        }
    }

    // ---------- 5. 법규 확인 ----------
    for c in crane_profile::presets() {
        let s = &c.structure;
        let per_fall = (s.rated_load_kg + s.hook_mass_kg) * crane_profile::G / c.hoist.reeving_falls.max(1.0);
        let sf = c.rope.min_breaking_force_kn * 1000.0 / per_fall.max(1.0);
        out.push(Benchmark {
            id: format!("rope_sf_{}", c.name),
            group: "rule".into(),
            label: format!("{} 정격 정적 로프 안전계수 (≥ 5)", c.name),
            reference: 5.0,
            unit: "".into(),
            before: sf,
            after: sf,
            error_before_pct: if sf >= 5.0 { 0.0 } else { err_pct(sf, 5.0) },
            error_after_pct: if sf >= 5.0 { 0.0 } else { err_pct(sf, 5.0) },
            knob: None,
            source: "산업안전보건기준에 관한 규칙 제163조".into(),
            confidence: "high".into(),
        });
    }

    let knobbed: Vec<&Benchmark> = out.iter().filter(|b| b.knob.is_some()).collect();
    let mae = |v: &[&Benchmark], f: &dyn Fn(&Benchmark) -> f64| if v.is_empty() { 0.0 } else { v.iter().map(|b| f(b).abs()).sum::<f64>() / v.len() as f64 };
    let all: Vec<&Benchmark> = out.iter().filter(|b| b.reference.abs() > 1e-9 || b.group == "weather").collect();
    notes.push(format!("기후 통계는 생성 {} 년 · 재현 순간풍속은 {} 년, 운영은 {} 일 (시드 {})", o.climate_years, o.return_years, o.sim_days, o.seed));
    notes.push("기준이 0 인 일수(예: 부산 한파일)는 모델 1 일/년을 100% 오차로 셉니다".into());
    notes.push("신뢰도 low 항목(로프 · 브레이크 교체 주기)은 사례 하나에 맞춘 값입니다 — 현장 기록으로 다시 맞추세요".into());
    FieldCalReport {
        mean_abs_error_before_pct: mae(&knobbed, &|b| b.error_before_pct),
        mean_abs_error_after_pct: mae(&knobbed, &|b| b.error_after_pct),
        mean_abs_error_all_after_pct: mae(&all, &|b| b.error_after_pct),
        benchmarks: out,
        fitted,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derating_follows_temperature_and_altitude() {
        let m = motor::preset("im_37kw_4p").unwrap();
        let k40 = derating_factor(&m, 40.0, 0.0, 120.0, 0.01);
        let k60 = derating_factor(&m, 60.0, 0.0, 120.0, 0.01);
        let kalt = derating_factor(&m, 40.0, 3000.0, 120.0, 0.01);
        assert!(k60 < k40 && kalt < k40, "{k40} {k60} {kalt}");
        assert!((k40 - 1.0).abs() < 0.08, "{k40}");
    }

    #[test]
    fn bisect_finds_root() {
        let x = bisect(&|x| x * x, 0.0, 3.0, 4.0, true, 40);
        assert!((x - 2.0).abs() < 1e-6);
    }

    #[test]
    fn quick_report_has_all_groups() {
        let r = run(&CalOptions::quick());
        for g in ["weather", "motor", "throughput", "durability", "rule"] {
            assert!(r.benchmarks.iter().any(|b| b.group == g), "{g}");
        }
        assert!(r.fitted.motor_allow_winding_c > 100.0 && r.fitted.motor_allow_winding_c < 160.0);
        assert_eq!(r.fitted.climates.len(), climate_data::targets().len());
        assert!(r.fitted.rope_field_factor > 0.0);
    }
}
