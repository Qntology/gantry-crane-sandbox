//! 날씨 API — Open-Meteo 에서 해안 위치의 실제 날씨를 가져와 저장하고, 시뮬레이션이 쓰는 꼴로 바꾼다.
//!
//! - ERA5 재분석 보관(`archive-api`, 모델 `era5_seamless` = ERA5-Land 0.1° + ERA5 0.25°): 일별 30년 · 시간별 1–2년
//! - CAMS 대기질(`air-quality-api`): 시간별 SO₂ (µg/m³) → ISO 9223 의 SO₂ 침착량
//! - 해양(`marine-api`): 시간별 유의 파고 (m) → 쇄파 · 비래염분
//!
//! 요청은 시스템 `curl` 로 보낸다 (키 없음, 무료 비상업 사용, 출처 표기 CC BY 4.0). 무료 한도는 분당 600 · 시간당 5,000 ·
//! 하루 10,000 회이고, 변수 10개 · 2주를 넘는 요청은 그만큼 여러 번으로 센다 (open-meteo.com/en/pricing).
//! 받은 응답은 그대로 `cache_dir` 에 저장해 두고, 같은 요청은 다시 보내지 않는다.
//!
//! 쓰는 길은 둘이다.
//! 1. 재생: 시간별 실제 날씨 → [`HourWeather`] (시정 · 낙뢰는 ERA5 에 없어 습도 · 강수 규칙으로 정한다).
//! 2. 기후: 일별 · 시간별 자료 → [`SiteStats`] (월 평년값, 문턱별 일수, 바람장미, 젖음 시간, 폭풍 봉우리) →
//!    [`Era5Adjust`] 로 관측소 기준에 맞춘 뒤 [`climate_from_stats`] 가 합성 날씨용 [`Climate`] 를 만든다.
//!    보정 계수는 기상청 6개 지점과 비교해 정하고, 지점 하나씩 빼고 맞춘 오차를 함께 남긴다 (`fieldcal`).

use crate::rng::Rng;
use crate::weather::{self, Climate, HourWeather, MonthClimate, TyphoonClimate, DAYS_IN_MONTH};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const ARCHIVE_URL: &str = "https://archive-api.open-meteo.com/v1/archive";
pub const AIR_URL: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";
pub const MARINE_URL: &str = "https://marine-api.open-meteo.com/v1/marine";
pub const MODEL: &str = "era5_seamless";
pub const DAILY_VARS: &str = "temperature_2m_mean,temperature_2m_max,temperature_2m_min,precipitation_sum,snowfall_sum,wind_speed_10m_mean,wind_speed_10m_max,wind_gusts_10m_max,relative_humidity_2m_mean,wind_direction_10m_dominant";
pub const HOURLY_VARS: &str = "temperature_2m,relative_humidity_2m,dew_point_2m,precipitation,snowfall,wind_speed_10m,wind_direction_10m,wind_gusts_10m";
pub const ATTRIBUTION: &str = "Weather data by Open-Meteo.com (ERA5 / ERA5-Land, CAMS, marine) — CC BY 4.0, Copernicus Climate Change Service";

/// 문턱별 일수 격자 — 관측소 정의에 맞는 문턱을 보정으로 고른다
pub const RAIN_MM: [f64; 7] = [0.1, 0.2, 0.5, 1.0, 1.5, 2.0, 3.0];
pub const SNOW_CM: [f64; 5] = [0.01, 0.1, 0.3, 0.6, 1.0];
pub const FOG_RH: [f64; 7] = [93.0, 94.0, 95.0, 96.0, 97.0, 98.0, 99.0];
pub const THUNDER_MM_H: [f64; 7] = [1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0];
/// 안개 판정의 풍속 상한 (10 m, m/s) · 비 상한 (mm/h)
pub const FOG_WIND_MAX: f64 = 2.5;
pub const FOG_RAIN_MAX: f64 = 0.5;
/// 뇌전 판정의 최저 기온 (°C) — 대류성 강수
pub const THUNDER_MIN_TEMP: f64 = 12.0;
/// 일 최대 풍속 분포 칸 (m/s)
pub const WIND_BIN: f64 = 0.5;
pub const WIND_BINS: usize = 80;
/// 방위 8칸(북 · 북동 · … · 북서), 풍속 1 m/s 칸 25개
pub const SECTORS: usize = 8;
pub const SPEED_BINS: usize = 25;
/// 폭풍 봉우리: 일 최대 순간풍속이 이 값 이상인 날을 3일 간격으로 묶어 봉우리만 남긴다
pub const PEAK_MIN_GUST: f64 = 15.0;
pub const PEAK_SEPARATION_D: usize = 3;
/// 젖음 시간 (ISO 9223): RH > 80 % 이고 기온 > 0 °C
pub const TOW_RH: f64 = 80.0;

// ---------------------------------------------------------------- 요청 · 저장

/// Open-Meteo 요청기 (응답 캐시 포함)
#[derive(Debug, Clone)]
pub struct Fetcher {
    pub cache_dir: PathBuf,
    /// true 면 캐시만 쓰고 인터넷에 요청하지 않는다
    pub offline: bool,
    pub timeout_s: u32,
}

impl Fetcher {
    pub fn new(cache_dir: &Path) -> Self {
        Self { cache_dir: cache_dir.to_path_buf(), offline: false, timeout_s: 300 }
    }

    pub fn cache_path(&self, kind: &str, lat: f64, lon: f64, start: &str, end: &str) -> PathBuf {
        self.cache_dir.join(format!("{kind}_{lat:.3}_{lon:.3}_{start}_{end}.json"))
    }

    /// 캐시에 있으면 읽고, 없으면 요청해 저장한다
    pub fn get(&self, kind: &str, base: &str, lat: f64, lon: f64, start: &str, end: &str, params: &str) -> Result<serde_json::Value, String> {
        let path = self.cache_path(kind, lat, lon, start, end);
        if let Ok(text) = std::fs::read_to_string(&path) {
            return parse_response(&text).map_err(|e| format!("{}: {e}", path.display()));
        }
        if self.offline {
            return Err(format!("캐시에 없음 (오프라인): {}", path.display()));
        }
        let url = format!("{base}?latitude={lat}&longitude={lon}&start_date={start}&end_date={end}&{params}&timezone=auto");
        let text = http_get(&url, self.timeout_s)?;
        let v = parse_response(&text)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, text.as_bytes()).map_err(|e| format!("{}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(v)
    }

    /// 일별 ERA5 (`from_year` 1월 1일 – `to_year` 12월 31일). 10년씩 나눠 요청한다 (분당 한도)
    pub fn daily(&self, lat: f64, lon: f64, from_year: i32, to_year: i32) -> Result<Daily, String> {
        if to_year < from_year {
            return Err("to_year < from_year".into());
        }
        let mut out: Option<Daily> = None;
        let mut a = from_year;
        while a <= to_year {
            let b = (a + 9).min(to_year);
            let (s, e) = (format!("{a}-01-01"), format!("{b}-12-31"));
            let v = self.get("daily-era5", ARCHIVE_URL, lat, lon, &s, &e, &format!("daily={DAILY_VARS}&models={MODEL}&wind_speed_unit=ms"))?;
            let d = Daily::from_json(&v)?;
            match out.as_mut() {
                None => out = Some(d),
                Some(o) => o.append(d),
            }
            a = b + 1;
        }
        out.ok_or_else(|| "자료 없음".into())
    }

    /// 시간별 ERA5 (날짜 "YYYY-MM-DD")
    pub fn hourly(&self, lat: f64, lon: f64, start: &str, end: &str) -> Result<Hourly, String> {
        let v = self.get("hourly-era5", ARCHIVE_URL, lat, lon, start, end, &format!("hourly={HOURLY_VARS}&models={MODEL}&wind_speed_unit=ms"))?;
        Hourly::from_json(&v)
    }

    /// 시간별 SO₂ (µg/m³, CAMS) — (시각, 값)
    pub fn so2(&self, lat: f64, lon: f64, start: &str, end: &str) -> Result<Vec<(String, f64)>, String> {
        let v = self.get("air", AIR_URL, lat, lon, start, end, "hourly=sulphur_dioxide")?;
        Ok(pairs(&v, "hourly", "sulphur_dioxide"))
    }

    /// 시간별 유의 파고 (m)
    pub fn waves(&self, lat: f64, lon: f64, start: &str, end: &str) -> Result<Vec<(String, f64)>, String> {
        let v = self.get("marine", MARINE_URL, lat, lon, start, end, "hourly=wave_height")?;
        Ok(pairs(&v, "hourly", "wave_height"))
    }
}

fn http_get(url: &str, timeout_s: u32) -> Result<String, String> {
    let out = std::process::Command::new("curl")
        .args(["-sS", "-L", "--max-time", &timeout_s.to_string(), url])
        .output()
        .map_err(|e| format!("curl 을 실행하지 못했습니다 ({e}) — 날씨 조회에는 curl 이 필요합니다"))?;
    if !out.status.success() {
        return Err(format!("날씨 요청 실패: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("응답이 UTF-8 이 아닙니다: {e}"))
}

fn parse_response(text: &str) -> Result<serde_json::Value, String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("응답 JSON 해석 실패: {e}"))?;
    if v.get("error").and_then(|e| e.as_bool()).unwrap_or(false) {
        return Err(format!("Open-Meteo 오류: {}", v.get("reason").and_then(|r| r.as_str()).unwrap_or("?")));
    }
    Ok(v)
}

fn num_series(v: &serde_json::Value, section: &str, key: &str) -> Vec<f64> {
    v.get(section)
        .and_then(|s| s.get(key))
        .and_then(|a| a.as_array())
        .map(|a| a.iter().map(|x| x.as_f64().unwrap_or(f64::NAN)).collect())
        .unwrap_or_default()
}

fn str_series(v: &serde_json::Value, section: &str) -> Vec<String> {
    v.get(section)
        .and_then(|s| s.get("time"))
        .and_then(|a| a.as_array())
        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
        .unwrap_or_default()
}

fn pairs(v: &serde_json::Value, section: &str, key: &str) -> Vec<(String, f64)> {
    let t = str_series(v, section);
    let y = num_series(v, section, key);
    t.into_iter().zip(y).filter(|(_, y)| y.is_finite()).collect()
}

/// 빈 값(NaN)을 앞뒤 값 사이 직선으로 메운다. 전부 비면 0
pub fn fill_gaps(v: &mut [f64]) -> usize {
    let n = v.len();
    let mut filled = 0;
    let mut last: Option<usize> = None;
    let mut i = 0;
    while i < n {
        if v[i].is_finite() {
            last = Some(i);
            i += 1;
            continue;
        }
        let mut j = i;
        while j < n && !v[j].is_finite() {
            j += 1;
        }
        for k in i..j {
            v[k] = match (last, (j < n).then_some(j)) {
                (Some(a), Some(b)) => v[a] + (v[b] - v[a]) * (k - a) as f64 / (b - a) as f64,
                (Some(a), None) => v[a],
                (None, Some(b)) => v[b],
                (None, None) => 0.0,
            };
            filled += 1;
        }
        i = j;
    }
    filled
}

/// "YYYY-MM-DD..." → (년, 월, 일)
pub fn ymd(s: &str) -> Option<(i32, u32, u32)> {
    Some((s.get(0..4)?.parse().ok()?, s.get(5..7)?.parse().ok()?, s.get(8..10)?.parse().ok()?))
}

/// 평년 기준 연중 날짜 (2월 29일은 None — 시뮬레이션은 365일 해를 쓴다)
pub fn doy_noleap(month: u32, day: u32) -> Option<u32> {
    if month == 2 && day == 29 {
        return None;
    }
    Some(weather::doy_of(month, day))
}

// ---------------------------------------------------------------- 자료 꼴

/// 일별 자료
#[derive(Debug, Clone, Default)]
pub struct Daily {
    pub lat: f64,
    pub lon: f64,
    pub elevation_m: f64,
    pub timezone: String,
    pub date: Vec<(i32, u32, u32)>,
    pub t_mean: Vec<f64>,
    pub t_max: Vec<f64>,
    pub t_min: Vec<f64>,
    pub precip: Vec<f64>,
    pub snow_cm: Vec<f64>,
    pub wind_mean: Vec<f64>,
    pub wind_max: Vec<f64>,
    pub gust_max: Vec<f64>,
    pub rh_mean: Vec<f64>,
    pub dir_dom: Vec<f64>,
    /// 메운 빈 값 수
    pub gaps: usize,
}

impl Daily {
    pub fn from_json(v: &serde_json::Value) -> Result<Self, String> {
        let t = str_series(v, "daily");
        if t.is_empty() {
            return Err("일별 자료(daily.time)가 없습니다".into());
        }
        let mut d = Daily {
            lat: v.get("latitude").and_then(|x| x.as_f64()).unwrap_or(f64::NAN),
            lon: v.get("longitude").and_then(|x| x.as_f64()).unwrap_or(f64::NAN),
            elevation_m: v.get("elevation").and_then(|x| x.as_f64()).unwrap_or(0.0),
            timezone: v.get("timezone").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            date: t.iter().map(|s| ymd(s).unwrap_or((0, 1, 1))).collect(),
            t_mean: num_series(v, "daily", "temperature_2m_mean"),
            t_max: num_series(v, "daily", "temperature_2m_max"),
            t_min: num_series(v, "daily", "temperature_2m_min"),
            precip: num_series(v, "daily", "precipitation_sum"),
            snow_cm: num_series(v, "daily", "snowfall_sum"),
            wind_mean: num_series(v, "daily", "wind_speed_10m_mean"),
            wind_max: num_series(v, "daily", "wind_speed_10m_max"),
            gust_max: num_series(v, "daily", "wind_gusts_10m_max"),
            rh_mean: num_series(v, "daily", "relative_humidity_2m_mean"),
            dir_dom: num_series(v, "daily", "wind_direction_10m_dominant"),
            gaps: 0,
        };
        let n = d.date.len();
        let mut gaps = 0;
        for s in [&mut d.t_mean, &mut d.t_max, &mut d.t_min, &mut d.precip, &mut d.snow_cm, &mut d.wind_mean, &mut d.wind_max, &mut d.gust_max, &mut d.rh_mean, &mut d.dir_dom] {
            if s.len() != n {
                s.resize(n, f64::NAN);
            }
            gaps += fill_gaps(s);
        }
        d.gaps = gaps;
        Ok(d)
    }

    fn append(&mut self, o: Daily) {
        self.date.extend(o.date);
        self.t_mean.extend(o.t_mean);
        self.t_max.extend(o.t_max);
        self.t_min.extend(o.t_min);
        self.precip.extend(o.precip);
        self.snow_cm.extend(o.snow_cm);
        self.wind_mean.extend(o.wind_mean);
        self.wind_max.extend(o.wind_max);
        self.gust_max.extend(o.gust_max);
        self.rh_mean.extend(o.rh_mean);
        self.dir_dom.extend(o.dir_dom);
        self.gaps += o.gaps;
    }

    pub fn len(&self) -> usize {
        self.date.len()
    }

    pub fn is_empty(&self) -> bool {
        self.date.is_empty()
    }

    pub fn years(&self) -> (i32, i32) {
        (self.date.first().map(|d| d.0).unwrap_or(0), self.date.last().map(|d| d.0).unwrap_or(0))
    }
}

/// 시간별 자료
#[derive(Debug, Clone, Default)]
pub struct Hourly {
    pub lat: f64,
    pub lon: f64,
    pub elevation_m: f64,
    pub timezone: String,
    /// (년, 월, 일, 시)
    pub time: Vec<(i32, u32, u32, u8)>,
    pub temp_c: Vec<f64>,
    pub rh_pct: Vec<f64>,
    pub dew_c: Vec<f64>,
    pub precip_mm: Vec<f64>,
    pub snow_cm: Vec<f64>,
    pub wind_mps: Vec<f64>,
    pub wind_from_deg: Vec<f64>,
    pub gust_mps: Vec<f64>,
    /// 시간 맞춘 SO₂ (µg/m³) · 파고 (m) — 없으면 비어 있다
    pub so2: Vec<f64>,
    pub wave_m: Vec<f64>,
    pub gaps: usize,
}

impl Hourly {
    pub fn from_json(v: &serde_json::Value) -> Result<Self, String> {
        let t = str_series(v, "hourly");
        if t.is_empty() {
            return Err("시간별 자료(hourly.time)가 없습니다".into());
        }
        let time: Vec<(i32, u32, u32, u8)> = t
            .iter()
            .map(|s| {
                let (y, m, d) = ymd(s).unwrap_or((0, 1, 1));
                let h: u8 = s.get(11..13).and_then(|x| x.parse().ok()).unwrap_or(0);
                (y, m, d, h)
            })
            .collect();
        let mut h = Hourly {
            lat: v.get("latitude").and_then(|x| x.as_f64()).unwrap_or(f64::NAN),
            lon: v.get("longitude").and_then(|x| x.as_f64()).unwrap_or(f64::NAN),
            elevation_m: v.get("elevation").and_then(|x| x.as_f64()).unwrap_or(0.0),
            timezone: v.get("timezone").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            time,
            temp_c: num_series(v, "hourly", "temperature_2m"),
            rh_pct: num_series(v, "hourly", "relative_humidity_2m"),
            dew_c: num_series(v, "hourly", "dew_point_2m"),
            precip_mm: num_series(v, "hourly", "precipitation"),
            snow_cm: num_series(v, "hourly", "snowfall"),
            wind_mps: num_series(v, "hourly", "wind_speed_10m"),
            wind_from_deg: num_series(v, "hourly", "wind_direction_10m"),
            gust_mps: num_series(v, "hourly", "wind_gusts_10m"),
            so2: Vec::new(),
            wave_m: Vec::new(),
            gaps: 0,
        };
        let n = h.time.len();
        let mut gaps = 0;
        for s in [&mut h.temp_c, &mut h.rh_pct, &mut h.dew_c, &mut h.precip_mm, &mut h.snow_cm, &mut h.wind_mps, &mut h.wind_from_deg, &mut h.gust_mps] {
            if s.len() != n {
                s.resize(n, f64::NAN);
            }
            gaps += fill_gaps(s);
        }
        h.gaps = gaps;
        Ok(h)
    }

    pub fn len(&self) -> usize {
        self.time.len()
    }

    pub fn is_empty(&self) -> bool {
        self.time.is_empty()
    }

    fn key(t: &(i32, u32, u32, u8)) -> String {
        format!("{:04}-{:02}-{:02}T{:02}:00", t.0, t.1, t.2, t.3)
    }

    /// 다른 요청(SO₂ · 파고)의 (시각, 값)을 이 자료의 시간에 맞춘다. 빈 시간은 메운다
    pub fn align(&self, p: &[(String, f64)]) -> Vec<f64> {
        if p.is_empty() {
            return Vec::new();
        }
        let map: std::collections::HashMap<&str, f64> = p.iter().map(|(t, v)| (t.as_str(), *v)).collect();
        let mut out: Vec<f64> = self.time.iter().map(|t| map.get(Self::key(t).as_str()).cloned().unwrap_or(f64::NAN)).collect();
        if out.iter().all(|v| !v.is_finite()) {
            return Vec::new();
        }
        fill_gaps(&mut out);
        out
    }

    /// 첫 시각에서 `date` ("YYYY-MM-DD") 0시의 색인
    pub fn index_of(&self, date: &str) -> Option<usize> {
        let (y, m, d) = ymd(date)?;
        self.time.iter().position(|t| t.0 == y && t.1 == m && t.2 == d && t.3 == 0)
    }
}

// ---------------------------------------------------------------- 관측소 맞춤 보정

/// ERA5 를 관측소 기준(기상청 지상 관측)에 맞추는 계수. 기본값은 보정 전(그대로)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Era5Adjust {
    /// 기온 · 상대습도 더하기 (관측 − ERA5)
    pub temp_offset_c: f64,
    pub rh_offset_pct: f64,
    /// 일교차 배수 (해안 격자는 일교차가 작다)
    pub range_scale: f64,
    /// 10 m 평균 풍속 배수 · 순간풍속 배수
    pub wind_scale: f64,
    pub gust_scale: f64,
    /// 강수량 배수
    pub precip_scale: f64,
    /// 강수일 · 눈 일수 문턱 (ERA5 일 합계) — 관측 강수일(0.1 mm) · 눈 일수에 맞춘 값
    pub rain_day_mm: f64,
    pub snow_day_cm: f64,
    /// 안개 판정 습도 문턱 (%) · 뇌전 판정 시간 강수 문턱 (mm/h)
    pub fog_rh_pct: f64,
    pub thunder_mm_h: f64,
    pub source: String,
}

impl Default for Era5Adjust {
    fn default() -> Self {
        Self {
            temp_offset_c: 0.0,
            rh_offset_pct: 0.0,
            range_scale: 1.0,
            wind_scale: 1.0,
            gust_scale: 1.0,
            precip_scale: 1.0,
            rain_day_mm: 0.1,
            snow_day_cm: 0.01,
            fog_rh_pct: 97.0,
            thunder_mm_h: 4.0,
            source: "보정 전 (ERA5 그대로, 관측 정의 문턱)".into(),
        }
    }
}

// ---------------------------------------------------------------- 통계 (저장용)

/// 한 달의 ERA5 통계 (해마다 평균)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Era5Month {
    pub temp_mean_c: f64,
    pub temp_max_c: f64,
    pub temp_min_c: f64,
    pub precip_mm: f64,
    pub rh_pct: f64,
    pub wind_mean_mps: f64,
    /// `RAIN_MM` 문턱별 강수일수 · `SNOW_CM` 문턱별 눈 일수
    pub rain_days: Vec<f64>,
    pub snow_days: Vec<f64>,
    /// 일 최저 −12 °C 이하 일수 (한파일)
    pub coldwave_days: f64,
    /// 일 최대(시간) 10 m 풍속 분포 (일/년, `WIND_BIN` 칸)
    pub wind_max_hist: Vec<f64>,
    /// 일 주풍향 8방위 빈도 (합 1)
    pub dir_freq: Vec<f64>,
    // --- 시간별 자료에서 (없으면 0 · 비어 있음)
    /// 이 달 시간 수 (해마다 평균)
    #[serde(default)]
    pub hours: f64,
    /// 젖음 시간 (RH > 80 %, T > 0 °C, h/년) · RH ≥ 90 · 95 % 시간
    #[serde(default)]
    pub tow_h: f64,
    #[serde(default)]
    pub rh90_h: f64,
    #[serde(default)]
    pub rh95_h: f64,
    /// 비 오는 시간 (≥ 0.1 mm/h)
    #[serde(default)]
    pub precip_h: f64,
    /// `FOG_RH` 문턱별 안개 일수 · `THUNDER_MM_H` 문턱별 뇌전 일수
    #[serde(default)]
    pub fog_days: Vec<f64>,
    #[serde(default)]
    pub thunder_days: Vec<f64>,
    /// 방위(8) × 풍속(1 m/s 칸 25) 시간 수 (h/년)
    #[serde(default)]
    pub sector_speed_h: Vec<Vec<f64>>,
    /// SO₂ 월평균 (µg/m³) · 파고 평균 · 90 % 값 (m)
    #[serde(default)]
    pub so2_ug_m3: Option<f64>,
    #[serde(default)]
    pub wave_mean_m: Option<f64>,
    #[serde(default)]
    pub wave_p90_m: Option<f64>,
}

/// 한 위치의 ERA5 통계 묶음 — `data/climate/ports.json` 에 저장한다
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SiteStats {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    /// 실제로 쓰인 격자점
    pub grid_lat: f64,
    pub grid_lon: f64,
    pub elevation_m: f64,
    pub timezone: String,
    pub model: String,
    /// 일별 자료 연도 범위 · 년 수
    pub daily_from: i32,
    pub daily_to: i32,
    pub years: f64,
    /// 시간별 자료 기간
    pub hourly_from: String,
    pub hourly_to: String,
    pub hourly_years: f64,
    pub months: Vec<Era5Month>,
    /// 일평균 풍속 와이블 형상 계수 (적률법)
    pub weibull_k: f64,
    /// 일평균 기온 이상치 표준편차 (7월 · 1월 중심 ±30일)
    pub temp_sd_summer_c: f64,
    pub temp_sd_winter_c: f64,
    /// 폭풍 봉우리 (월 0–11, 일 최대 순간풍속 m/s) — 3일 간격으로 묶은 값
    pub storm_peaks: Vec<(u8, f64)>,
    pub gaps: usize,
    pub source: String,
}

fn sector_of(deg: f64) -> usize {
    (((deg.rem_euclid(360.0) + 22.5) / 45.0).floor() as usize) % SECTORS
}

/// 일별(필수) · 시간별(선택) 자료에서 통계를 만든다
pub fn build_stats(id: &str, lat: f64, lon: f64, d: &Daily, h: Option<&Hourly>) -> SiteStats {
    let (y0, y1) = d.years();
    let years = ((y1 - y0 + 1) as f64).max(1.0);
    let mut months: Vec<Era5Month> = (0..12)
        .map(|_| Era5Month {
            rain_days: vec![0.0; RAIN_MM.len()],
            snow_days: vec![0.0; SNOW_CM.len()],
            wind_max_hist: vec![0.0; WIND_BINS],
            dir_freq: vec![0.0; SECTORS],
            ..Default::default()
        })
        .collect();
    let mut n_days = [0.0f64; 12];
    let mut sums = [[0.0f64; 6]; 12];
    for i in 0..d.len() {
        let (_, mo, _) = d.date[i];
        let m = (mo.clamp(1, 12) - 1) as usize;
        n_days[m] += 1.0;
        let s = &mut sums[m];
        s[0] += d.t_mean[i];
        s[1] += d.t_max[i];
        s[2] += d.t_min[i];
        s[3] += d.precip[i];
        s[4] += d.rh_mean[i];
        s[5] += d.wind_mean[i];
        let mm = &mut months[m];
        for (k, th) in RAIN_MM.iter().enumerate() {
            if d.precip[i] >= *th {
                mm.rain_days[k] += 1.0;
            }
        }
        for (k, th) in SNOW_CM.iter().enumerate() {
            if d.snow_cm[i] >= *th {
                mm.snow_days[k] += 1.0;
            }
        }
        if d.t_min[i] <= -12.0 {
            mm.coldwave_days += 1.0;
        }
        let b = ((d.wind_max[i].max(0.0) / WIND_BIN) as usize).min(WIND_BINS - 1);
        mm.wind_max_hist[b] += 1.0;
        mm.dir_freq[sector_of(d.dir_dom[i])] += 1.0;
    }
    for m in 0..12 {
        let n = n_days[m].max(1.0);
        let mm = &mut months[m];
        mm.temp_mean_c = sums[m][0] / n;
        mm.temp_max_c = sums[m][1] / n;
        mm.temp_min_c = sums[m][2] / n;
        mm.precip_mm = sums[m][3] / years;
        mm.rh_pct = sums[m][4] / n;
        mm.wind_mean_mps = sums[m][5] / n;
        for x in mm.rain_days.iter_mut().chain(mm.snow_days.iter_mut()).chain(mm.wind_max_hist.iter_mut()) {
            *x /= years;
        }
        mm.coldwave_days /= years;
        let tot: f64 = mm.dir_freq.iter().sum::<f64>().max(1.0);
        for x in mm.dir_freq.iter_mut() {
            *x /= tot;
        }
    }
    // 와이블 형상 (일평균 풍속의 변동 계수에서, Justus 근사 k = (σ/μ)^-1.086)
    let w: Vec<f64> = d.wind_mean.iter().cloned().filter(|x| *x > 0.0).collect();
    let mu = w.iter().sum::<f64>() / w.len().max(1) as f64;
    let sd = (w.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / w.len().max(2) as f64).sqrt();
    let weibull_k = if mu > 0.0 && sd > 0.0 { (sd / mu).powf(-1.086).clamp(0.8, 4.0) } else { 2.0 };
    // 기온 이상치: 월평균 사이 보간 기후값을 뺀 일평균
    let clim_t: Vec<f64> = months.iter().map(|m| m.temp_mean_c).collect();
    let interp = |doy: f64| -> f64 {
        let mids: Vec<f64> = (0..12).map(|m| DAYS_IN_MONTH[..m].iter().sum::<u32>() as f64 + DAYS_IN_MONTH[m] as f64 / 2.0).collect();
        let x = doy.rem_euclid(365.0);
        let mut a = 11;
        for (m, md) in mids.iter().enumerate() {
            if x >= *md {
                a = m;
            }
        }
        let (i0, i1) = if x < mids[0] { (11, 0) } else { (a, (a + 1) % 12) };
        let x0 = if x < mids[0] { mids[11] - 365.0 } else { mids[i0] };
        let x1 = if i1 == 0 && i0 == 11 && x >= mids[0] { mids[0] + 365.0 } else { mids[i1] };
        let f = ((x - x0) / (x1 - x0)).clamp(0.0, 1.0);
        clim_t[i0] * (1.0 - f) + clim_t[i1] * f
    };
    let (mut ss, mut ns, mut sw, mut nw) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for i in 0..d.len() {
        let (_, mo, dd) = d.date[i];
        let Some(doy) = doy_noleap(mo, dd) else { continue };
        let a = d.t_mean[i] - interp(doy as f64);
        let dj = (doy as i32 - 15).rem_euclid(365).min((15 - doy as i32).rem_euclid(365));
        let dy = (doy as i32 - 196).abs();
        if dj <= 30 {
            sw += a * a;
            nw += 1.0;
        }
        if dy <= 30 {
            ss += a * a;
            ns += 1.0;
        }
    }
    // 폭풍 봉우리
    let mut peaks = Vec::new();
    let mut i = 0;
    while i < d.len() {
        if d.gust_max[i] >= PEAK_MIN_GUST {
            let mut j = i;
            let mut best = i;
            let mut quiet = 0;
            while j < d.len() && quiet < PEAK_SEPARATION_D {
                if d.gust_max[j] >= PEAK_MIN_GUST {
                    quiet = 0;
                    if d.gust_max[j] > d.gust_max[best] {
                        best = j;
                    }
                } else {
                    quiet += 1;
                }
                j += 1;
            }
            peaks.push(((d.date[best].1.clamp(1, 12) - 1) as u8, d.gust_max[best]));
            i = j;
        } else {
            i += 1;
        }
    }
    let mut s = SiteStats {
        id: id.into(),
        lat,
        lon,
        grid_lat: d.lat,
        grid_lon: d.lon,
        elevation_m: d.elevation_m,
        timezone: d.timezone.clone(),
        model: MODEL.into(),
        daily_from: y0,
        daily_to: y1,
        years,
        hourly_from: String::new(),
        hourly_to: String::new(),
        hourly_years: 0.0,
        months,
        weibull_k,
        temp_sd_summer_c: (ss / ns.max(1.0)).sqrt(),
        temp_sd_winter_c: (sw / nw.max(1.0)).sqrt(),
        storm_peaks: peaks,
        gaps: d.gaps,
        source: format!("Open-Meteo {MODEL} 일별 {y0}–{y1}"),
    };
    if let Some(h) = h {
        add_hourly(&mut s, h);
    }
    s
}

/// 시간별 자료에서 젖음 시간 · 안개 · 뇌전 · 방위별 풍속 · SO₂ · 파고 통계를 더한다
pub fn add_hourly(s: &mut SiteStats, h: &Hourly) {
    if h.is_empty() {
        return;
    }
    let first = h.time[0];
    let last = *h.time.last().unwrap();
    let hy = (h.len() as f64 / 8766.0).max(1.0 / 12.0);
    s.hourly_from = format!("{:04}-{:02}-{:02}", first.0, first.1, first.2);
    s.hourly_to = format!("{:04}-{:02}-{:02}", last.0, last.1, last.2);
    s.hourly_years = hy;
    for m in s.months.iter_mut() {
        m.hours = 0.0;
        m.tow_h = 0.0;
        m.rh90_h = 0.0;
        m.rh95_h = 0.0;
        m.precip_h = 0.0;
        m.fog_days = vec![0.0; FOG_RH.len()];
        m.thunder_days = vec![0.0; THUNDER_MM_H.len()];
        m.sector_speed_h = vec![vec![0.0; SPEED_BINS]; SECTORS];
    }
    // 해마다 그 달의 시간 수가 다르므로 '몇 년치 그 달'로 나눈다
    let mut month_hours = [0.0f64; 12];
    let mut so2 = [(0.0f64, 0.0f64); 12];
    let mut waves: Vec<Vec<f64>> = vec![Vec::new(); 12];
    let mut i = 0;
    while i < h.len() {
        // 하루 단위로 안개 · 뇌전 판정
        let (y, mo, dd, _) = h.time[i];
        let m = (mo.clamp(1, 12) - 1) as usize;
        let mut j = i;
        let mut fog_hit = vec![false; FOG_RH.len()];
        let mut th_hit = vec![false; THUNDER_MM_H.len()];
        while j < h.len() && h.time[j].0 == y && h.time[j].1 == mo && h.time[j].2 == dd {
            let (t, rh, p, w) = (h.temp_c[j], h.rh_pct[j], h.precip_mm[j], h.wind_mps[j]);
            month_hours[m] += 1.0;
            let mm = &mut s.months[m];
            if rh > TOW_RH && t > 0.0 {
                mm.tow_h += 1.0;
            }
            if rh >= 90.0 {
                mm.rh90_h += 1.0;
            }
            if rh >= 95.0 {
                mm.rh95_h += 1.0;
            }
            if p >= 0.1 {
                mm.precip_h += 1.0;
            }
            if w <= FOG_WIND_MAX && p < FOG_RAIN_MAX {
                for (k, th) in FOG_RH.iter().enumerate() {
                    if rh >= *th {
                        fog_hit[k] = true;
                    }
                }
            }
            if t >= THUNDER_MIN_TEMP {
                for (k, th) in THUNDER_MM_H.iter().enumerate() {
                    if p >= *th {
                        th_hit[k] = true;
                    }
                }
            }
            if w >= 0.5 {
                let b = (w as usize).min(SPEED_BINS - 1);
                mm.sector_speed_h[sector_of(h.wind_from_deg[j])][b] += 1.0;
            }
            if let Some(v) = h.so2.get(j) {
                so2[m].0 += v;
                so2[m].1 += 1.0;
            }
            if let Some(v) = h.wave_m.get(j) {
                waves[m].push(*v);
            }
            j += 1;
        }
        for (k, hit) in fog_hit.iter().enumerate() {
            if *hit {
                s.months[m].fog_days[k] += 1.0;
            }
        }
        for (k, hit) in th_hit.iter().enumerate() {
            if *hit {
                s.months[m].thunder_days[k] += 1.0;
            }
        }
        i = j;
    }
    for (m, mm) in s.months.iter_mut().enumerate() {
        // 그 달 자료가 몇 년치인가 (시간 수 / 그 달 평년 시간 수)
        let yrs = (month_hours[m] / (DAYS_IN_MONTH[m] as f64 * 24.0)).max(1e-9);
        if month_hours[m] <= 0.0 {
            continue;
        }
        mm.hours = month_hours[m] / yrs;
        for x in [&mut mm.tow_h, &mut mm.rh90_h, &mut mm.rh95_h, &mut mm.precip_h] {
            *x /= yrs;
        }
        for x in mm.fog_days.iter_mut().chain(mm.thunder_days.iter_mut()) {
            *x /= yrs;
        }
        for row in mm.sector_speed_h.iter_mut() {
            for x in row.iter_mut() {
                *x /= yrs;
            }
        }
        if so2[m].1 > 0.0 {
            mm.so2_ug_m3 = Some(so2[m].0 / so2[m].1);
        }
        if !waves[m].is_empty() {
            let w = &mut waves[m];
            w.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            mm.wave_mean_m = Some(w.iter().sum::<f64>() / w.len() as f64);
            mm.wave_p90_m = Some(w[((w.len() as f64 * 0.9) as usize).min(w.len() - 1)]);
        }
    }
    s.source = format!("{} · 시간별 {}–{}", s.source, s.hourly_from, s.hourly_to);
}

/// 단조 감소 격자(문턱 → 일수)에서 일수 `target` 에 맞는 문턱을 직선 보간으로 찾는다
pub fn threshold_for(grid: &[f64], counts: &[f64], target: f64) -> f64 {
    if grid.is_empty() || counts.len() != grid.len() {
        return f64::NAN;
    }
    if target >= counts[0] {
        return grid[0];
    }
    for k in 1..grid.len() {
        if target >= counts[k] {
            let (c0, c1) = (counts[k - 1], counts[k]);
            let f = if (c0 - c1).abs() < 1e-12 { 0.0 } else { (c0 - target) / (c0 - c1) };
            return grid[k - 1] + f * (grid[k] - grid[k - 1]);
        }
    }
    *grid.last().unwrap()
}

/// 문턱 `x` 에서의 일수 (격자 직선 보간, 바깥은 끝값)
pub fn count_at(grid: &[f64], counts: &[f64], x: f64) -> f64 {
    if grid.is_empty() || counts.len() != grid.len() {
        return 0.0;
    }
    if x <= grid[0] {
        return counts[0];
    }
    for k in 1..grid.len() {
        if x <= grid[k] {
            let f = (x - grid[k - 1]) / (grid[k] - grid[k - 1]);
            return counts[k - 1] + f * (counts[k] - counts[k - 1]);
        }
    }
    *counts.last().unwrap()
}

impl SiteStats {
    pub fn has_hourly(&self) -> bool {
        self.hourly_years > 0.0
    }

    /// 연간 젖음 시간 (h/년)
    pub fn tow_h(&self) -> f64 {
        self.months.iter().map(|m| m.tow_h).sum()
    }

    pub fn annual_mean_temp(&self) -> f64 {
        self.months.iter().zip(DAYS_IN_MONTH.iter()).map(|(m, d)| m.temp_mean_c * *d as f64).sum::<f64>() / 365.0
    }

    pub fn annual_mean_rh(&self) -> f64 {
        self.months.iter().zip(DAYS_IN_MONTH.iter()).map(|(m, d)| m.rh_pct * *d as f64).sum::<f64>() / 365.0
    }

    /// 폭풍일 (일 최대 풍속 × 배수 ≥ 13.9 m/s, 일/년)
    pub fn storm_days(&self, wind_scale: f64) -> f64 {
        let th = 13.9 / wind_scale.max(1e-6);
        self.months
            .iter()
            .map(|m| m.wind_max_hist.iter().enumerate().map(|(b, c)| if (b as f64 + 0.5) * WIND_BIN >= th { *c } else { 0.0 }).sum::<f64>())
            .sum()
    }

    /// 봉우리 배수 `gust_scale` 를 적용한 T 년 재현 순간풍속 (임계 25 m/s 초과 봉우리, 지수 꼬리)
    pub fn return_gust(&self, gust_scale: f64, years: f64) -> f64 {
        let ty = storm_climate(self, gust_scale);
        match ty {
            Some(t) => t.return_level(years),
            None => self.storm_peaks.iter().map(|p| p.1 * gust_scale).fold(0.0, f64::max),
        }
    }
}

/// 폭풍 봉우리 → 임계값 초과 모델 (임계 25 m/s, 기상청 프리셋과 같은 꼴)
pub fn storm_climate(s: &SiteStats, gust_scale: f64) -> Option<TyphoonClimate> {
    let thr = 25.0;
    let ex: Vec<(u8, f64)> = s.storm_peaks.iter().map(|(m, g)| (*m, g * gust_scale)).filter(|(_, g)| *g >= thr).collect();
    if ex.len() < 2 || s.years <= 0.0 {
        return None;
    }
    let beta = (ex.iter().map(|(_, g)| g - thr).sum::<f64>() / ex.len() as f64).max(0.5);
    let mut w = [0.2f64; 12];
    for (m, _) in ex.iter() {
        w[*m as usize] += 1.0;
    }
    Some(TyphoonClimate {
        per_year: ex.len() as f64 / s.years,
        threshold_mps: thr,
        excess_scale_mps: beta,
        month_weights: w.to_vec(),
        duration_h: 18.0,
        source: format!("ERA5 일 최대 순간풍속 봉우리 {}건 ({}–{}), 순간풍속 배수 {:.2}", ex.len(), s.daily_from, s.daily_to, gust_scale),
    })
}

/// 통계 + 보정 → 합성 날씨용 기후
pub fn climate_from_stats(s: &SiteStats, adj: &Era5Adjust, id: &str, name: &str) -> Climate {
    let months: Vec<MonthClimate> = s
        .months
        .iter()
        .map(|m| {
            let range = ((m.temp_max_c - m.temp_min_c) * adj.range_scale).max(0.0);
            MonthClimate {
                temp_mean_c: m.temp_mean_c + adj.temp_offset_c,
                temp_range_c: range,
                wind_mean_mps: (m.wind_mean_mps * adj.wind_scale).max(0.1),
                precip_mm: m.precip_mm * adj.precip_scale,
                rain_days: count_at(&RAIN_MM, &m.rain_days, adj.rain_day_mm).max(0.0),
                snow_days: count_at(&SNOW_CM, &m.snow_days, adj.snow_day_cm).max(0.0),
                thunder_days: if m.thunder_days.is_empty() { 0.0 } else { count_at(&THUNDER_MM_H, &m.thunder_days, adj.thunder_mm_h).max(0.0) },
                fog_days: if m.fog_days.is_empty() { 0.0 } else { count_at(&FOG_RH, &m.fog_days, adj.fog_rh_pct).max(0.0) },
                humidity_pct: (m.rh_pct + adj.rh_offset_pct).clamp(5.0, 100.0),
            }
        })
        .collect();
    let rose: Vec<Vec<f64>> = s.months.iter().map(|m| m.dir_freq.clone()).collect();
    Climate {
        id: id.into(),
        name: name.into(),
        source: format!("{} → 기상청 기준 보정 ({}) · {ATTRIBUTION}", s.source, adj.source),
        months,
        gust_factor: 1.5,
        weibull_k: s.weibull_k,
        hourly_wind_sd: 0.25,
        alpha: 0.14,
        temp_anomaly_sd_c: s.temp_sd_summer_c.clamp(0.5, 5.0),
        temp_anomaly_sd_winter_c: s.temp_sd_winter_c.clamp(0.5, 8.0),
        snow_temp_c: 1.5,
        snow_flurry_prob: 0.0,
        typhoon: storm_climate(s, adj.gust_scale),
        wind_rose: rose,
    }
}

// ---------------------------------------------------------------- 재생

/// 시간별 실제 날씨 → 시뮬레이션 날씨. `from` 색인부터 `days` 일 (2월 29일은 건너뛴다)
pub fn replay(h: &Hourly, from: usize, days: u32, height_m: f64, alpha: f64, adj: &Era5Adjust, seed: u64) -> Result<Vec<HourWeather>, String> {
    let need = days as usize * 24;
    let hfac = (height_m.max(2.0) / 10.0).powf(alpha.clamp(0.0, 0.4));
    let mut rng = Rng::new(seed ^ 0x4552_4135);
    let mut out = Vec::with_capacity(need);
    let mut last_wet: Option<usize> = None;
    let mut i = from;
    while out.len() < need {
        if i >= h.len() {
            return Err(format!("재생 날씨가 모자랍니다: {} 시간 필요, {} 시간 남음 (자료 {}–{})", need, out.len(), Hourly::key(&h.time[0]), Hourly::key(h.time.last().unwrap())));
        }
        let (_, mo, dd, hr) = h.time[i];
        let Some(doy) = doy_noleap(mo, dd) else {
            i += 1;
            continue;
        };
        let k = out.len();
        let t = h.temp_c[i] + adj.temp_offset_c;
        let rh = (h.rh_pct[i] + adj.rh_offset_pct).clamp(1.0, 100.0);
        let w10 = (h.wind_mps[i] * adj.wind_scale).max(0.0);
        let wind = w10 * hfac;
        let gust = (h.gust_mps[i] * adj.gust_scale * hfac).max(wind * 1.1);
        let rain = (h.precip_mm[i] * adj.precip_scale).max(0.0);
        let snow = h.snow_cm[i] > 0.05 || (rain > 0.0 && t <= 1.5);
        if rain > 0.0 {
            last_wet = Some(k);
        }
        let fog = rh >= adj.fog_rh_pct && w10 <= FOG_WIND_MAX && rain < FOG_RAIN_MAX;
        let mut vis: f64 = 20_000.0;
        if rain > 0.0 {
            vis = if snow { 600.0 + 1500.0 * rng.uniform() } else if rain > 10.0 { 1200.0 } else { 4000.0 + 4000.0 * rng.uniform() };
        }
        if fog {
            // 문턱에서 900 m, 포화에 가까울수록 60 m 쪽으로 (로그 보간)
            let f = ((rh - adj.fog_rh_pct) / (100.0 - adj.fog_rh_pct).max(0.5)).clamp(0.0, 1.0);
            vis = vis.min((900.0f64.ln() + (60.0f64.ln() - 900.0f64.ln()) * f).exp());
        }
        let recent_wet = last_wet.map(|w| k - w <= 12).unwrap_or(false);
        let icy = t <= 0.5 && (recent_wet || fog);
        let thunder_km = if rain >= adj.thunder_mm_h && t >= THUNDER_MIN_TEMP { Some(15.0 * rng.uniform()) } else { None };
        let summer = (5..=9).contains(&mo);
        out.push(HourWeather {
            day: (k / 24) as u32,
            hour: hr,
            doy,
            temp_c: t,
            humidity_pct: rh,
            wind_mean_mps: wind,
            gust_mps: gust,
            // 시뮬레이션 풍향은 주행 방향(+x) 기준 '불어 가는' 쪽 — 주행 축을 북쪽으로 보고 바꾼다
            wind_dir_deg: (h.wind_from_deg[i] + 180.0).rem_euclid(360.0),
            wind_from_deg: Some(h.wind_from_deg[i].rem_euclid(360.0)),
            rain_mm: rain,
            snow,
            icy,
            thunder_km,
            visibility_m: vis,
            fog: fog && vis < 1000.0,
            typhoon: h.gust_mps[i] * adj.gust_scale >= 25.0,
            feels_like_c: weather::feels_like_c(t, rh, w10, summer),
        });
        i += 1;
    }
    // 시각이 0시에서 시작하지 않으면 day/hour 를 실제 시각에 맞춘다
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_daily(years: i32) -> Daily {
        let mut d = Daily { lat: 35.0, lon: 129.0, elevation_m: 10.0, timezone: "Asia/Seoul".into(), ..Default::default() };
        let mut rng = Rng::new(7);
        for y in 2000..2000 + years {
            for m in 1..=12u32 {
                for day in 1..=DAYS_IN_MONTH[(m - 1) as usize] {
                    d.date.push((y, m, day));
                    let base = 15.0 - 10.0 * (2.0 * std::f64::consts::PI * (m as f64 - 1.0) / 12.0).cos();
                    let tm = base + rng.normal();
                    d.t_mean.push(tm);
                    d.t_max.push(tm + 4.0);
                    d.t_min.push(tm - 4.0);
                    d.precip.push(if rng.chance(0.3) { rng.exp(8.0) } else { 0.0 });
                    d.snow_cm.push(0.0);
                    let w = 3.0 + rng.uniform();
                    d.wind_mean.push(w);
                    d.wind_max.push(w * 1.8);
                    d.gust_max.push(if rng.chance(0.01) { 30.0 + 5.0 * rng.uniform() } else { w * 3.0 });
                    d.rh_mean.push(70.0);
                    d.dir_dom.push(if m <= 3 { 315.0 } else { 160.0 });
                }
            }
        }
        d
    }

    #[test]
    fn gaps_are_filled_linearly() {
        let mut v = vec![f64::NAN, 1.0, f64::NAN, f64::NAN, 4.0, f64::NAN];
        assert_eq!(fill_gaps(&mut v), 4);
        assert_eq!(v, vec![1.0, 1.0, 2.0, 3.0, 4.0, 4.0]);
    }

    #[test]
    fn threshold_grid_inverts_counts() {
        let counts = [100.0, 80.0, 60.0, 40.0, 30.0, 20.0, 10.0];
        let th = threshold_for(&RAIN_MM, &counts, 70.0);
        assert!((th - 0.35).abs() < 1e-9, "{th}");
        assert!((count_at(&RAIN_MM, &counts, th) - 70.0).abs() < 1e-9);
    }

    #[test]
    fn stats_and_climate_round_trip() {
        let d = fake_daily(10);
        let s = build_stats("x", 35.0, 129.0, &d, None);
        assert_eq!(s.months.len(), 12);
        assert!((s.years - 10.0).abs() < 1e-9);
        // 1월 평균 ≈ 5 °C, 7월 ≈ 25 °C 근처
        assert!(s.months[0].temp_mean_c < s.months[6].temp_mean_c);
        // 풍향: 1–3월 북서(7번 칸)
        assert!(s.months[0].dir_freq[7] > 0.99, "{:?}", s.months[0].dir_freq);
        let c = climate_from_stats(&s, &Era5Adjust::default(), "x", "x");
        c.validate().unwrap();
        assert_eq!(c.wind_rose.len(), 12);
        // 강수일: 0.1 mm 문턱 → 약 30 % 의 날
        let rd: f64 = c.months.iter().map(|m| m.rain_days).sum();
        assert!((rd - 0.3 * 365.0).abs() < 25.0, "{rd}");
        // 폭풍 봉우리가 있으면 사건 모델
        assert!(c.typhoon.is_some());
    }

    #[test]
    fn replay_converts_hours_and_skips_leap_day() {
        let mut h = Hourly { lat: 35.0, lon: 129.0, ..Default::default() };
        for (m, d) in [(2u32, 28u32), (2, 29), (3, 1)] {
            for hr in 0..24u8 {
                h.time.push((2024, m, d, hr));
                h.temp_c.push(5.0);
                h.rh_pct.push(if hr < 6 { 99.0 } else { 60.0 });
                h.dew_c.push(4.0);
                h.precip_mm.push(0.0);
                h.snow_cm.push(0.0);
                h.wind_mps.push(1.0);
                h.wind_from_deg.push(180.0);
                h.gust_mps.push(3.0);
            }
        }
        let w = replay(&h, 0, 2, 10.0, 0.14, &Era5Adjust::default(), 1).unwrap();
        assert_eq!(w.len(), 48);
        assert_eq!(w[24].doy, weather::doy_of(3, 1));
        assert!(w[0].fog && w[0].visibility_m < 1000.0);
        assert!(!w[12].fog);
        // 남풍(180°에서 불어옴) → 북쪽(0°)으로 불어 간다
        assert!(w[0].wind_dir_deg.abs() < 1e-9 || (w[0].wind_dir_deg - 360.0).abs() < 1e-9);
        assert!(replay(&h, 0, 3, 10.0, 0.14, &Era5Adjust::default(), 1).is_err());
    }
}
