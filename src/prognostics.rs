//! 예측 점수 백테스트 · 교차검증 — 부품 상태 '점수'(남은 수명 분포 · 기간 내 고장 확률 · 건강 점수)를 재는 절차.
//!
//! 1. 수집: 현장 운영 시뮬레이션을 여러 해 돌려 부품별 하루 물리 소비(원장 증분) 시계열을 얻는다 — 날씨 · 화물 ·
//!    물동 · 염해가 들어간 실제 사용 패턴.
//! 2. 재현: 같은 소비 시계열에서 개체 차이 · 감마 경로가 다른 부품을 여러 번(복제) 만들고, 점검 · 센서 주기로 관측한다.
//!    교체 기준(참 상태 1)에 닿으면 그날이 수명 끝이고 새 부품으로 바꾼다. 수명이 긴 부품은 소비를 가속해
//!    (평균 수명 약 300일) 기간 안에 수명 끝이 생기게 한다 — 가속 배수는 보고서에 남긴다.
//! 3. 예측: 30일마다 그때까지의 관측만으로 v0 · v1 · v2 · v3 가 남은 수명 분위수 · 30일 고장 확률 · 지금 상태를 낸다.
//! 4. 채점 (`scoring`): CRPS · 핀볼 · 80 % 구간 적중 · 구간 점수 · PIT KS · Brier(분해 · 기술 점수) · α 정확도 ·
//!    지금 상태 80 % 구간 적중.
//! 5. 교차검증: v3 의 퍼짐 보정 배수는 현장 하나를 빼고 나머지에서 맞추고, 뺀 현장에서 잰다 (현장 하나 빼기).
//!    예측은 언제나 그 시점까지의 자료만 쓴다 (시간 순).
//! 6. v4: 뺀 현장 밖 v3 기록의 부품 종류별 PIT 분포 G 로 F' = G∘F 재보정 (Kuleshov et al. 2018) — 같은 현장 하나
//!    빼기로 재고, v3 보다 나을 때만(CRPS 1 % 안 · PIT KS 감소) 현장 운영에 쓴다.

use crate::crane_profile::Axis;
use crate::degradation::{self, Calib, KindParams, Method};
use crate::health::ComponentKind;
use crate::operation::{DecisionPolicy, OpSim, OperationConfig};
use crate::rng::Rng;
use crate::scoring::{self, BrierReport, PitRecal, QuantileForecast, ReliabilityBin};
use crate::site::SitePreset;
use serde::{Deserialize, Serialize};

/// 가속 목표 평균 수명 (일)
pub const TARGET_LIFE_DAYS: f64 = 300.0;

/// 저장소에 넣어 둔 예측 점수 보고서 (`crane_demo prognostics` 기본 설정 결과)
pub const SAVED_REPORT_JSON: &str = include_str!("../data/reference/prognostics_report.json");

pub fn saved_report() -> Option<PrognosticsReport> {
    serde_json::from_str(SAVED_REPORT_JSON).ok()
}
pub const MAX_ACCEL: f64 = 50.0;
/// v3 퍼짐 보정 배수 격자
pub const SPREAD_GRID: [f64; 12] = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.4, 1.6, 1.8, 2.2, 2.6];
/// α 정확도 (Saxena: 예측이 참 남은 수명의 ±α 안)
pub const ALPHA_ACC: f64 = 0.2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompSeries {
    pub id: String,
    pub kind: ComponentKind,
    pub label: String,
    pub doses: Vec<f64>,
    /// 소비 가속 배수 (1 = 실제 시간)
    pub accel: f64,
    /// 관측 주기 (일, 0 = 관측 없음)
    pub inspect_every: u32,
    pub params: KindParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoseSeries {
    pub site: String,
    pub days: u32,
    pub comps: Vec<CompSeries>,
}

/// 현장 운영을 `days` 일 돌려 부품별 하루 물리 소비를 모은다 (결정은 권장, 사고 뽑기 없음)
pub fn collect(site: &SitePreset, days: u32, seed: u64) -> Result<DoseSeries, String> {
    let cfg = OperationConfig { asset_id: format!("bt-{}", site.id), days, start_month: Some(1), start_day: Some(1), seed, policy: DecisionPolicy::Recommended, incidents: false, fresh: true, stochastic: false };
    let mut sim = OpSim::new(site.clone(), cfg, None)?;
    let spec = site.spec()?;
    let prot = site.environment.protection()?;
    let comps: Vec<(String, ComponentKind, String, Option<Axis>)> = sim.ledger().components.iter().map(|c| (c.id.clone(), c.kind, c.label.clone(), c.axis)).collect();
    let mut doses: Vec<Vec<f64>> = vec![Vec::with_capacity(days as usize); comps.len()];
    for _ in 0..days {
        let _ = sim.run(24);
        let ld = &sim.ledger().last_dose;
        for (k, (id, ..)) in comps.iter().enumerate() {
            doses[k].push(ld.iter().find(|x| &x.0 == id).map(|x| x.1).unwrap_or(0.0));
        }
    }
    let periodic = site.operations.periodic_inspection_days.max(1);
    let out = comps
        .into_iter()
        .zip(doses)
        .map(|((id, kind, label, axis), d)| {
            let usable = axis.map(|a| (spec.drive(a).brake.lining_new_mm - spec.drive(a).brake.lining_min_mm).max(1.0)).unwrap_or(10.0);
            let params = degradation::params(kind, usable, prot.access_hdg_um);
            let mean = d.iter().sum::<f64>() / d.len().max(1) as f64;
            let life = if mean > 0.0 { 1.0 / mean } else { f64::INFINITY };
            let accel = (life / TARGET_LIFE_DAYS).clamp(1.0, MAX_ACCEL);
            let inspect_every = match kind {
                ComponentKind::Coating => prot.coating_inspection_days.max(30.0) as u32,
                ComponentKind::RailClamp if prot.clamp_test_days > 0.0 => prot.clamp_test_days as u32,
                ComponentKind::Walkway => 365,
                _ if matches!(params.indicator, degradation::Indicator::None) => 0,
                _ => periodic,
            };
            CompSeries { id, kind, label, doses: d, accel, inspect_every, params }
        })
        .collect();
    Ok(DoseSeries { site: site.id.clone(), days, comps: out })
}

/// 예측 기록 하나
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rec {
    pub site: String,
    pub comp: String,
    pub kind: ComponentKind,
    pub method: Method,
    pub origin: f64,
    pub q: QuantileForecast,
    pub p30: f64,
    pub l_now: [f64; 3],
    pub truth_now: f64,
    /// 참 남은 수명 (수명 끝을 못 보면 None)
    pub true_rul: Option<f64>,
    /// 30일 안에 수명 끝인가 (모르면 None)
    pub fail30: Option<bool>,
    pub n_obs: usize,
}

fn hash3(a: &str, b: &str, r: u32) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for x in a.bytes().chain([0u8]).chain(b.bytes()) {
        h ^= x as u64;
        h = h.wrapping_mul(0x0100_0000_01B3);
    }
    h ^ (r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

/// 예측을 시작하기 전 현장 이력을 모으는 기간 (일) — 실제 원장도 운전 이력이 쌓인 뒤 계절 투영을 쓴다
pub const WARMUP_DAYS: u32 = 365;

/// 백테스트: 복제마다 참 경로 · 관측을 만들고 `origin_every` 일마다 네 방식으로 예측한다 (적응 기간 뒤부터)
pub fn backtest(series: &[DoseSeries], replicas: u32, origin_every: u32, seed: u64) -> Vec<Rec> {
    backtest_from(series, replicas, origin_every, seed, WARMUP_DAYS)
}

pub fn backtest_from(series: &[DoseSeries], replicas: u32, origin_every: u32, seed: u64, warmup: u32) -> Vec<Rec> {
    let mut out: Vec<Rec> = Vec::new();
    let calib = Calib { spread: 1.0, rate_window: 365, recal: None };
    for s in series {
        for c in s.comps.iter() {
            for r in 0..replicas {
                let mut rng = Rng::new(seed ^ hash3(&s.site, &c.id, r));
                let mut rng_fc = rng.fork(0x51);
                let mut unit = degradation::Unit::new(&c.params, 0.0, &mut rng);
                let mut pending: Vec<usize> = Vec::new();
                let mut hist: Vec<f64> = Vec::with_capacity(s.days as usize);
                for day in 0..s.days {
                    let dayf = day as f64 + 1.0;
                    let d = c.doses.get(day as usize).cloned().unwrap_or(0.0) * c.accel;
                    unit.advance(&c.params, d, &mut rng);
                    hist.push(d);
                    let age = (dayf - unit.installed_day).round() as u32;
                    if c.inspect_every > 0 && age > 0 && age % c.inspect_every == 0 {
                        unit.observe(&c.params, dayf, &mut rng);
                    }
                    if unit.truth >= 1.0 {
                        for &k in pending.iter() {
                            let rul = dayf - out[k].origin;
                            out[k].true_rul = Some(rul);
                            out[k].fail30 = Some(rul <= 30.0);
                        }
                        pending.clear();
                        unit = degradation::Unit::new(&c.params, dayf, &mut rng);
                        continue;
                    }
                    if day + 1 >= warmup && (day + 1) % origin_every == 0 && unit.dose > 0.0 {
                        for m in Method::all() {
                            let p = degradation::forecast_with(m, &c.params, &unit, dayf, &calib, &mut rng_fc, &hist);
                            out.push(Rec {
                                site: s.site.clone(),
                                comp: c.id.clone(),
                                kind: c.kind,
                                method: m,
                                origin: dayf,
                                p30: p.p_fail[1],
                                q: p.rul,
                                l_now: p.l_now,
                                truth_now: unit.truth,
                                true_rul: None,
                                fail30: None,
                                n_obs: p.n_obs,
                            });
                            pending.push(out.len() - 1);
                        }
                    }
                }
                // 중도절단 없애기: 소비 시계열이 끝나면 같은 시계열을 이어 붙여 참 경로를 수명 끝까지 진행한다
                // (기간 안에 끝난 개체만 채점하면 오래 버틴 개체가 빠져 PIT 가 0 쪽으로 몰린다)
                if !pending.is_empty() && !c.doses.is_empty() {
                    let n = c.doses.len();
                    let mut day = s.days as usize;
                    let end = s.days as usize + degradation::RUL_CAP_DAYS as usize;
                    while day < end {
                        let d = c.doses[day % n] * c.accel;
                        unit.advance(&c.params, d, &mut rng);
                        day += 1;
                        if unit.truth >= 1.0 {
                            break;
                        }
                    }
                    let eol = day as f64;
                    let reached = unit.truth >= 1.0;
                    for &k in pending.iter() {
                        if reached {
                            let rul = eol - out[k].origin;
                            out[k].true_rul = Some(rul);
                            out[k].fail30 = Some(rul <= 30.0);
                        } else if out[k].origin + 30.0 <= eol {
                            out[k].fail30 = Some(false);
                        }
                    }
                }
            }
        }
    }
    out
}

/// 로그 RUL 중앙값 둘레로 분위수 퍼짐을 배수만큼 바꾼다 (v3 보정)
pub fn scale_spread(q: &QuantileForecast, s: f64) -> QuantileForecast {
    if (s - 1.0).abs() < 1e-12 {
        return q.clone();
    }
    let med = q.median().max(0.5);
    let qs = q.qs.iter().map(|x| (med * (s * (x.max(0.5) / med).ln()).exp()).min(degradation::RUL_CAP_DAYS)).collect();
    QuantileForecast::new(&q.taus, qs)
}

/// 방식 하나의 점수
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MethodScore {
    pub method: String,
    pub label: String,
    pub spread: f64,
    /// 남은 수명 점수 (수명 끝을 본 예측만)
    pub n_rul: usize,
    pub crps: f64,
    pub pinball: f64,
    pub mae_median: f64,
    pub coverage80: f64,
    pub coverage50: f64,
    pub width80: f64,
    pub interval80: f64,
    pub pit_ks: f64,
    pub pit_hist: Vec<f64>,
    /// α 정확도: 중앙값이 참 남은 수명 ±20 % 안인 비율
    pub alpha_acc: f64,
    /// 30일 고장 확률 Brier · 분해
    pub brier: BrierReport,
    /// 지금 상태 80 % 구간에 참 상태가 드는 비율
    pub nowcast_cov80: f64,
    /// 건강 점수(중앙) 분포 (10칸, 0–100)
    pub health_hist: Vec<f64>,
}

pub fn score(recs: &[&Rec], method: Method, spread: f64) -> MethodScore {
    let mine: Vec<&&Rec> = recs.iter().filter(|r| r.method == method).collect();
    let (mut fc, mut ys) = (Vec::new(), Vec::new());
    let mut alpha_hits = 0usize;
    for r in mine.iter() {
        if let Some(y) = r.true_rul {
            let q = scale_spread(&r.q, spread);
            if (q.median() - y).abs() <= ALPHA_ACC * y.max(1.0) {
                alpha_hits += 1;
            }
            fc.push(q);
            ys.push(y);
        }
    }
    let qs = scoring::score_quantiles(&fc, &ys);
    let (mut p, mut o) = (Vec::new(), Vec::new());
    let (mut cov_now, mut n_now) = (0usize, 0usize);
    let mut health = vec![0.0; 10];
    for r in mine.iter() {
        if let Some(f) = r.fail30 {
            let p30 = if (spread - 1.0).abs() < 1e-12 { r.p30 } else { scale_spread(&r.q, spread).cdf(30.0) };
            p.push(p30.clamp(0.0, 1.0));
            o.push(f);
        }
        n_now += 1;
        if r.truth_now >= r.l_now[0] - 1e-9 && r.truth_now <= r.l_now[2] + 1e-9 {
            cov_now += 1;
        }
        let h = 100.0 * (1.0 - r.l_now[1]).clamp(0.0, 1.0);
        health[((h / 10.0) as usize).min(9)] += 1.0;
    }
    let tot: f64 = health.iter().sum::<f64>().max(1.0);
    MethodScore {
        method: method.key().into(),
        label: method.label().into(),
        spread,
        n_rul: qs.n,
        crps: qs.crps,
        pinball: qs.pinball,
        mae_median: qs.mae_median,
        coverage80: qs.coverage80,
        coverage50: qs.coverage50,
        width80: qs.width80,
        interval80: qs.interval80,
        pit_ks: qs.pit_ks,
        pit_hist: qs.pit_hist,
        alpha_acc: if qs.n > 0 { alpha_hits as f64 / qs.n as f64 } else { 0.0 },
        brier: scoring::brier(&p, &o, 10),
        nowcast_cov80: if n_now > 0 { cov_now as f64 / n_now as f64 } else { 0.0 },
        health_hist: health.iter().map(|x| x / tot).collect(),
    }
}

/// 퍼짐 배수를 CRPS 최소로 고른다
pub fn fit_spread(recs: &[&Rec]) -> f64 {
    let mut best = (1.0, f64::INFINITY);
    for s in SPREAD_GRID {
        let sc = score(recs, Method::V3Calibrated, s);
        if sc.n_rul > 0 && sc.crps < best.1 {
            best = (s, sc.crps);
        }
    }
    best.0
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fold {
    pub held_out: String,
    pub spread: f64,
    pub score: MethodScore,
    /// v4 (뺀 현장 밖에서 맞춘 부품별 PIT 재보정을 뺀 현장에 적용)
    #[serde(default)]
    pub recal: Option<MethodScore>,
}

/// PIT 재보정을 줄이는 표본 수 (기록 n 개면 가중 n / (n + 이 값)) — 가정
pub const RECAL_SHRINK_N: f64 = 100.0;

/// 기록들의 부품 종류별 PIT 재보정 (참 남은 수명을 아는 기록만)
pub fn fit_recal(recs: &[&Rec]) -> Vec<(String, PitRecal)> {
    let mut kinds: Vec<ComponentKind> = Vec::new();
    for r in recs.iter() {
        if !kinds.contains(&r.kind) {
            kinds.push(r.kind);
        }
    }
    kinds
        .into_iter()
        .map(|k| {
            let pits: Vec<f64> = recs.iter().filter(|r| r.kind == k).filter_map(|r| r.true_rul.map(|y| r.q.cdf(y))).collect();
            (kind_key(k), PitRecal::fit(&pits, RECAL_SHRINK_N))
        })
        .collect()
}

/// v3 기록 → v4 기록 (종류별 재보정 적용)
fn recal_rec(r: &Rec, table: &[(String, PitRecal)]) -> Rec {
    let mut x = r.clone();
    x.method = Method::V4Recal;
    if let Some((_, rc)) = table.iter().find(|(k, _)| *k == kind_key(r.kind)) {
        x.p30 = rc.cdf(r.q.cdf(30.0));
        x.q = rc.apply(&r.q, 0.0);
    }
    x
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KindScore {
    pub kind: String,
    pub accel: f64,
    pub methods: Vec<MethodScore>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrognosticsReport {
    pub sites: Vec<String>,
    pub days: u32,
    pub replicas: u32,
    pub origin_every: u32,
    pub n_records: usize,
    /// 전체: v0 · v1 · v2 · v3(현장 하나 빼기 보정)
    pub overall: Vec<MethodScore>,
    pub by_kind: Vec<KindScore>,
    pub folds: Vec<Fold>,
    /// 모든 현장으로 맞춘 퍼짐 배수 (현장 운영의 v3 에 쓴다)
    pub spread_all: f64,
    /// v3 30일 고장 확률 신뢰도 표 (v4 를 쓰면 v4)
    pub reliability: Vec<ReliabilityBin>,
    pub notes: Vec<String>,
    /// 모든 현장 v3 기록(퍼짐 `spread_all`)으로 맞춘 부품 종류별 PIT 재보정 — 현장 운영의 v4 에 쓴다
    #[serde(default)]
    pub recal: Vec<(String, PitRecal)>,
    /// v4 가 현장 하나 빼기에서 v3 보다 나아 채택했는가 (CRPS 가 1 % 넘게 나빠지지 않고 PIT KS 가 줄면)
    #[serde(default)]
    pub recal_adopted: bool,
}

pub fn kind_key(k: ComponentKind) -> String {
    serde_json::to_value(k).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default()
}

/// 기록 묶음을 채점하고 v3 를 현장 하나 빼기로 교차검증한다
pub fn evaluate(series: &[DoseSeries], recs: &[Rec], replicas: u32, origin_every: u32) -> PrognosticsReport {
    let all: Vec<&Rec> = recs.iter().collect();
    let sites: Vec<String> = series.iter().map(|s| s.site.clone()).collect();
    let mut folds = Vec::new();
    let mut held_recs: Vec<Rec> = Vec::new();
    let mut held_v4: Vec<Rec> = Vec::new();
    let scaled = |r: &Rec, sp: f64| {
        let mut x = r.clone();
        x.q = scale_spread(&r.q, sp);
        x.p30 = x.q.cdf(30.0);
        x
    };
    for s in sites.iter() {
        let train: Vec<&Rec> = recs.iter().filter(|r| &r.site != s).collect();
        let test: Vec<&Rec> = recs.iter().filter(|r| &r.site == s).collect();
        let sp = if train.is_empty() { 1.0 } else { fit_spread(&train) };
        // v4: 뺀 현장 밖의 (퍼짐 보정한) v3 기록으로 종류별 PIT 재보정을 맞춰 뺀 현장에 적용
        let train_v3: Vec<Rec> = train.iter().filter(|r| r.method == Method::V3Calibrated).map(|r| scaled(r, sp)).collect();
        let table = fit_recal(&train_v3.iter().collect::<Vec<_>>());
        let test_v3: Vec<Rec> = test.iter().filter(|r| r.method == Method::V3Calibrated).map(|r| scaled(r, sp)).collect();
        let test_v4: Vec<Rec> = test_v3.iter().map(|r| recal_rec(r, &table)).collect();
        let recal_score = score(&test_v4.iter().collect::<Vec<_>>(), Method::V4Recal, 1.0);
        folds.push(Fold { held_out: s.clone(), spread: sp, score: score(&test, Method::V3Calibrated, sp), recal: Some(recal_score) });
        // 접힘마다 보정한 v3 · v4 를 하나로 모아 전체 점수를 낸다
        held_recs.extend(test_v3);
        held_v4.extend(test_v4);
    }
    let held: Vec<&Rec> = held_recs.iter().collect();
    let held4: Vec<&Rec> = held_v4.iter().collect();
    let mut overall: Vec<MethodScore> = [Method::V0Physics, Method::V1Holt, Method::V2Bayes].iter().map(|m| score(&all, *m, 1.0)).collect();
    let mut v3 = score(&held, Method::V3Calibrated, 1.0);
    v3.spread = folds.iter().map(|f| f.spread).sum::<f64>() / folds.len().max(1) as f64;
    overall.push(v3.clone());
    let mut v4 = score(&held4, Method::V4Recal, 1.0);
    v4.spread = v3.spread;
    let recal_adopted = v4.n_rul > 0 && v4.crps <= v3.crps * 1.01 && v4.pit_ks < v3.pit_ks;
    overall.push(v4.clone());
    let spread_all = fit_spread(&all);
    let all_v3: Vec<Rec> = recs.iter().filter(|r| r.method == Method::V3Calibrated).map(|r| scaled(r, spread_all)).collect();
    let recal_all = fit_recal(&all_v3.iter().collect::<Vec<_>>());
    // 부품 종류별
    let mut kinds: Vec<ComponentKind> = Vec::new();
    for r in recs.iter() {
        if !kinds.contains(&r.kind) {
            kinds.push(r.kind);
        }
    }
    let by_kind = kinds
        .into_iter()
        .map(|k| {
            let sub: Vec<&Rec> = recs.iter().filter(|r| r.kind == k).collect();
            let sub_h: Vec<&Rec> = held_recs.iter().filter(|r| r.kind == k).collect();
            let sub_4: Vec<&Rec> = held_v4.iter().filter(|r| r.kind == k).collect();
            let accel = series.iter().flat_map(|s| s.comps.iter()).filter(|c| c.kind == k).map(|c| c.accel).fold(1.0, f64::max);
            let mut ms: Vec<MethodScore> = [Method::V0Physics, Method::V1Holt, Method::V2Bayes].iter().map(|m| score(&sub, *m, 1.0)).collect();
            ms.push(score(&sub_h, Method::V3Calibrated, 1.0));
            ms.push(score(&sub_4, Method::V4Recal, 1.0));
            KindScore { kind: kind_key(k), accel, methods: ms }
        })
        .collect();
    let notes = vec![
        format!("복제 {replicas} · 적응 {WARMUP_DAYS}일 뒤 예측 시점 {origin_every}일마다 · 기록 {}건 (방식 4개)", recs.len()),
        "수명이 긴 부품은 소비를 가속했다 (부품별 가속 배수 — 실제 시간이 아니라 열화 진행 기준의 점수)".into(),
        "기간 끝에서 수명이 남은 개체는 같은 소비 시계열을 이어 붙여 수명 끝까지 진행했다 (중도절단 편향 제거)".into(),
        "v3 퍼짐 배수는 현장 하나를 빼고 나머지에서 CRPS 최소로 맞춘 뒤 뺀 현장에서 쟀다".into(),
        format!(
            "v4 부품별 PIT 재보정(Kuleshov 2018, 표본 {RECAL_SHRINK_N:.0} 개 기준으로 항등 쪽으로 줄임)도 현장 하나 빼기로 쟀다: CRPS {:.1} → {:.1} · PIT KS {:.3} → {:.3} · 80 % 적중 {:.2} → {:.2} — {}",
            v3.crps,
            v4.crps,
            v3.pit_ks,
            v4.pit_ks,
            v3.coverage80,
            v4.coverage80,
            if recal_adopted { "채택 (현장 운영이 v4 를 쓴다)" } else { "채택하지 않음 (v3 유지)" }
        ),
        "참 상태는 시뮬레이션 안에서만 안다 — 현장 운영 화면의 건강 점수는 관측으로 낸 추정이다".into(),
    ];
    PrognosticsReport {
        sites,
        days: series.first().map(|s| s.days).unwrap_or(0),
        replicas,
        origin_every,
        n_records: recs.len(),
        reliability: if recal_adopted { v4.brier.bins.clone() } else { v3.brier.bins.clone() },
        overall,
        by_kind,
        folds,
        spread_all,
        notes,
        recal: recal_all,
        recal_adopted,
    }
}

/// 수집 → 백테스트 → 채점 한 번에
pub fn run(sites: &[SitePreset], days: u32, replicas: u32, origin_every: u32, seed: u64) -> Result<PrognosticsReport, String> {
    let mut series = Vec::new();
    for (k, s) in sites.iter().enumerate() {
        series.push(collect(s, days, seed.wrapping_add(k as u64 * 7919))?);
    }
    let recs = backtest(&series, replicas, origin_every, seed);
    Ok(evaluate(&series, &recs, replicas, origin_every))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth(site: &str, days: u32, rate: f64) -> DoseSeries {
        let mut comps = Vec::new();
        for (id, kind) in [("rope_hoist", ComponentKind::Rope), ("brake_hoist", ComponentKind::Brake), ("wheel_travel", ComponentKind::Wheel)] {
            let mut d = Vec::new();
            for k in 0..days {
                // 주중 · 주말 차이가 있는 소비
                d.push(if k % 7 < 5 { rate * 1.2 } else { rate * 0.5 });
            }
            comps.push(CompSeries { id: id.into(), kind, label: id.into(), doses: d, accel: 1.0, inspect_every: 30, params: degradation::params(kind, 10.0, 85.0) });
        }
        DoseSeries { site: site.into(), days, comps }
    }

    #[test]
    fn bayes_beats_physics_and_calibration_helps() {
        let series = vec![synth("a", 900, 1.0 / 280.0), synth("b", 900, 1.0 / 250.0), synth("c", 900, 1.0 / 320.0)];
        let recs = backtest_from(&series, 6, 30, 3, 0);
        let rep = evaluate(&series, &recs, 6, 30);
        let get = |k: &str| rep.overall.iter().find(|m| m.method == k).unwrap().clone();
        let (v0, v2, v3) = (get("v0"), get("v2"), get("v3"));
        assert!(v0.n_rul > 50, "{}", v0.n_rul);
        // 개체 차이를 배우는 v2 가 점 예측 v0 보다 CRPS 가 낮다
        assert!(v2.crps < v0.crps, "v2 {} v0 {}", v2.crps, v0.crps);
        // 보정한 v3 는 80 % 구간 적중이 목표 근처
        assert!(v3.coverage80 > 0.6 && v3.coverage80 < 0.95, "{}", v3.coverage80);
        // 30일 고장 확률은 기후값보다 낫다
        assert!(v3.brier.skill > 0.0, "{:?}", v3.brier);
        assert_eq!(rep.folds.len(), 3);
    }
}

#[cfg(test)]
mod probe {
    use super::*;

    #[test]
    #[ignore]
    fn prior_only_pit() {
        for (name, rate, jitter) in [("flat", 1.0 / 280.0, 0.0), ("seasonal", 1.0 / 280.0, 0.8)] {
            let mut comps = Vec::new();
            let mut d = Vec::new();
            for k in 0..730u32 {
                let season = 1.0 + jitter * (2.0 * std::f64::consts::PI * k as f64 / 365.0).sin();
                d.push(rate * season);
            }
            comps.push(CompSeries { id: "walkway".into(), kind: ComponentKind::Walkway, label: "w".into(), doses: d, accel: 1.0, inspect_every: 0, params: degradation::params(ComponentKind::Walkway, 10.0, 85.0) });
            let series = vec![DoseSeries { site: name.into(), days: 730, comps }];
            let recs = backtest(&series, 40, 30, 9);
            eprintln!("records {}", recs.len());
            let all: Vec<&Rec> = recs.iter().collect();
            for m in [Method::V2Bayes, Method::V3Calibrated] {
                let sc = score(&all, m, 1.0);
                eprintln!("{name} {:?} n={} cov80={:.2} ks={:.3} pit={:?}", m, sc.n_rul, sc.coverage80, sc.pit_ks, sc.pit_hist.iter().map(|x| (x * 100.0).round() / 100.0).collect::<Vec<_>>());
            }
        }
    }
}
