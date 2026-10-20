//! ERA5(Open-Meteo) ↔ 기상청 교차검증 · 보정 — 기상청 6개 지점 좌표의 ERA5 통계를 1991–2020 평년값 · 연 일수 ·
//! 순위값과 비교하고, `Era5Adjust` 계수를 지점 하나를 빼고 맞춘 뒤 뺀 지점에서 오차를 잰다 (현장 하나 빼기).
//! 이렇게 맞춘 계수를 세계 항만 기후(`ports`)에 쓰고, 지점 하나 빼기 오차를 그 기후의 예상 오차로 남긴다.
//!
//! | 계수 | 맞추는 대상 (기상청) |
//! | --- | --- |
//! | temp_offset_c · rh_offset_pct | 월평균 기온 · 상대습도 (평균 차) |
//! | range_scale | 월평균 일교차 (최소제곱 배수) |
//! | wind_scale | 월평균 풍속 (최소제곱 배수) |
//! | precip_scale | 연 강수량 (총량 비) |
//! | rain_day_mm · snow_day_cm | 강수일 (0.1 mm) · 눈 일수 (문턱 격자 보간) |
//! | fog_rh_pct · thunder_mm_h | 안개 · 뇌전 일수 (시간 자료 문턱 격자) |
//! | gust_scale | 10년 재현 순간풍속 (순위값) |

use crate::climate_data::{self, ClimateTargets};
use crate::weather::{Climate, DAYS_IN_MONTH};
use crate::weather_api::{self, Era5Adjust, SiteStats, FOG_RH, RAIN_MM, SNOW_CM, THUNDER_MM_H};
use serde::{Deserialize, Serialize};

/// 한 지점의 비교 자료
#[derive(Debug, Clone)]
pub struct Station {
    pub id: String,
    pub stats: SiteStats,
    pub kma: Climate,
    pub targets: ClimateTargets,
}

/// 기상청 지점 (ERA5 통계가 있는 것만)
pub fn stations() -> Vec<Station> {
    let kma = climate_data::raw_presets();
    let tg = climate_data::targets();
    kma.into_iter()
        .filter_map(|c| {
            let stats = crate::ports::stats(&format!("kma_{}", c.id))?;
            let targets = tg.iter().find(|t| t.id == c.id)?.clone();
            Some(Station { id: c.id.clone(), stats, kma: c, targets })
        })
        .collect()
}

/// 순위값에서 T 년 재현 순간풍속 (순위/기록 년수 = 1/T 를 직선 보간)
pub fn kma_return_gust(t: &ClimateTargets, years: f64) -> f64 {
    let r = t.record_years / years;
    let v = &t.record_gusts;
    if v.is_empty() {
        return f64::NAN;
    }
    let k = (r - 1.0).clamp(0.0, (v.len() - 1) as f64);
    let i = k.floor() as usize;
    let f = k - i as f64;
    if i + 1 < v.len() {
        v[i] * (1.0 - f) + v[i + 1] * f
    } else {
        v[i]
    }
}

fn month_w(m: usize) -> f64 {
    DAYS_IN_MONTH[m] as f64 / 365.0
}

/// 문턱 격자에서 연 합계가 `target` 이 되는 문턱 (합계는 문턱에 대해 감소)
fn fit_threshold(grid: &[f64], sums: &[f64], target: f64) -> f64 {
    weather_api::threshold_for(grid, sums, target)
}

fn sum_grid(st: &[&Station], f: impl Fn(&Station) -> Vec<f64>, n: usize) -> Vec<f64> {
    let mut acc = vec![0.0; n];
    for s in st {
        for (k, v) in f(s).iter().enumerate() {
            if k < n {
                acc[k] += v;
            }
        }
    }
    acc
}

/// 지점 묶음으로 보정 계수를 맞춘다
pub fn fit(st: &[&Station]) -> Era5Adjust {
    let mut a = Era5Adjust::default();
    if st.is_empty() {
        return a;
    }
    let (mut dt, mut drh, mut n) = (0.0, 0.0, 0.0);
    let (mut rr, mut re, mut wk, mut we) = (0.0, 0.0, 0.0, 0.0);
    let (mut pk, mut pe) = (0.0, 0.0);
    for s in st {
        for m in 0..12 {
            let e = &s.stats.months[m];
            let k = &s.kma.months[m];
            dt += k.temp_mean_c - e.temp_mean_c;
            drh += k.humidity_pct - e.rh_pct;
            n += 1.0;
            let er = e.temp_max_c - e.temp_min_c;
            rr += k.temp_range_c * er;
            re += er * er;
            wk += k.wind_mean_mps * e.wind_mean_mps;
            we += e.wind_mean_mps * e.wind_mean_mps;
            pk += k.precip_mm;
            pe += e.precip_mm;
        }
    }
    a.temp_offset_c = dt / n;
    a.rh_offset_pct = drh / n;
    a.range_scale = if re > 0.0 { rr / re } else { 1.0 };
    a.wind_scale = if we > 0.0 { wk / we } else { 1.0 };
    a.precip_scale = if pe > 0.0 { pk / pe } else { 1.0 };
    // 일수 문턱: 지점 합계가 기상청 합계와 같도록
    let rain_target: f64 = st.iter().map(|s| s.kma.months.iter().map(|m| m.rain_days).sum::<f64>()).sum();
    let rain = sum_grid(st, |s| (0..RAIN_MM.len()).map(|k| s.stats.months.iter().map(|m| m.rain_days[k]).sum()).collect(), RAIN_MM.len());
    a.rain_day_mm = fit_threshold(&RAIN_MM, &rain, rain_target);
    let snow_target: f64 = st.iter().map(|s| s.targets.snow_days).sum();
    let snow = sum_grid(st, |s| (0..SNOW_CM.len()).map(|k| s.stats.months.iter().map(|m| m.snow_days[k]).sum()).collect(), SNOW_CM.len());
    a.snow_day_cm = fit_threshold(&SNOW_CM, &snow, snow_target);
    let hourly: Vec<&&Station> = st.iter().filter(|s| s.stats.has_hourly()).collect();
    if !hourly.is_empty() {
        let fog_target: f64 = hourly.iter().map(|s| s.targets.fog_days).sum();
        let fog = sum_grid(&hourly.iter().map(|s| **s).collect::<Vec<_>>(), |s| (0..FOG_RH.len()).map(|k| s.stats.months.iter().map(|m| m.fog_days.get(k).cloned().unwrap_or(0.0)).sum()).collect(), FOG_RH.len());
        a.fog_rh_pct = fit_threshold(&FOG_RH, &fog, fog_target);
        let th_target: f64 = hourly.iter().map(|s| s.targets.thunder_days).sum();
        let th = sum_grid(&hourly.iter().map(|s| **s).collect::<Vec<_>>(), |s| (0..THUNDER_MM_H.len()).map(|k| s.stats.months.iter().map(|m| m.thunder_days.get(k).cloned().unwrap_or(0.0)).sum()).collect(), THUNDER_MM_H.len());
        a.thunder_mm_h = fit_threshold(&THUNDER_MM_H, &th, th_target);
    }
    // 순간풍속 배수: 지점마다 10년 재현값이 순위값과 같아지는 배수의 중앙값 (이분법)
    let mut gs: Vec<f64> = st
        .iter()
        .filter_map(|s| {
            let target = kma_return_gust(&s.targets, 10.0);
            if !target.is_finite() {
                return None;
            }
            let (mut lo, mut hi) = (0.5, 3.0);
            for _ in 0..40 {
                let mid = 0.5 * (lo + hi);
                if s.stats.return_gust(mid, 10.0) < target {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            Some(0.5 * (lo + hi))
        })
        .collect();
    gs.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    if !gs.is_empty() {
        a.gust_scale = gs[gs.len() / 2];
    }
    a.source = format!("기상청 {}개 지점 맞춤 (era5cal)", st.len());
    a
}

/// 한 지점의 항목별 (ERA5 보정값, 기상청 값)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pair {
    pub name: String,
    pub unit: String,
    pub kma: f64,
    pub era5: f64,
}

/// ERA5 통계 + 보정 → 기상청 비교 항목 (연 단위)
pub fn compare(s: &Station, a: &Era5Adjust) -> Vec<Pair> {
    let c = weather_api::climate_from_stats(&s.stats, a, &s.id, &s.id);
    let annual = |f: &dyn Fn(usize) -> f64| (0..12).map(|m| f(m) * month_w(m)).sum::<f64>();
    let sum = |f: &dyn Fn(usize) -> f64| (0..12).map(f).sum::<f64>();
    let p = |name: &str, unit: &str, kma: f64, era5: f64| Pair { name: name.into(), unit: unit.into(), kma, era5 };
    let mut v = vec![
        p("연평균 기온", "°C", annual(&|m| s.kma.months[m].temp_mean_c), annual(&|m| c.months[m].temp_mean_c)),
        p("평균 일교차", "K", annual(&|m| s.kma.months[m].temp_range_c), annual(&|m| c.months[m].temp_range_c)),
        p("연평균 풍속", "m/s", annual(&|m| s.kma.months[m].wind_mean_mps), annual(&|m| c.months[m].wind_mean_mps)),
        p("연평균 상대습도", "%", annual(&|m| s.kma.months[m].humidity_pct), annual(&|m| c.months[m].humidity_pct)),
        p("연 강수량", "mm", sum(&|m| s.kma.months[m].precip_mm), sum(&|m| c.months[m].precip_mm)),
        p("연 강수일수", "일", sum(&|m| s.kma.months[m].rain_days), sum(&|m| c.months[m].rain_days)),
        p("눈 일수", "일", s.targets.snow_days, sum(&|m| c.months[m].snow_days)),
        p("폭풍일 (10분 13.9 m/s)", "일", s.targets.storm_days, s.stats.storm_days(a.wind_scale)),
        p("10년 순간풍속", "m/s", kma_return_gust(&s.targets, 10.0), s.stats.return_gust(a.gust_scale, 10.0)),
    ];
    if s.stats.has_hourly() {
        v.push(p("안개 일수", "일", s.targets.fog_days, sum(&|m| c.months[m].fog_days)));
        v.push(p("뇌전 일수", "일", s.targets.thunder_days, sum(&|m| c.months[m].thunder_days)));
    }
    v
}

/// 항목 하나의 지점 평균 오차
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemError {
    pub name: String,
    pub unit: String,
    pub kma_mean: f64,
    /// 지점 평균 절대 오차 — 보정 전 · 지점 하나 빼기 보정 · 전체 맞춤
    pub mae_before: f64,
    pub mae_loso: f64,
    pub mae_all: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Era5Report {
    pub stations: Vec<String>,
    /// 전체 지점으로 맞춘 계수 (항만 기후에 쓴다)
    pub adjust: Option<Era5Adjust>,
    /// 지점 하나 빼기 때 맞춘 계수
    pub folds: Vec<(String, Era5Adjust)>,
    pub items: Vec<ItemError>,
    /// 지점별 상세 (보정 전 · 지점 하나 빼기)
    pub detail: Vec<(String, Vec<Pair>, Vec<Pair>)>,
    pub notes: Vec<String>,
}

pub fn run() -> Era5Report {
    let st = stations();
    let mut rep = Era5Report { stations: st.iter().map(|s| s.id.clone()).collect(), ..Default::default() };
    if st.is_empty() {
        rep.notes.push("ERA5 통계(data/climate/ports.json)에 기상청 지점이 없습니다 — crane_demo weather build".into());
        return rep;
    }
    let all: Vec<&Station> = st.iter().collect();
    let adj_all = fit(&all);
    let before = Era5Adjust::default();
    let mut loso_pairs: Vec<Vec<Pair>> = Vec::new();
    let mut before_pairs: Vec<Vec<Pair>> = Vec::new();
    let mut all_pairs: Vec<Vec<Pair>> = Vec::new();
    for (k, s) in st.iter().enumerate() {
        let train: Vec<&Station> = st.iter().enumerate().filter(|(j, _)| *j != k).map(|(_, x)| x).collect();
        let a = fit(&train);
        rep.folds.push((s.id.clone(), a.clone()));
        let b = compare(s, &before);
        let l = compare(s, &a);
        rep.detail.push((s.id.clone(), b.clone(), l.clone()));
        before_pairs.push(b);
        loso_pairs.push(l);
        all_pairs.push(compare(s, &adj_all));
    }
    let names: Vec<(String, String)> = before_pairs.iter().flat_map(|v| v.iter().map(|p| (p.name.clone(), p.unit.clone()))).fold(Vec::new(), |mut acc, x| {
        if !acc.contains(&x) {
            acc.push(x);
        }
        acc
    });
    for (name, unit) in names {
        let pick = |v: &Vec<Vec<Pair>>| -> Vec<(f64, f64)> { v.iter().filter_map(|ps| ps.iter().find(|p| p.name == name).map(|p| (p.kma, p.era5))).filter(|(a, b)| a.is_finite() && b.is_finite()).collect() };
        let mae = |xs: &Vec<(f64, f64)>| xs.iter().map(|(k, e)| (e - k).abs()).sum::<f64>() / xs.len().max(1) as f64;
        let b = pick(&before_pairs);
        let l = pick(&loso_pairs);
        let a = pick(&all_pairs);
        rep.items.push(ItemError {
            name: name.clone(),
            unit,
            kma_mean: b.iter().map(|x| x.0).sum::<f64>() / b.len().max(1) as f64,
            mae_before: mae(&b),
            mae_loso: mae(&l),
            mae_all: mae(&a),
        });
    }
    rep.adjust = Some(adj_all);
    rep.notes.push("ERA5 = Open-Meteo era5_seamless (ERA5-Land 0.1° + ERA5), 기상청 = 1991–2020 평년값 · 연 일수 · 순위값".into());
    rep.notes.push("안개 · 뇌전은 ERA5 에 없어 시간 자료(2022–2023)의 습도 · 강수 문턱으로 판정 — 2년 표본".into());
    rep
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_gust_from_ranks() {
        let t = ClimateTargets { id: "x".into(), station: 1, storm_days: 0.0, coldwave_days: 0.0, snow_days: 0.0, fog_days: 0.0, thunder_days: 0.0, record_gusts: vec![40.0, 38.0, 36.0, 34.0, 32.0], record_years: 50.0 };
        // 10년 재현 = 순위 5 (50/10)
        assert!((kma_return_gust(&t, 10.0) - 32.0).abs() < 1e-9);
        assert!((kma_return_gust(&t, 25.0) - 38.0).abs() < 1e-9);
    }
}
