//! 현장 기후 · 시간별 날씨 생성 · 예보 · 체감온도.
//!
//! 월별 기후 평년값(기온 · 일교차 · 풍속 · 강수 · 뇌전 · 안개 · 습도)과 태풍 빈도에서 시간별 날씨를 시드로 재현 가능하게 만든다.
//! - 기온: 월 중앙값 사이를 선형 보간한 계절 곡선 + 일 이상치 AR(1) + 일교차 코사인 (최고 15시, 최저 3시).
//! - 바람: 일평균 풍속은 월평균을 평균으로 하는 와이블 분포(가우스 코퓰라 AR(1) 로 날짜 간 지속성),
//!   시간 변동은 낮 최대의 일변화 × 로그 정규 AR(1). 크레인 높이 풍속 = 10 m 풍속 × (h/10)^α,
//!   3초 돌풍 = 평균 × 돌풍 계수 (EN 13001-2:2021 운전 풍 '보통' 상태 20 / 13.3 m/s = 1.5).
//! - 강수: 강수일 확률(월 강수일수 / 일수)의 2상태 마르코프 연쇄, 비 오는 시간은 한 덩어리, 강도는 월 강수량에 맞춘다.
//!   기온 1 °C 이하의 강수는 눈, 영하에서 젖은 레일은 결빙.
//! - 뇌전 · 안개: 월 일수로 그날 여부를 정하고, 뇌전은 오후, 안개는 새벽에 몇 시간 이어진다.
//! - 태풍: 연 영향 횟수 × 월 가중치의 포아송 사건. 최대 순간풍속은 로그 정규(중앙값 · 90% 값).
//!
//! 체감온도는 기상청 산출식(여름: Stull 습구온도 기반, 겨울: 풍속 냉각)을 쓴다.

use crate::rng::{gamma, norm_cdf, Rng};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

pub const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// 연중 날짜(0 = 1월 1일, 평년) → 월(0–11)
pub fn month_of(doy: u32) -> usize {
    let mut d = doy % 365;
    for (m, n) in DAYS_IN_MONTH.iter().enumerate() {
        if d < *n {
            return m;
        }
        d -= n;
    }
    11
}

/// 월(1–12) · 일 → 연중 날짜 (0 기반)
pub fn doy_of(month: u32, day: u32) -> u32 {
    let m = month.clamp(1, 12) as usize;
    DAYS_IN_MONTH[..m - 1].iter().sum::<u32>() + day.clamp(1, DAYS_IN_MONTH[m - 1]) - 1
}

/// 연중 날짜 → "MM-DD"
pub fn date_label(doy: u32) -> String {
    let mut d = doy % 365;
    for (m, n) in DAYS_IN_MONTH.iter().enumerate() {
        if d < *n {
            return format!("{:02}-{:02}", m + 1, d + 1);
        }
        d -= n;
    }
    "12-31".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthClimate {
    /// 월평균 기온 (°C)
    pub temp_mean_c: f64,
    /// 평균 일교차 (일 최고 − 일 최저, K)
    pub temp_range_c: f64,
    /// 10 m 높이 월평균 풍속 (m/s)
    pub wind_mean_mps: f64,
    /// 월 강수량 (mm)
    pub precip_mm: f64,
    /// 강수일수 (일 강수량 0.1 mm 이상)
    pub rain_days: f64,
    /// 눈 일수 (평년값 — 생성은 기온으로 하고 이 값은 보정 비교에 쓴다)
    #[serde(default)]
    pub snow_days: f64,
    /// 뇌전 일수
    pub thunder_days: f64,
    /// 안개 일수
    pub fog_days: f64,
    /// 평균 상대습도 (%)
    pub humidity_pct: f64,
}

/// 태풍 · 발달한 저기압 같은 강풍 사건 (임계값 초과 봉우리 모델, peaks over threshold).
///
/// 최대 순간풍속(10 m)이 `threshold_mps` 이상인 사건이 연 `per_year` 번(포아송) 오고, 봉우리는
/// threshold + 지수 분포(척도 β) — 굼벨 꼬리와 같다. 기상청 지점별 최대 순간풍속 순위값(상위 10개)의
/// 초과 빈도 ln(순위/기록 년수) 를 풍속에 직선 회귀해 β 와 빈도를 정했다 (`fieldcal` 이 다시 확인한다).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TyphoonClimate {
    /// 최대 순간풍속이 threshold 이상인 사건 수 (회/년)
    pub per_year: f64,
    /// 사건 임계 순간풍속 (10 m, m/s)
    pub threshold_mps: f64,
    /// 초과분 지수 분포 척도 β (m/s)
    pub excess_scale_mps: f64,
    /// 월별 가중치 12개 (합이 1 이 아니어도 정규화)
    pub month_weights: Vec<f64>,
    /// 강풍이 이어지는 시간 (h)
    pub duration_h: f64,
    pub source: String,
}

impl TyphoonClimate {
    /// 연 초과 빈도 λ(v) — 최대 순간풍속 v 이상 사건 (회/년)
    pub fn exceedance_per_year(&self, v: f64) -> f64 {
        if v <= self.threshold_mps {
            return self.per_year;
        }
        self.per_year * (-(v - self.threshold_mps) / self.excess_scale_mps.max(0.1)).exp()
    }

    /// 이 달 태풍 사건이 주는 기대 강수량 (mm) — 사건당 약 14 h × 평균 12.5 mm/h (6–10월만 호우)
    pub fn expected_rain_mm(&self, month: usize) -> f64 {
        if !(5..=9).contains(&month) {
            return 0.0;
        }
        let wsum: f64 = self.month_weights.iter().map(|w| w.max(0.0)).sum::<f64>().max(1e-12);
        let per_event = 0.78 * self.duration_h * 12.5;
        self.per_year * self.month_weights.get(month).cloned().unwrap_or(0.0).max(0.0) / wsum * per_event
    }

    /// 재현 기간 T 년 순간풍속 (λ(v) = 1/T)
    pub fn return_level(&self, years: f64) -> f64 {
        let r = self.per_year * years.max(1e-6);
        if r <= 1.0 {
            return self.threshold_mps;
        }
        self.threshold_mps + self.excess_scale_mps * r.ln()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Climate {
    pub id: String,
    pub name: String,
    /// 평년값 출처
    pub source: String,
    /// 1–12월
    pub months: Vec<MonthClimate>,
    /// 3초 돌풍 / 평균 풍속
    #[serde(default = "default_gust_factor")]
    pub gust_factor: f64,
    /// 일평균 풍속 와이블 형상 계수
    #[serde(default = "default_weibull_k")]
    pub weibull_k: f64,
    /// 시간 평균 풍속의 로그 표준편차 (하루 안 변동)
    #[serde(default = "default_hourly_sd")]
    pub hourly_wind_sd: f64,
    /// 풍속 고도 지수 α (개활지 · 해안 0.14 근처)
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    /// 일평균 기온 이상치 표준편차 (K) — 여름(7월) 값과 겨울(1월) 값, 사이는 코사인으로 잇는다
    #[serde(default = "default_temp_sd")]
    pub temp_anomaly_sd_c: f64,
    #[serde(default = "default_temp_sd_winter")]
    pub temp_anomaly_sd_winter_c: f64,
    /// 이 기온 이하의 강수는 눈 (°C)
    #[serde(default = "default_snow_temp")]
    pub snow_temp_c: f64,
    /// 강수 없는 추운 날(일평균 2 °C 이하)에 눈발이 날릴 확률 — 기상청 눈 일수는 강수량이 없는 눈도 센다
    #[serde(default)]
    pub snow_flurry_prob: f64,
    #[serde(default)]
    pub typhoon: Option<TyphoonClimate>,
}

fn default_gust_factor() -> f64 {
    1.5
}
fn default_weibull_k() -> f64 {
    2.0
}
fn default_hourly_sd() -> f64 {
    0.25
}
fn default_alpha() -> f64 {
    0.14
}
fn default_temp_sd() -> f64 {
    1.8
}
fn default_temp_sd_winter() -> f64 {
    3.5
}
fn default_snow_temp() -> f64 {
    1.5
}

impl Climate {
    pub fn validate(&self) -> Result<(), String> {
        if self.months.len() != 12 {
            return Err(format!("기후 {}: months 는 12개여야 합니다 ({})", self.id, self.months.len()));
        }
        for (i, m) in self.months.iter().enumerate() {
            let dim = DAYS_IN_MONTH[i] as f64;
            for (k, v, hi) in [
                ("rain_days", m.rain_days, dim),
                ("thunder_days", m.thunder_days, dim),
                ("fog_days", m.fog_days, dim),
                ("snow_days", m.snow_days, dim),
                ("wind_mean_mps", m.wind_mean_mps, 40.0),
                ("humidity_pct", m.humidity_pct, 100.0),
            ] {
                if !(v.is_finite() && v >= 0.0 && v <= hi) {
                    return Err(format!("기후 {} {}월 {} = {} 이 범위 밖입니다", self.id, i + 1, k, v));
                }
            }
        }
        if !(self.gust_factor >= 1.0 && self.weibull_k > 0.5) {
            return Err(format!("기후 {}: gust_factor ≥ 1, weibull_k > 0.5 여야 합니다", self.id));
        }
        Ok(())
    }

    /// 월 중앙값 사이 선형 보간 (연중 날짜 기준)
    fn interp(&self, doy: f64, f: impl Fn(&MonthClimate) -> f64) -> f64 {
        let mids: Vec<f64> = (0..12).map(|m| DAYS_IN_MONTH[..m].iter().sum::<u32>() as f64 + DAYS_IN_MONTH[m] as f64 / 2.0).collect();
        let d = doy.rem_euclid(365.0);
        let mut i = 11;
        for m in 0..12 {
            if d >= mids[m] {
                i = m;
            }
        }
        let (a, b) = if d < mids[0] { (11, 0) } else { (i, (i + 1) % 12) };
        let (ma, mb) = (mids[a], if b == 0 && a == 11 { mids[0] + 365.0 } else { mids[b] });
        let dd = if d < mids[0] { d + 365.0 } else { d };
        let w = ((dd - ma) / (mb - ma)).clamp(0.0, 1.0);
        f(&self.months[a]) * (1.0 - w) + f(&self.months[b]) * w
    }

    pub fn annual_mean_temp(&self) -> f64 {
        self.months.iter().zip(DAYS_IN_MONTH.iter()).map(|(m, d)| m.temp_mean_c * *d as f64).sum::<f64>() / 365.0
    }
}

/// 한 시간의 날씨 (크레인 높이 기준 바람)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HourWeather {
    /// 시뮬레이션 시작부터의 날 (0 기반)과 시 (0–23)
    pub day: u32,
    pub hour: u8,
    /// 연중 날짜 (0 = 1월 1일)
    pub doy: u32,
    pub temp_c: f64,
    pub humidity_pct: f64,
    /// 크레인 높이 시간 평균 풍속 (m/s)
    pub wind_mean_mps: f64,
    /// 크레인 높이 3초 돌풍 (이 시간 최대, m/s)
    pub gust_mps: f64,
    /// 풍향: 0° = 주행 방향(+x), 90° = 횡행 방향(+y)
    pub wind_dir_deg: f64,
    /// 시간 강수량 (mm)
    pub rain_mm: f64,
    pub snow: bool,
    /// 레일 · 바퀴 결빙 (영하 + 최근 강수 · 안개)
    pub icy: bool,
    /// 가장 가까운 낙뢰 거리 (km), 없으면 None
    pub thunder_km: Option<f64>,
    pub visibility_m: f64,
    /// 안개 (시정 1 km 미만, 기상청 정의)
    #[serde(default)]
    pub fog: bool,
    /// 태풍 · 강풍 사건 중
    pub typhoon: bool,
    /// 체감온도 (°C, 기상청 식)
    pub feels_like_c: f64,
}

impl HourWeather {
    /// 실내 크레인: 바깥 날씨에서 실내 조건으로 (바람 · 강수 · 낙뢰 · 시정 영향 없음)
    pub fn indoor(&self, base_c: f64, coupling: f64, heat_source_c: f64) -> HourWeather {
        let t = base_c + coupling * (self.temp_c - base_c) + heat_source_c;
        let rh = self.humidity_pct.clamp(30.0, 80.0);
        HourWeather {
            temp_c: t,
            humidity_pct: rh,
            wind_mean_mps: 0.0,
            gust_mps: 0.0,
            rain_mm: 0.0,
            snow: false,
            icy: false,
            thunder_km: None,
            visibility_m: 10_000.0,
            fog: false,
            typhoon: false,
            feels_like_c: feels_like_c(t, rh, 0.0, true),
            ..self.clone()
        }
    }

    pub fn label(&self) -> String {
        format!("D{} {:02}:00", self.day + 1, self.hour)
    }

    /// 풍속을 주행(x) · 횡행(y) 성분으로
    pub fn wind_components(&self, v: f64) -> [f64; 2] {
        let a = self.wind_dir_deg.to_radians();
        [v * a.cos(), v * a.sin()]
    }
}

/// Stull(2011) 습구온도 (°C)
pub fn wet_bulb_stull(t: f64, rh: f64) -> f64 {
    let rh = rh.clamp(5.0, 99.0);
    t * (0.151_977 * (rh + 8.313_659).sqrt()).atan() + (t + rh).atan() - (rh - 1.676_331).atan() + 0.003_918_38 * rh.powf(1.5) * (0.023_101 * rh).atan() - 4.686_035
}

/// 체감온도 (기상청): 여름(5–9월)은 습구온도 기반 식, 겨울은 풍속 냉각 지수(기온 10 °C 이하, 풍속 1.3 m/s 이상)
pub fn feels_like_c(t: f64, rh: f64, wind_10m_mps: f64, summer: bool) -> f64 {
    if summer {
        let tw = wet_bulb_stull(t, rh);
        -0.2442 + 0.553_99 * tw + 0.455_35 * t - 0.0022 * tw * tw + 0.002_78 * tw * t + 3.0
    } else {
        let v = wind_10m_mps * 3.6;
        if t <= 10.0 && v >= 4.8 {
            let p = v.powf(0.16);
            13.12 + 0.6215 * t - 11.37 * p + 0.3965 * p * t
        } else {
            t
        }
    }
}

fn is_summer(month: usize) -> bool {
    (4..=8).contains(&month)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WeatherOptions {
    /// 시작 연중 날짜 (0 = 1월 1일)
    pub start_doy: u32,
    pub days: u32,
    /// 바람을 잴 높이 (크레인 거더 · 트롤리 높이, m)
    pub height_m: f64,
    pub seed: u64,
}

/// 시간별 날씨를 만든다 (길이 = days × 24)
pub fn generate(c: &Climate, o: &WeatherOptions) -> Vec<HourWeather> {
    let mut rng = Rng::new(o.seed ^ 0x5745_4154_4845_5221);
    let k = c.weibull_k.clamp(0.8, 4.0);
    let g1k = gamma(1.0 + 1.0 / k);
    let hfac = (o.height_m.max(2.0) / 10.0).powf(c.alpha.clamp(0.0, 0.4));
    let n_h = (o.days as usize) * 24;
    let mut out: Vec<HourWeather> = Vec::with_capacity(n_h);

    // 강풍 사건 (시작 시간 색인, 최고 순간풍속 10 m, 지속 h) — 임계값 초과 봉우리 모델
    let mut typhoons: Vec<(usize, f64, f64)> = Vec::new();
    if let Some(ty) = &c.typhoon {
        let wsum: f64 = ty.month_weights.iter().map(|w| w.max(0.0)).sum::<f64>().max(1e-12);
        let mut trng = rng.fork(0x7479);
        for d in 0..o.days {
            let m = month_of(o.start_doy + d);
            let p = ty.per_year * ty.month_weights.get(m).cloned().unwrap_or(0.0).max(0.0) / wsum / DAYS_IN_MONTH[m] as f64;
            if trng.chance(p) {
                let peak = ty.threshold_mps + trng.exp(ty.excess_scale_mps.max(0.1));
                let start = d as usize * 24 + (trng.uniform() * 24.0) as usize;
                typhoons.push((start, peak, ty.duration_h.max(6.0)));
            }
        }
    }

    let mut t_anom = 0.0;
    let mut z_wind = 0.0;
    let mut wet_prev = false;
    let mut x_h = 0.0;
    let mut last_wet_h: Option<usize> = None;
    for d in 0..o.days {
        let doy = o.start_doy + d;
        let m = month_of(doy);
        let mc = &c.months[m];
        let dim = DAYS_IN_MONTH[m] as f64;
        // 기온
        let phi_t: f64 = 0.7;
        let winter = 0.5 * (1.0 + (2.0 * PI * (doy as f64 - 15.0) / 365.0).cos());
        let t_sd = c.temp_anomaly_sd_c + (c.temp_anomaly_sd_winter_c - c.temp_anomaly_sd_c) * winter;
        t_anom = phi_t * t_anom + t_sd * (1.0 - phi_t * phi_t).sqrt() * rng.normal();
        let t_mean = c.interp(doy as f64, |x| x.temp_mean_c) + t_anom;
        let range = c.interp(doy as f64, |x| x.temp_range_c).max(0.0);
        // 바람 (일평균, 10 m)
        let phi_w: f64 = 0.5;
        z_wind = phi_w * z_wind + (1.0 - phi_w * phi_w).sqrt() * rng.normal();
        let u = norm_cdf(z_wind).clamp(1e-9, 1.0 - 1e-9);
        let w_mean = c.interp(doy as f64, |x| x.wind_mean_mps).max(0.1);
        let w_day = w_mean / g1k * (-(1.0 - u).ln()).powf(1.0 / k);
        let dir_day = rng.uniform() * 360.0;
        // 강수 (2상태 마르코프, 지연 상관 0.3)
        let p_wet = (mc.rain_days / dim).clamp(0.0, 0.98);
        let rho = 0.3;
        let p_now = if wet_prev { p_wet + rho * (1.0 - p_wet) } else { p_wet * (1.0 - rho) };
        let wet = rng.chance(p_now);
        wet_prev = wet;
        // 비 오는 시간은 하루 안에 다 들어가게 (자정에서 잘리면 강수량이 모자란다)
        let rain_len = if wet { 3 + (rng.uniform() * 10.0) as usize } else { 0 };
        let rain_start = if wet { (rng.uniform() * (24 - rain_len + 1) as f64) as usize } else { 0 };
        // 강수 없는 추운 날의 눈발 (흔적 강설)
        let flurry = !wet && t_mean <= 2.0 && rng.chance(c.snow_flurry_prob);
        let (fl_start, fl_len) = (4 + (rng.uniform() * 14.0) as usize, 1 + (rng.uniform() * 3.0) as usize);
        // 평년 강수량에는 태풍 비가 이미 들어 있으므로, 태풍 사건이 줄 기대 강수량만큼 보통 비를 줄인다
        let ty_rain = c.typhoon.as_ref().map(|t| t.expected_rain_mm(m)).unwrap_or(0.0);
        let base_share = (1.0 - ty_rain / mc.precip_mm.max(1.0)).clamp(0.3, 1.0);
        let mean_rate = if mc.rain_days > 0.0 { base_share * mc.precip_mm / mc.rain_days / 7.5 } else { 0.0 };
        // 뇌전 · 안개
        let thunder = rng.chance(mc.thunder_days / dim);
        let (th_start, th_len) = (12 + (rng.uniform() * 8.0) as usize, 1 + (rng.uniform() * 3.0) as usize);
        let fog = rng.chance(mc.fog_days / dim);
        let (fog_start, fog_len) = (2 + (rng.uniform() * 4.0) as usize, 2 + (rng.uniform() * 5.0) as usize);
        let rh_day = (mc.humidity_pct + if wet { 12.0 } else { 0.0 } + 4.0 * rng.normal()).clamp(15.0, 100.0);
        for h in 0..24usize {
            let idx = d as usize * 24 + h;
            let hr = h as f64;
            let in_rain = wet && h >= rain_start && h < rain_start + rain_len;
            let r = if in_rain { 0.5 } else { 1.0 };
            let temp = t_mean + 0.5 * range * r * (2.0 * PI * (hr - 15.0) / 24.0).cos();
            let rh = (rh_day - 2.0 * (temp - t_mean)).clamp(10.0, 100.0);
            // 시간 바람
            x_h = 0.7 * x_h + (1.0f64 - 0.49).sqrt() * rng.normal();
            let sd = c.hourly_wind_sd.max(0.0);
            let diurnal = 1.0 + 0.25 * (2.0 * PI * (hr - 14.0) / 24.0).cos();
            let w10 = w_day * diurnal * (sd * x_h - 0.5 * sd * sd).exp();
            let mut wind = w10 * hfac;
            let mut gust = wind * c.gust_factor * (0.08 * rng.normal()).exp();
            let mut rain = if in_rain { rng.exp(mean_rate.max(0.05)) } else { 0.0 };
            let mut is_ty = false;
            for (s, peak, dur) in typhoons.iter() {
                let tt = idx as f64 - *s as f64;
                if tt >= 0.0 && tt <= *dur {
                    let shape = (PI * tt / dur).sin().max(0.0).powf(0.8);
                    let g = peak * hfac * shape;
                    if g > gust {
                        gust = g;
                        wind = wind.max(g / c.gust_factor);
                    }
                    if shape > 0.3 {
                        // 태풍철(6–10월)은 호우, 겨울 · 봄 저기압은 약한 비
                        let wet = if (5..=9).contains(&m) { 5.0 + 20.0 * shape * rng.uniform() } else { 2.0 * shape * rng.uniform() };
                        rain = rain.max(wet);
                    }
                    is_ty = shape > 0.2;
                }
            }
            let flurry_now = flurry && h >= fl_start && h < fl_start + fl_len;
            let snow = (rain > 0.0 && temp <= c.snow_temp_c) || flurry_now;
            if rain > 0.0 {
                last_wet_h = Some(idx);
            }
            let fog_now = fog && h >= fog_start && h < fog_start + fog_len && rain < 1.0;
            let mut vis: f64 = 20_000.0;
            if rain > 0.0 {
                vis = if snow { 600.0 + 1500.0 * rng.uniform() } else if rain > 10.0 { 1200.0 } else { 4000.0 + 4000.0 * rng.uniform() };
            }
            if fog_now {
                vis = vis.min((60.0f64.ln() + (900.0f64.ln() - 60.0f64.ln()) * rng.uniform()).exp());
            }
            let recent_wet = last_wet_h.map(|w| idx - w <= 12).unwrap_or(false);
            let icy = temp <= 0.5 && (recent_wet || fog_now);
            let thunder_km = if thunder && h >= th_start && h < th_start + th_len { Some(15.0 * rng.uniform()) } else { None };
            out.push(HourWeather {
                day: d,
                hour: h as u8,
                doy: doy % 365,
                temp_c: temp,
                humidity_pct: rh,
                wind_mean_mps: wind,
                gust_mps: gust,
                wind_dir_deg: (dir_day + 25.0 * rng.normal()).rem_euclid(360.0),
                rain_mm: rain,
                snow,
                icy,
                thunder_km,
                visibility_m: vis,
                fog: vis < 1000.0 && fog_now,
                typhoon: is_ty,
                feels_like_c: feels_like_c(temp, rh, w10, is_summer(m)),
            });
        }
    }
    out
}

/// 예보: 실제 미래 값에 앞선 시간만큼 커지는 오차를 얹는다 (같은 시점 · 같은 선행 시간이면 같은 값)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Forecaster {
    pub seed: u64,
    /// 상대 오차 표준편차 = base + per_hour × 선행 시간
    pub base_sd: f64,
    pub per_hour_sd: f64,
}

impl Default for Forecaster {
    fn default() -> Self {
        Self { seed: 1, base_sd: 0.08, per_hour_sd: 0.012 }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Outlook {
    pub hours: u32,
    pub max_gust_mps: f64,
    pub hour_of_max: u32,
    pub thunder: bool,
    pub min_visibility_m: f64,
    pub max_feels_like_c: f64,
    pub min_temp_c: f64,
    pub icy: bool,
    pub typhoon: bool,
}

impl Forecaster {
    /// `from` 시간 색인 다음 `hours` 시간의 예보 요약
    /// 예보 오차는 시간마다 따로 뽑지 않고 예보 하나에 하나를 쓴다 (시간마다 따로 뽑으면 최대값이 위로 치우친다).
    /// 오차 크기는 최대 순간풍속이 나오는 선행 시간으로 정한다.
    pub fn outlook(&self, w: &[HourWeather], from: usize, hours: u32) -> Outlook {
        let mut o = Outlook { hours, min_visibility_m: f64::MAX, min_temp_c: f64::MAX, max_feels_like_c: f64::MIN, ..Default::default() };
        let mut true_max = 0.0;
        for lead in 1..=hours as usize {
            let Some(x) = w.get(from + lead) else { break };
            if x.gust_mps > true_max {
                true_max = x.gust_mps;
                o.hour_of_max = lead as u32;
            }
            o.thunder |= x.thunder_km.is_some();
            o.min_visibility_m = o.min_visibility_m.min(x.visibility_m);
            o.max_feels_like_c = o.max_feels_like_c.max(x.feels_like_c);
            o.min_temp_c = o.min_temp_c.min(x.temp_c);
            o.icy |= x.icy;
            o.typhoon |= x.typhoon;
        }
        // 같은 예보 시각에는 같은 오차 (연속한 시각의 예보 오차는 서로 상관) — 6 시간 묶음
        let mut r = Rng::new(self.seed ^ (((from / 6) as u64) << 20) ^ hours as u64);
        let sd = self.base_sd + self.per_hour_sd * o.hour_of_max as f64;
        o.max_gust_mps = true_max * (sd * r.normal() - 0.5 * sd * sd).exp();
        o.max_feels_like_c += 3.0 * sd * r.normal();
        if o.min_visibility_m == f64::MAX {
            o.min_visibility_m = 20_000.0;
            o.min_temp_c = 0.0;
            o.max_feels_like_c = 0.0;
        }
        o
    }
}

/// 생성한 날씨의 월별 요약 (보정 비교용)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonthStats {
    pub month: usize,
    pub days: f64,
    pub temp_mean_c: f64,
    pub wind_mean_10m_mps: f64,
    pub precip_mm: f64,
    pub rain_days: f64,
    pub snow_days: f64,
    pub thunder_days: f64,
    pub fog_days: f64,
    /// 크레인 높이 순간풍속 20 m/s 이상 시간 (h)
    pub gust_over_20_h: f64,
    /// 폭풍일: 일 최대 10분 평균 풍속(10 m) 13.9 m/s 이상 (기상청 정의)
    #[serde(default)]
    pub storm_days: f64,
    /// 한파일: 일 최저기온 −12 °C 이하
    #[serde(default)]
    pub coldwave_days: f64,
    /// 일 최대 순간풍속(10 m) 중 가장 큰 값
    #[serde(default)]
    pub max_gust_10m_mps: f64,
}

/// 시간별 날씨를 월별 평균 · 일수로 묶는다 (풍속은 높이 보정을 되돌린 10 m 값)
pub fn monthly_stats(w: &[HourWeather], height_m: f64, alpha: f64) -> Vec<MonthStats> {
    monthly_stats_with(w, height_m, alpha, 1.5)
}

/// `gust_factor` 로 순간풍속에서 10분 평균을 되돌린다 (폭풍일 판정)
pub fn monthly_stats_with(w: &[HourWeather], height_m: f64, alpha: f64, gust_factor: f64) -> Vec<MonthStats> {
    let hfac = (height_m.max(2.0) / 10.0).powf(alpha.clamp(0.0, 0.4));
    let mut s: Vec<MonthStats> = (0..12).map(|m| MonthStats { month: m + 1, ..Default::default() }).collect();
    for day in w.chunks(24) {
        let m = month_of(day[0].doy);
        let x = &mut s[m];
        x.days += 1.0;
        x.temp_mean_c += day.iter().map(|h| h.temp_c).sum::<f64>() / 24.0;
        x.wind_mean_10m_mps += day.iter().map(|h| h.wind_mean_mps).sum::<f64>() / 24.0 / hfac;
        let p: f64 = day.iter().map(|h| h.rain_mm).sum();
        x.precip_mm += p;
        x.rain_days += if p >= 0.1 { 1.0 } else { 0.0 };
        x.snow_days += if day.iter().any(|h| h.snow) { 1.0 } else { 0.0 };
        x.thunder_days += if day.iter().any(|h| h.thunder_km.is_some()) { 1.0 } else { 0.0 };
        x.fog_days += if day.iter().any(|h| h.fog) { 1.0 } else { 0.0 };
        x.gust_over_20_h += day.iter().filter(|h| h.gust_mps >= 20.0).count() as f64;
        let g10 = day.iter().map(|h| h.gust_mps / hfac).fold(0.0, f64::max);
        x.storm_days += if g10 / gust_factor.max(1.0) >= 13.9 { 1.0 } else { 0.0 };
        x.coldwave_days += if day.iter().map(|h| h.temp_c).fold(f64::MAX, f64::min) <= -12.0 { 1.0 } else { 0.0 };
        x.max_gust_10m_mps = x.max_gust_10m_mps.max(g10);
    }
    for x in s.iter_mut() {
        if x.days > 0.0 {
            let n = x.days;
            x.temp_mean_c /= n;
            x.wind_mean_10m_mps /= n;
            // 일수 · 강수량은 그 달 길이로 환산
            let dim = DAYS_IN_MONTH[x.month - 1] as f64;
            let k = dim / n;
            x.precip_mm *= k;
            x.rain_days *= k;
            x.snow_days *= k;
            x.thunder_days *= k;
            x.fog_days *= k;
            x.gust_over_20_h *= k;
            x.storm_days *= k;
            x.coldwave_days *= k;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn flat_climate() -> Climate {
        let m = MonthClimate {
            temp_mean_c: 15.0,
            temp_range_c: 8.0,
            wind_mean_mps: 3.5,
            precip_mm: 100.0,
            rain_days: 10.0,
            snow_days: 0.0,
            thunder_days: 2.0,
            fog_days: 3.0,
            humidity_pct: 70.0,
        };
        Climate {
            id: "flat".into(),
            name: "flat".into(),
            source: "test".into(),
            months: vec![m; 12],
            gust_factor: 1.5,
            weibull_k: 2.0,
            hourly_wind_sd: 0.25,
            alpha: 0.14,
            temp_anomaly_sd_c: 2.5,
            temp_anomaly_sd_winter_c: 2.5,
            snow_temp_c: 1.0,
            snow_flurry_prob: 0.0,
            typhoon: None,
        }
    }

    #[test]
    fn calendar_helpers() {
        assert_eq!(month_of(0), 0);
        assert_eq!(month_of(31), 1);
        assert_eq!(month_of(364), 11);
        assert_eq!(doy_of(3, 1), 59);
        assert_eq!(date_label(doy_of(7, 15)), "07-15");
    }

    #[test]
    fn generated_weather_matches_normals() {
        let c = flat_climate();
        c.validate().unwrap();
        let w = generate(&c, &WeatherOptions { start_doy: 0, days: 365 * 3, height_m: 10.0, seed: 3 });
        assert_eq!(w.len(), 365 * 3 * 24);
        let st = monthly_stats(&w, 10.0, c.alpha);
        let t = st.iter().map(|s| s.temp_mean_c).sum::<f64>() / 12.0;
        let v = st.iter().map(|s| s.wind_mean_10m_mps).sum::<f64>() / 12.0;
        let rd = st.iter().map(|s| s.rain_days).sum::<f64>() / 12.0;
        let th = st.iter().map(|s| s.thunder_days).sum::<f64>() / 12.0;
        assert!((t - 15.0).abs() < 0.6, "temp {t}");
        assert!((v - 3.5).abs() < 0.35, "wind {v}");
        assert!((rd - 10.0).abs() < 1.5, "rain days {rd}");
        assert!((th - 2.0).abs() < 0.7, "thunder days {th}");
        // 돌풍은 평균보다 크다
        assert!(w.iter().all(|h| h.gust_mps >= 0.0));
        let g = w.iter().map(|h| h.gust_mps).sum::<f64>() / w.len() as f64;
        assert!(g > 1.3 * v && g < 1.8 * v, "{g} vs {v}");
    }

    #[test]
    fn same_seed_same_weather_and_typhoon_raises_gusts() {
        let mut c = flat_climate();
        let o = WeatherOptions { start_doy: doy_of(8, 1), days: 60, height_m: 25.0, seed: 11 };
        let a = generate(&c, &o);
        let b = generate(&c, &o);
        assert_eq!(a.len(), b.len());
        assert!(a.iter().zip(b.iter()).all(|(x, y)| x.gust_mps == y.gust_mps));
        c.typhoon = Some(TyphoonClimate {
            per_year: 50.0,
            threshold_mps: 25.0,
            excess_scale_mps: 5.0,
            month_weights: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0],
            duration_h: 18.0,
            source: "test".into(),
        });
        let t = generate(&c, &o);
        let max_a = a.iter().map(|h| h.gust_mps).fold(0.0, f64::max);
        let max_t = t.iter().map(|h| h.gust_mps).fold(0.0, f64::max);
        assert!(t.iter().any(|h| h.typhoon));
        assert!(max_t > max_a + 5.0, "{max_t} vs {max_a}");
    }

    #[test]
    fn feels_like_follows_kma_formulas() {
        // 여름: 기온 33 °C, 습도 60% → 체감온도는 기온 근처에서 습도에 따라 오른다
        let a = feels_like_c(33.0, 60.0, 1.0, true);
        let b = feels_like_c(33.0, 80.0, 1.0, true);
        assert!(a > 31.0 && a < 36.0, "{a}");
        assert!(b > a);
        // 겨울: 바람이 불면 더 춥게
        assert!(feels_like_c(-5.0, 50.0, 8.0, false) < -10.0);
        assert_eq!(feels_like_c(15.0, 50.0, 8.0, false), 15.0);
    }

    #[test]
    fn forecast_is_repeatable_and_tracks_truth() {
        let c = flat_climate();
        let w = generate(&c, &WeatherOptions { start_doy: 0, days: 5, height_m: 20.0, seed: 5 });
        let f = Forecaster::default();
        let a = f.outlook(&w, 10, 6);
        let b = f.outlook(&w, 10, 6);
        assert_eq!(a.max_gust_mps, b.max_gust_mps);
        let truth = w[11..=16].iter().map(|h| h.gust_mps).fold(0.0, f64::max);
        assert!((a.max_gust_mps / truth - 1.0).abs() < 0.5);
    }
}
