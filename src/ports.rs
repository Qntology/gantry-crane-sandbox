//! 세계 항만 · 해안 위치 — 위치 목록(`data/climate/port_sites.json`)과 Open-Meteo ERA5 통계(`data/climate/ports.json`).
//!
//! - `kind = "kma_station"`: 기상청 지점 좌표의 ERA5 — 평년값과 비교해 ERA5 보정 계수를 정하는 데만 쓴다 (`fieldcal`).
//! - `kind = "port"`: 항만 · 해안 현장 — 보정한 ERA5 통계로 합성 날씨용 기후를 만들고, 해안 염해 계산에 바람장미 ·
//!   젖음 시간 · 파고 · SO₂ 를 준다.
//!
//! 통계 파일은 `crane_demo weather build` 가 캐시(`out/weather_cache`)에서 다시 만든다. 인터넷 없이도 저장해 둔
//! 통계로 동작한다.

use crate::weather::Climate;
use crate::weather_api::{self, Fetcher, SiteStats};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const SITES_JSON: &str = include_str!("../data/climate/port_sites.json");
pub const PORTS_JSON: &str = include_str!("../data/climate/ports.json");

/// 일별 · 시간별 자료 기간 (prefetch 와 같아야 캐시를 다시 쓴다)
pub const KMA_DAILY: (i32, i32) = (1991, 2020);
pub const KMA_HOURLY: (&str, &str) = ("2022-01-01", "2023-12-31");
pub const PORT_DAILY: (i32, i32) = (2014, 2023);
pub const PORT_HOURLY: (&str, &str) = ("2023-01-01", "2023-12-31");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSite {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    /// "kma_station" · "port"
    pub kind: String,
    #[serde(default)]
    pub kma_station: Option<u32>,
    /// 가까운 기상청 평년값 기후 (있으면)
    #[serde(default)]
    pub climate_id: Option<String>,
    /// 현장에서 바다를 보는 방위 (°, 북 0 시계 방향)
    pub sea_bearing_deg: f64,
    /// harbor · bay · open · tidal · strait · river
    pub coast_type: String,
    #[serde(default)]
    pub marine_lat: Option<f64>,
    #[serde(default)]
    pub marine_lon: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SitesFile {
    #[serde(default)]
    note: String,
    sites: Vec<PortSite>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortsFile {
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub built: String,
    #[serde(default)]
    pub sites: Vec<SiteStats>,
}

pub fn sites() -> Vec<PortSite> {
    static S: OnceLock<Vec<PortSite>> = OnceLock::new();
    S.get_or_init(|| serde_json::from_str::<SitesFile>(SITES_JSON).map(|f| f.sites).unwrap_or_default()).clone()
}

pub fn site(id: &str) -> Option<PortSite> {
    sites().into_iter().find(|s| s.id == id)
}

fn stats_cache() -> &'static Vec<SiteStats> {
    static S: OnceLock<Vec<SiteStats>> = OnceLock::new();
    S.get_or_init(|| serde_json::from_str::<PortsFile>(PORTS_JSON).map(|f| f.sites).unwrap_or_default())
}

pub fn stats_all() -> Vec<SiteStats> {
    stats_cache().clone()
}

pub fn stats(id: &str) -> Option<SiteStats> {
    stats_cache().iter().find(|s| s.id == id).cloned()
}

/// 항만 기후 (보정한 ERA5). 기상청 지점(kma_station)은 기후로 내지 않는다
pub fn climate(id: &str) -> Option<Climate> {
    let s = site(id)?;
    if s.kind != "port" {
        return None;
    }
    let st = stats(id)?;
    let adj = crate::field_constants::era5_adjust();
    let mut c = weather_api::climate_from_stats(&st, &adj, &s.id, &format!("{} — ERA5 ({})", s.name, s.country));
    crate::field_constants::apply_climate_calibration(&mut c);
    Some(c)
}

pub fn climates() -> Vec<Climate> {
    sites().iter().filter(|s| s.kind == "port").filter_map(|s| climate(&s.id)).collect()
}

/// 위치 하나의 일별 · 시간별 · SO₂ · 파고를 받아 통계를 만든다 (캐시 우선)
pub fn build_one(f: &Fetcher, s: &PortSite) -> Result<SiteStats, String> {
    let kma = s.kind == "kma_station";
    let (y0, y1) = if kma { KMA_DAILY } else { PORT_DAILY };
    let (h0, h1) = if kma { KMA_HOURLY } else { PORT_HOURLY };
    let d = f.daily(s.lat, s.lon, y0, y1)?;
    let mut h = f.hourly(s.lat, s.lon, h0, h1)?;
    if !kma {
        if let Ok(p) = f.so2(s.lat, s.lon, h0, h1) {
            h.so2 = h.align(&p);
        }
        if let (Some(la), Some(lo)) = (s.marine_lat, s.marine_lon) {
            if let Ok(p) = f.waves(la, lo, h0, h1) {
                h.wave_m = h.align(&p);
            }
        }
    }
    let mut st = weather_api::build_stats(&s.id, s.lat, s.lon, &d, Some(&h));
    round_stats(&mut st);
    Ok(st)
}

/// 저장 파일을 작게: 유효 자리 줄이기
pub fn round_stats(s: &mut SiteStats) {
    let r = |x: &mut f64, k: f64| *x = (*x * k).round() / k;
    for m in s.months.iter_mut() {
        for x in [&mut m.temp_mean_c, &mut m.temp_max_c, &mut m.temp_min_c, &mut m.rh_pct, &mut m.wind_mean_mps] {
            r(x, 1000.0);
        }
        for x in [&mut m.precip_mm, &mut m.coldwave_days, &mut m.hours, &mut m.tow_h, &mut m.rh90_h, &mut m.rh95_h, &mut m.precip_h] {
            r(x, 100.0);
        }
        for v in [&mut m.rain_days, &mut m.snow_days, &mut m.wind_max_hist, &mut m.fog_days, &mut m.thunder_days] {
            for x in v.iter_mut() {
                r(x, 1000.0);
            }
        }
        for x in m.dir_freq.iter_mut() {
            r(x, 10000.0);
        }
        for row in m.sector_speed_h.iter_mut() {
            for x in row.iter_mut() {
                r(x, 100.0);
            }
        }
        for x in [&mut m.so2_ug_m3, &mut m.wave_mean_m, &mut m.wave_p90_m].into_iter().flatten() {
            r(x, 1000.0);
        }
    }
    for x in [&mut s.weibull_k, &mut s.temp_sd_summer_c, &mut s.temp_sd_winter_c, &mut s.hourly_years] {
        r(x, 10000.0);
    }
    for p in s.storm_peaks.iter_mut() {
        r(&mut p.1, 100.0);
    }
}

/// 목록 전체를 만든다 (실패한 위치는 건너뛰고 이유를 남긴다)
pub fn build(f: &Fetcher, list: &[PortSite], log: &mut dyn FnMut(&str)) -> PortsFile {
    let mut out = PortsFile {
        note: format!("crane_demo weather build 로 만든 ERA5 통계 ({}). 기상청 지점 {}–{} · 항만 {}–{} 일별, 시간별 {}–{} / {}–{}", weather_api::ATTRIBUTION, KMA_DAILY.0, KMA_DAILY.1, PORT_DAILY.0, PORT_DAILY.1, KMA_HOURLY.0, KMA_HOURLY.1, PORT_HOURLY.0, PORT_HOURLY.1),
        built: String::new(),
        sites: Vec::new(),
    };
    for s in list {
        match build_one(f, s) {
            Ok(st) => {
                log(&format!("{:<22} 격자 {:.2},{:.2} · {}–{} 일별 · 시간 {:.1} 년 · 젖음 {:.0} h/년 · 폭풍 봉우리 {}", s.id, st.grid_lat, st.grid_lon, st.daily_from, st.daily_to, st.hourly_years, st.tow_h(), st.storm_peaks.len()));
                out.sites.push(st);
            }
            Err(e) => log(&format!("{:<22} 건너뜀: {e}", s.id)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_list_parses_and_has_kma_stations_and_ports() {
        let s = sites();
        assert!(s.iter().filter(|x| x.kind == "kma_station").count() == 6, "{}", s.len());
        assert!(s.iter().filter(|x| x.kind == "port").count() >= 10);
        for x in s.iter() {
            assert!((-90.0..=90.0).contains(&x.lat) && (-180.0..=180.0).contains(&x.lon), "{}", x.id);
            assert!(["harbor", "bay", "open", "tidal", "strait", "river"].contains(&x.coast_type.as_str()), "{}", x.id);
        }
    }
}
