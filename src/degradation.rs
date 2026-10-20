//! 부품 열화 시계열 · 관측 · 예측 — 원장의 물리 증분(하루 수명 소비)에 확률 열화와 개체 차이를 얹은 '참 상태',
//! 점검 · 센서로 보는 '관측 지표', 그 관측으로 남은 수명(RUL) 분포 · 기간 내 고장 확률 · 건강 점수를 내는 '예측'.
//!
//! 참 상태 ℓ(t) = u · X(D(t))
//! - D(t): 원장 물리 모델의 누적 수명 소비 (Feyrer · mayr · IEC 60085 · ISO 281 · ISO 4301 · ISO 9223/9224 …)
//! - X: 감마 과정, 증분 X(D+ΔD) − X(D) ~ Ga(모양 αΔD, 율 α) → E = ΔD, 수명 끝 변동 계수 1/√α
//!   (van Noortwijk 2009 · Strauss & Demissie 2012: E = v/u, Var = v/u², CoV = 1/√v)
//! - u: 개체 차이 (로그정규, 평균 1) — 수명 변동 계수는 부품별 와이블 형상에서 (Barringer/Bloch & Geitner:
//!   베어링 β 1.3, 기어 2, 교류 모터 1.2, 고체 제어기 0.7; NASA TP-2013-215305: 기어박스 현장 β 2.189)
//!
//! 관측 지표 z = g(ℓ) + 잡음 (부품별): 로프 단선 수 (ISO 4309 폐기 개수), 라이닝 마모 (mm), 진동 (mm/s),
//! 차륜 마모 (mm), 훅 입 벌어짐 (%), 도장 녹 면적 (%), 클램프 유지력 저하 (%), 아연 두께 (µm).
//!
//! 예측 판 (같은 백테스트 · 같은 점수로 비교, `prognostics`):
//! - v0 물리: 원장 방식 — 하루 소비율로 남은 날 (점 예측)
//! - v1 Holt: 관측 지표를 수명 비율로 되돌린 시계열에 기존 `timeseries::forecast`(Holt 감쇠 추세, 정규 구간)
//! - v2 감마 베이즈: 관측으로 개체 배수 u 를 베이즈 갱신(로그 정규 근사) → 남은 소비량의 감마 과정 첫 통과 → 몬테카를로 분위수
//! - v3 보정: v2 + 최근 1년 소비율로 미래 투영 + 로그 RUL 퍼짐 보정 배수(등각 방식, 다른 현장에서 맞춤)

use crate::health::ComponentKind;
use crate::rng::Rng;
use crate::scoring::{uniform_taus, QuantileForecast};
use serde::{Deserialize, Serialize};

/// 분위수 격자 개수 (τ = 0.025 … 0.975)
pub const N_TAUS: usize = 20;
/// 남은 수명 몬테카를로 표본 수
pub const MC_SAMPLES: usize = 400;
/// 예측 상한 (일) — 넘으면 이 값으로 자른다
pub const RUL_CAP_DAYS: f64 = 3650.0;

/// 관측 지표
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Indicator {
    /// 로프 단선 수 (30d 길이): g = limit·(e^{kℓ}−1)/(e^k−1), 육안 검출률 detect
    BrokenWires { limit: f64, k: f64, detect: f64 },
    /// 선형 지표 (마모 mm · 입 벌어짐 % · 유지력 저하 % · 아연 소모 µm): g = span·ℓ, 정규 잡음 sd
    Linear { span: f64, sd: f64 },
    /// 진동 속도 (mm/s): g = v0·(v_lim/v0)^{ℓ²}, 로그 정규 잡음 sd_log
    Vibration { v0: f64, v_lim: f64, sd_log: f64 },
    /// 도장 녹 면적 (%): g = 1 %·exp(k(ℓ−1)), k = ln 8 / 0.25 (Ri3 1 % · Ri4 8 %), 로그 정규 잡음
    RustArea { sd_log: f64 },
    /// 직접 보는 지표 없음 (거더 피로 · 전장)
    None,
}

impl Indicator {
    pub fn name(&self) -> &'static str {
        match self {
            Indicator::BrokenWires { .. } => "단선 수 (30d)",
            Indicator::Linear { .. } => "마모 · 저하",
            Indicator::Vibration { .. } => "진동 (mm/s)",
            Indicator::RustArea { .. } => "녹 면적 (%)",
            Indicator::None => "없음",
        }
    }

    /// 참 상태 → 기대 지표
    pub fn expected(&self, l: f64) -> f64 {
        let l = l.max(0.0);
        match *self {
            Indicator::BrokenWires { limit, k, .. } => limit * ((k * l).exp() - 1.0) / (k.exp() - 1.0),
            Indicator::Linear { span, .. } => span * l,
            Indicator::Vibration { v0, v_lim, .. } => v0 * (v_lim / v0).powf(l * l),
            Indicator::RustArea { .. } => (8.0f64.ln() / 0.25 * (l - 1.0)).exp(),
            Indicator::None => 0.0,
        }
    }

    /// 관측 하나를 뽑는다
    pub fn sample(&self, l: f64, rng: &mut Rng) -> f64 {
        let g = self.expected(l);
        match *self {
            Indicator::BrokenWires { detect, .. } => rng.poisson(detect * g) as f64 / detect.max(1e-6),
            Indicator::Linear { sd, .. } => g + sd * rng.normal(),
            Indicator::Vibration { sd_log, .. } | Indicator::RustArea { sd_log } => g * (sd_log * rng.normal() - 0.5 * sd_log * sd_log).exp(),
            Indicator::None => 0.0,
        }
    }

    /// 지표 → (수명 비율 추정, 분산). 정보가 없으면 None (예: 녹 0.01 % 아래, 단선 0개는 '하한' 정보만)
    pub fn invert(&self, z: f64) -> Option<(f64, f64)> {
        match *self {
            Indicator::BrokenWires { limit, k, detect } => {
                let a = (k.exp() - 1.0) / limit;
                let zz = z.max(0.0);
                let l = (1.0 + zz * a).ln() / k;
                // 포아송 분산 (평균 ≥ 0.5 개로 바닥) → 델타법
                let var_z = (zz.max(0.5)) / detect.max(1e-6);
                let dl = a / (k * (1.0 + zz * a));
                Some((l, (dl * dl * var_z).max(1e-4)))
            }
            Indicator::Linear { span, sd } => Some((z / span.max(1e-9), (sd / span.max(1e-9)).powi(2).max(1e-6))),
            Indicator::Vibration { v0, v_lim, sd_log } => {
                let r = (v_lim / v0).ln().max(1e-6);
                let q = ((z.max(1e-9) / v0).ln() / r).max(0.0);
                let l = q.sqrt();
                let var_q = (sd_log / r).powi(2);
                Some((l, (var_q / (4.0 * l.max(0.2).powi(2))).max(1e-4)))
            }
            Indicator::RustArea { sd_log } => {
                if z < 0.01 {
                    return None;
                }
                let k = 8.0f64.ln() / 0.25;
                Some((1.0 + z.ln() / k, (sd_log / k).powi(2).max(1e-5)))
            }
            Indicator::None => None,
        }
    }
}

/// 부품 종류별 열화 · 관측 매개변수
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindParams {
    /// 개체 차이: 수명 변동 계수 (로그 정규 u)
    pub unit_cv: f64,
    /// 감마 과정 수명 끝 변동 계수 1/√α
    pub gamma_cv: f64,
    pub indicator: Indicator,
    /// 지표 단위
    pub unit: String,
    pub source: String,
}

/// 와이블 형상 β 의 수명 변동 계수 √(Γ(1+2/β) − Γ(1+1/β)²) / Γ(1+1/β)
pub fn weibull_cv(beta: f64) -> f64 {
    let g1 = crate::rng::gamma(1.0 + 1.0 / beta);
    let g2 = crate::rng::gamma(1.0 + 2.0 / beta);
    (g2 - g1 * g1).max(0.0).sqrt() / g1
}

/// 기본 매개변수. `span` 들은 크레인 제원에서 정한다 (라이닝 가용 두께 등)
pub fn params(kind: ComponentKind, lining_usable_mm: f64, access_hdg_um: f64) -> KindParams {
    // 감마 과정 잡음: 레이저 열화 예(arXiv 2212.02688)에서 1000–4000 h 변동 계수 0.18–0.09 → 0.15 (가정에 가까운 근거)
    let gcv = 0.15;
    let p = |unit_cv: f64, ind: Indicator, unit: &str, src: &str| KindParams { unit_cv, gamma_cv: gcv, indicator: ind, unit: unit.into(), source: src.into() };
    match kind {
        ComponentKind::Rope => p(
            0.3,
            Indicator::BrokenWires { limit: 8.0, k: 4.0, detect: 0.8 },
            "개",
            "ISO 4309:2010 RCN 3 보통 꼬임 30d 폐기 8개 (CERTEX 재현), 끝으로 갈수록 빨라지는 단선 증가 k = 4 · 개체 변동 0.3 · 검출률 0.8 은 가정",
        ),
        ComponentKind::Brake => p(0.25, Indicator::Linear { span: lining_usable_mm.max(1.0), sd: 0.2 }, "mm", "라이닝 가용 두께(제원)까지 선형 마모, 측정 ±0.2 mm · 개체 변동 0.25 는 가정 (공개 마모 산포 자료 없음)"),
        ComponentKind::Motor => p(weibull_cv(1.2), Indicator::Vibration { v0: 1.0, v_lim: 4.5, sd_log: 0.15 }, "mm/s", "교류 모터 와이블 β 1.2 (Barringer · Bloch & Geitner), 진동 4.5 mm/s 경보 (ISO 10816-3 그룹 2 경계, 미확인)"),
        ComponentKind::Gearbox => p(weibull_cv(2.189), Indicator::Vibration { v0: 1.4, v_lim: 7.1, sd_log: 0.15 }, "mm/s", "기어박스 현장 와이블 β 2.189 (NASA TP-2013-215305), 진동 7.1 mm/s 경보 (미확인)"),
        ComponentKind::Wheel => p(0.14, Indicator::Linear { span: 19.0, sd: 0.3 }, "mm", "철도 차륜 마모 수명 42–56만 km 의 산포 (arXiv 2212.02688, 3개 차륜) · Ø630 의 3 % 폐기 19 mm (가정)"),
        ComponentKind::Hook => p(0.3, Indicator::Linear { span: 5.0, sd: 0.3 }, "%", "훅 입 벌어짐 5 % 폐기 (ASME B30.10), 개체 변동 0.3 은 가정"),
        ComponentKind::Coating => p(0.2, Indicator::RustArea { sd_log: 0.3 }, "%", "ISO 12944 내구성 범위 폭(±25 %)에서 변동 0.2, 녹 면적 ISO 4628-3 (Ri3 1 %)"),
        ComponentKind::RailClamp => p(0.3, Indicator::Linear { span: 30.0, sd: 3.0 }, "%", "클램프 유지력 저하 30 % 폐기 · 시험 ±3 % 는 가정"),
        ComponentKind::Girder => p(0.5, Indicator::None, "", "용접부 피로 S-N 산포 (가정 0.5), 직접 보는 지표 없음 (법정 검사 균열 검사만)"),
        ComponentKind::Electrical => p(weibull_cv(0.7), Indicator::None, "", "고체 제어기 와이블 β 0.7 (Barringer), 지표 없음"),
        ComponentKind::Walkway => p(0.2, Indicator::Linear { span: access_hdg_um.max(1.0), sd: 5.0 }, "µm", "아연 두께 측정 ±5 µm (가정), 용융아연 두께(방지 프리셋)"),
    }
}

/// 관측 하나
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obs {
    pub day: f64,
    /// 그때까지 물리 누적 소비
    pub dose: f64,
    pub z: f64,
    /// 되돌린 수명 비율과 분산 (정보 없으면 None)
    pub l_hat: Option<f64>,
    pub var: f64,
}

/// 부품 한 개(설치부터 교체까지)의 상태
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Unit {
    pub installed_day: f64,
    /// 개체 배수 u (숨김)
    pub u: f64,
    /// 참 수명 비율 (숨김) · 물리 누적 소비
    pub truth: f64,
    pub dose: f64,
    /// 하루 물리 소비 기록 (최근 400일)
    pub daily: Vec<f64>,
    pub obs: Vec<Obs>,
}

impl Unit {
    pub fn new(kp: &KindParams, day: f64, rng: &mut Rng) -> Self {
        let s = (1.0 + kp.unit_cv * kp.unit_cv).ln().sqrt();
        let u = (s * rng.normal() - 0.5 * s * s).exp();
        Unit { installed_day: day, u, ..Default::default() }
    }

    /// 하루 진행: 물리 소비 d → 참 상태 증분 u · Ga(αd, α)
    pub fn advance(&mut self, kp: &KindParams, d: f64, rng: &mut Rng) {
        let d = d.max(0.0);
        self.dose += d;
        self.daily.push(d);
        if self.daily.len() > 400 {
            self.daily.remove(0);
        }
        if d > 0.0 {
            let alpha = 1.0 / (kp.gamma_cv * kp.gamma_cv).max(1e-9);
            self.truth += self.u * rng.gamma(alpha * d, 1.0 / alpha);
        }
    }

    pub fn observe(&mut self, kp: &KindParams, day: f64, rng: &mut Rng) -> Option<&Obs> {
        if matches!(kp.indicator, Indicator::None) {
            return None;
        }
        let z = kp.indicator.sample(self.truth, rng);
        let inv = kp.indicator.invert(z);
        self.obs.push(Obs { day, dose: self.dose, z, l_hat: inv.map(|x| x.0), var: inv.map(|x| x.1).unwrap_or(f64::INFINITY) });
        if self.obs.len() > 120 {
            self.obs.remove(0);
        }
        self.obs.last()
    }

    /// 최근 `n` 일 평균 하루 소비 (자료가 적으면 있는 만큼)
    pub fn rate(&self, n: usize) -> f64 {
        let k = self.daily.len().min(n.max(1));
        if k == 0 {
            return 0.0;
        }
        self.daily[self.daily.len() - k..].iter().sum::<f64>() / k as f64
    }

    /// 하루 소비의 지수 평활 (원장 방식, 0.9/0.1)
    pub fn rate_ema(&self) -> f64 {
        let mut r = 0.0;
        for (k, d) in self.daily.iter().enumerate() {
            r = if k == 0 { *d } else { 0.9 * r + 0.1 * d };
        }
        r
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    V0Physics,
    V1Holt,
    V2Bayes,
    V3Calibrated,
    /// v3 + 부품 종류별 PIT 재보정 (백테스트에서는 v3 기록에서 만든다)
    V4Recal,
}

impl Method {
    pub fn all() -> [Method; 4] {
        [Method::V0Physics, Method::V1Holt, Method::V2Bayes, Method::V3Calibrated]
    }
    pub fn key(&self) -> &'static str {
        match self {
            Method::V0Physics => "v0",
            Method::V1Holt => "v1",
            Method::V2Bayes => "v2",
            Method::V3Calibrated => "v3",
            Method::V4Recal => "v4",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Method::V0Physics => "v0 물리 소비율 (원장 방식)",
            Method::V1Holt => "v1 물리 사전값 + 잔차 Holt",
            Method::V2Bayes => "v2 감마 과정 베이즈",
            Method::V3Calibrated => "v3 v2 + 생존 조건 · 계절 소비 투영 · 퍼짐 보정",
            Method::V4Recal => "v4 v3 + 부품별 PIT 재보정",
        }
    }
}

/// v3 보정값 (다른 현장 백테스트에서 맞춤)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calib {
    /// 로그 RUL 퍼짐 배수 (1 = v2 그대로)
    pub spread: f64,
    /// 미래 소비율 창 (일)
    pub rate_window: usize,
    /// v4: 부품 종류의 PIT 재보정 (없으면 v3 와 같다)
    #[serde(default)]
    pub recal: Option<crate::scoring::PitRecal>,
}

impl Default for Calib {
    fn default() -> Self {
        Self { spread: 1.0, rate_window: 365, recal: None }
    }
}

/// 예측 결과
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Prognosis {
    pub method: String,
    /// 남은 수명 분위수 (일)
    pub rul: QuantileForecast,
    /// 7 · 30 · 90 일 안 고장(교체 기준 도달) 확률
    pub p_fail: [f64; 3],
    /// 지금 수명 비율 P10 · P50 · P90
    pub l_now: [f64; 3],
    /// 건강 점수 (0–100) P10 · P50 · P90 — 100 (1 − 수명 비율)
    pub health: [f64; 3],
    pub n_obs: usize,
}

fn quantiles_of(mut v: Vec<f64>) -> QuantileForecast {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let taus = uniform_taus(N_TAUS);
    let n = v.len().max(1);
    let qs = taus.iter().map(|t| v[((t * n as f64) as usize).min(n - 1)]).collect();
    QuantileForecast::new(&taus, qs)
}

fn point_forecast(x: f64) -> QuantileForecast {
    let taus = uniform_taus(N_TAUS);
    QuantileForecast::new(&taus, vec![x; N_TAUS])
}

fn health_of(l: [f64; 3]) -> [f64; 3] {
    // 수명 비율 P90 이 건강 점수 P10 (높은 열화 = 낮은 점수)
    [100.0 * (1.0 - l[2]).clamp(0.0, 1.0), 100.0 * (1.0 - l[1]).clamp(0.0, 1.0), 100.0 * (1.0 - l[0]).clamp(0.0, 1.0)]
}

fn p_within(rul: &[f64], days: f64) -> f64 {
    if rul.is_empty() {
        return 0.0;
    }
    rul.iter().filter(|r| **r <= days).count() as f64 / rul.len() as f64
}

/// 개체 배수 u 의 사후 (로그 정규 근사): 사전 LN(−s²/2, s) × 관측.
/// 감마 과정에서 척도의 충분통계량은 누적 총량이다 (증분 합 = 마지막 수준) — 그래서 마지막 관측 비율 l/D 로 u 를 보고,
/// 분산은 측정 잡음 var/D² 과 경로 잡음 u²/(αD) 를 더한다. 측정 잡음이 큰 지표(단선 수)는 마지막 몇 개를 측정 분산 가중으로
/// 같은 누적 소비로 옮겨 평균한다
pub fn posterior_u(kp: &KindParams, unit: &Unit) -> (f64, f64) {
    let s0 = (1.0 + kp.unit_cv * kp.unit_cv).ln().sqrt();
    let prior_m = -0.5 * s0 * s0;
    let alpha = 1.0 / (kp.gamma_cv * kp.gamma_cv).max(1e-9);
    let info: Vec<&Obs> = unit.obs.iter().filter(|o| o.l_hat.is_some() && o.var.is_finite() && o.dose > 1e-6).collect();
    let Some(last) = info.last() else {
        return (prior_m, s0);
    };
    // 마지막 3개: 비율 l/D 를 측정 분산 가중 평균 (같은 누적 소비 기준으로 보는 근사)
    let tail = &info[info.len().saturating_sub(3)..];
    let (mut wsum, mut rsum) = (0.0, 0.0);
    for o in tail.iter() {
        let w = o.dose * o.dose / o.var.max(1e-9);
        wsum += w;
        rsum += w * o.l_hat.unwrap_or(0.0) / o.dose;
    }
    let u_obs = (rsum / wsum.max(1e-12)).max(0.02);
    let d = last.dose;
    let var_u = 1.0 / wsum.max(1e-12) + u_obs * u_obs / (alpha * d.max(1e-6));
    let (y, var_y) = (u_obs.ln(), var_u / (u_obs * u_obs));
    let prec = 1.0 / (s0 * s0) + 1.0 / var_y;
    ((prior_m / (s0 * s0) + y / var_y) / prec, (1.0 / prec).sqrt())
}

/// 계절 나이브: 현장의 지난 하루 소비 `hist`(오늘까지)가 1년 이상이면, 내일부터는 '몇 해 전 같은 날짜부터'의 소비를
/// 그대로 쌓는다 (해는 무작위로 골라 해마다 다른 변동을 넣는다). 1년이 안 되면 None
pub fn seasonal_days(need: f64, hist: &[f64], rng: &mut Rng) -> Option<f64> {
    let n = hist.len();
    if n < 365 {
        return None;
    }
    if need <= 0.0 {
        return Some(0.0);
    }
    let years = n / 365;
    let k = 1 + ((rng.uniform() * years as f64) as usize).min(years - 1);
    let base = n - 365 * k;
    let (mut acc, mut days) = (0.0, 0usize);
    while (days as f64) < RUL_CAP_DAYS {
        acc += hist[base + days % 365];
        days += 1;
        if acc >= need {
            return Some(days as f64);
        }
    }
    Some(RUL_CAP_DAYS)
}

/// 지난 하루 소비 `hist` 를 `block` 일 묶음으로 다시 뽑아 쌓아 누적 `need` 에 닿는 날 수
pub fn bootstrap_days(need: f64, hist: &[f64], block: usize, rng: &mut Rng) -> f64 {
    if need <= 0.0 {
        return 0.0;
    }
    let n = hist.len();
    if n == 0 || hist.iter().all(|d| *d <= 0.0) {
        return RUL_CAP_DAYS;
    }
    let b = block.clamp(1, n);
    let (mut acc, mut days) = (0.0, 0.0);
    while days < RUL_CAP_DAYS {
        let start = ((rng.uniform() * (n - b + 1) as f64) as usize).min(n - b);
        for d in hist[start..start + b].iter() {
            acc += d;
            days += 1.0;
            if acc >= need {
                return days;
            }
        }
    }
    RUL_CAP_DAYS
}

/// 예측. `site_hist` = 이 부품 자리의 지난 하루 물리 소비 전체(개체가 바뀌어도 이어진 것, 오늘까지) — 계절 투영에 쓴다
pub fn forecast(method: Method, kp: &KindParams, unit: &Unit, now_day: f64, calib: &Calib, rng: &mut Rng) -> Prognosis {
    forecast_with(method, kp, unit, now_day, calib, rng, &[])
}

pub fn forecast_with(method: Method, kp: &KindParams, unit: &Unit, now_day: f64, calib: &Calib, rng: &mut Rng, site_hist: &[f64]) -> Prognosis {
    let n_obs = unit.obs.iter().filter(|o| o.l_hat.is_some()).count();
    match method {
        Method::V0Physics => {
            let l = unit.dose;
            let r = unit.rate_ema();
            let rul = if r > 1e-12 { ((1.0 - l) / r).clamp(0.0, RUL_CAP_DAYS) } else { RUL_CAP_DAYS };
            let pf = |h: f64| if rul <= h { 1.0 } else { 0.0 };
            Prognosis { method: method.key().into(), rul: point_forecast(rul), p_fail: [pf(7.0), pf(30.0), pf(90.0)], l_now: [l; 3], health: health_of([l; 3]), n_obs }
        }
        Method::V1Holt => {
            // 관측 수명 비율 시계열 (날짜), 없으면 물리값
            let mut s = crate::timeseries::Series::new("life", "").with_time_unit("day");
            for o in unit.obs.iter() {
                if let Some(l) = o.l_hat {
                    s.push(o.day - unit.installed_day, l);
                }
            }
            if s.len() < 3 {
                return forecast_with(Method::V0Physics, kp, unit, now_day, calib, rng, site_hist);
            }
            let step = s.median_step().unwrap_or(30.0).max(1.0);
            let horizon = ((RUL_CAP_DAYS / step).ceil() as usize).clamp(1, 400);
            // 기존 파이프라인 방식: 물리 사전값(누적 소비 D, 미래는 최근 30일 소비율로 연장)을 빼고 잔차만 Holt 로 예측
            let obs_d: Vec<(f64, f64)> = unit.obs.iter().filter(|o| o.l_hat.is_some()).map(|o| (o.day - unit.installed_day, o.dose)).collect();
            let (t_last, d_last) = obs_d.last().cloned().unwrap_or((0.0, unit.dose));
            let r30 = unit.rate(30);
            let prior = move |t: f64| -> f64 {
                if t <= t_last + 1e-9 {
                    obs_d.iter().min_by(|a, b| (a.0 - t).abs().partial_cmp(&(b.0 - t).abs()).unwrap_or(std::cmp::Ordering::Equal)).map(|x| x.1).unwrap_or(0.0)
                } else {
                    d_last + r30 * (t - t_last)
                }
            };
            let f = crate::timeseries::forecast(&s, horizon, None, Some(&prior), Some(1.0));
            let since_last = now_day - unit.installed_day - s.t.last().cloned().unwrap_or(0.0);
            let cross = |q: Option<&Vec<f64>>| -> f64 {
                q.and_then(|v| v.iter().position(|x| *x >= 1.0)).map(|k| (k as f64 + 1.0) * f.step - since_last).unwrap_or(RUL_CAP_DAYS).clamp(0.0, RUL_CAP_DAYS)
            };
            // 위 밴드(0.9)가 먼저 닿는다 = 짧은 RUL(10 %), 아래 밴드(0.1)가 늦다 = 긴 RUL(90 %)
            let r10 = cross(f.band(0.9));
            let r50 = cross(f.band(0.5)).max(r10);
            let r90 = cross(f.band(0.1)).max(r50);
            let q = QuantileForecast::new(&[0.1, 0.5, 0.9], vec![r10, r50, r90]);
            let last = s.y.last().cloned().unwrap_or(0.0);
            let sd = f.quantiles.first().zip(f.quantiles.get(2)).map(|(a, b)| (b.first().cloned().unwrap_or(0.0) - a.first().cloned().unwrap_or(0.0)) / 2.563).unwrap_or(0.0).abs();
            let l_now = [last - 1.2816 * sd, last, last + 1.2816 * sd];
            let pf = |h: f64| q.cdf(h).clamp(0.0, 1.0);
            Prognosis { method: method.key().into(), p_fail: [pf(7.0), pf(30.0), pf(90.0)], rul: q, l_now, health: health_of(l_now), n_obs }
        }
        Method::V4Recal => {
            // v3 를 낸 뒤 남은 수명 분포를 G∘F 로 고친다 (지금 상태 추정은 그대로)
            let mut p = forecast_with(Method::V3Calibrated, kp, unit, now_day, calib, rng, site_hist);
            if let Some(rc) = &calib.recal {
                p.rul = rc.apply(&p.rul, 0.0);
                p.p_fail = [rc.cdf(p.p_fail[0]), rc.cdf(p.p_fail[1]), rc.cdf(p.p_fail[2])];
            }
            p.method = method.key().into();
            p
        }
        Method::V2Bayes | Method::V3Calibrated => {
            let (m, s) = posterior_u(kp, unit);
            let alpha = 1.0 / (kp.gamma_cv * kp.gamma_cv).max(1e-9);
            let window = if method == Method::V3Calibrated { calib.rate_window } else { 30 };
            let rate = unit.rate(window).max(unit.rate(30) * 0.25).max(1e-9);
            // 지금 상태: 마지막 관측과 u·D 를 분산 가중으로 합친다
            let last = unit.obs.iter().rev().find(|o| o.l_hat.is_some());
            let mut ls = Vec::with_capacity(MC_SAMPLES);
            let mut ruls = Vec::with_capacity(MC_SAMPLES);
            let v3 = method == Method::V3Calibrated;
            for _ in 0..MC_SAMPLES {
                let mut tries = 0;
                let (u, now) = loop {
                    let u = (m + s * rng.normal()).exp();
                    let prior_now = u * unit.dose;
                    let prior_var = u * u * unit.dose / alpha + 1e-6;
                    let now = match last {
                        Some(o) => {
                            // 관측 뒤 소비분은 u 로 더한다
                            let l_obs = o.l_hat.unwrap_or(prior_now) + u * (unit.dose - o.dose);
                            let w = prior_var / (prior_var + o.var);
                            let mean = w * l_obs + (1.0 - w) * prior_now;
                            let var = (prior_var * o.var / (prior_var + o.var)).max(1e-8);
                            (mean + var.sqrt() * rng.normal()).max(0.0)
                        }
                        None => (prior_now + prior_var.sqrt() * rng.normal()).max(0.0),
                    };
                    // v3: 아직 교체 기준 전이라는 사실(지금 운전 중)로 조건을 건다 — 1 을 넘는 표본은 다시 뽑는다
                    if !v3 || now < 1.0 || tries >= 30 {
                        break (u, if v3 { now.min(0.999) } else { now });
                    }
                    tries += 1;
                };
                ls.push(now);
                let need = (1.0 - now).max(0.0);
                // 남은 소비량: u·X(ΔD) ≥ need 인 ΔD, X 는 감마 과정 → 정규 근사 (평균 need/u, 변동 계수 1/√(α·need/u))
                let mean_d = need / u.max(1e-6);
                let cv = if mean_d > 0.0 { 1.0 / (alpha * mean_d).sqrt() } else { 0.0 };
                let dd = (mean_d * (1.0 + cv * rng.normal())).max(0.0);
                // v3: 미래 소비 — 현장 이력이 1년 이상이면 계절 나이브(몇 해 전 같은 날짜부터), 아니면 지난 소비를
                // 30일 묶음으로 다시 뽑아 쌓는다 (계절 · 폭풍 몰림 · 주말을 그대로)
                let days = if v3 {
                    seasonal_days(dd, site_hist, rng).unwrap_or_else(|| {
                        let h = if site_hist.len() > unit.daily.len() { site_hist } else { &unit.daily[..] };
                        bootstrap_days(dd, h, 30, rng)
                    })
                } else {
                    dd / rate
                };
                ruls.push(days.min(RUL_CAP_DAYS));
            }
            if method == Method::V3Calibrated && (calib.spread - 1.0).abs() > 1e-9 {
                // 로그 RUL 중앙값 둘레로 퍼짐을 배수만큼 넓히거나 좁힌다
                let mut v = ruls.clone();
                v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let med = v[v.len() / 2].max(0.5);
                for r in ruls.iter_mut() {
                    let x = (r.max(0.5) / med).ln();
                    *r = (med * (calib.spread * x).exp()).min(RUL_CAP_DAYS);
                }
            }
            let mut lsorted = ls.clone();
            lsorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let k = |p: f64| lsorted[((p * lsorted.len() as f64) as usize).min(lsorted.len() - 1)];
            let l_now = [k(0.1), k(0.5), k(0.9)];
            Prognosis {
                method: method.key().into(),
                p_fail: [p_within(&ruls, 7.0), p_within(&ruls, 30.0), p_within(&ruls, 90.0)],
                rul: quantiles_of(ruls),
                l_now,
                health: health_of(l_now),
                n_obs,
            }
        }
    }
}

/// 현장 운영 원장에 이어 저장하는 부품 자리 하나의 열화 상태
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompDeg {
    pub id: String,
    pub unit: Unit,
    /// 이 자리의 하루 물리 소비 이력 (개체가 바뀌어도 이어짐, 최근 1100일) — 계절 투영
    pub hist: Vec<f64>,
    #[serde(default)]
    pub prog: Option<Prognosis>,
    #[serde(default)]
    pub last_z: Option<f64>,
    #[serde(default)]
    pub last_prog_day: f64,
}

/// 원장의 열화 시계열 상태
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegState {
    pub rng: Rng,
    /// v3 퍼짐 보정 배수 (백테스트 · 현장 하나 빼기로 맞춘 값)
    pub spread: f64,
    pub comps: Vec<CompDeg>,
}

impl DegState {
    pub fn new(seed: u64, spread: f64) -> Self {
        Self { rng: Rng::new(seed), spread, comps: Vec::new() }
    }

    pub fn get(&self, id: &str) -> Option<&CompDeg> {
        self.comps.iter().find(|c| c.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicators_invert_their_mean() {
        for ind in [
            Indicator::BrokenWires { limit: 8.0, k: 4.0, detect: 0.8 },
            Indicator::Linear { span: 12.0, sd: 0.2 },
            Indicator::Vibration { v0: 1.0, v_lim: 4.5, sd_log: 0.15 },
            Indicator::RustArea { sd_log: 0.3 },
        ] {
            for l in [0.6, 0.8, 1.0] {
                let z = ind.expected(l);
                let (lh, var) = ind.invert(z).unwrap();
                assert!((lh - l).abs() < 1e-9, "{ind:?} {l} {lh}");
                assert!(var > 0.0 && var.is_finite());
            }
        }
        assert!((weibull_cv(1.0) - 1.0).abs() < 1e-6);
        assert!((weibull_cv(2.0) - 0.5227).abs() < 1e-3);
    }

    #[test]
    fn gamma_process_has_unit_mean_and_target_cov() {
        let kp = params(ComponentKind::Brake, 10.0, 85.0);
        let mut rng = Rng::new(5);
        let mut ends = Vec::new();
        for _ in 0..2000 {
            let mut u = Unit { u: 1.0, ..Default::default() };
            for _ in 0..100 {
                u.advance(&kp, 0.01, &mut rng);
            }
            ends.push(u.truth);
        }
        let mean = ends.iter().sum::<f64>() / ends.len() as f64;
        let sd = (ends.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / ends.len() as f64).sqrt();
        assert!((mean - 1.0).abs() < 0.02, "{mean}");
        assert!((sd / mean - kp.gamma_cv).abs() < 0.02, "{}", sd / mean);
    }

    #[test]
    fn bayes_learns_the_unit_factor_from_observations() {
        let kp = params(ComponentKind::Brake, 10.0, 85.0);
        let mut rng = Rng::new(11);
        let mut unit = Unit { u: 1.3, ..Default::default() };
        for d in 0..300 {
            unit.advance(&kp, 0.0015, &mut rng);
            if d % 30 == 29 {
                unit.observe(&kp, d as f64, &mut rng);
            }
        }
        let (m, s) = posterior_u(&kp, &unit);
        // 사전(평균 1)에서 참값(1.3) 쪽으로 움직이고, 불확실성은 사전보다 작다
        assert!(m.exp() > 1.05 && s < 0.246, "{} ± {s}", m.exp());
        // 참 남은 수명: (1 − 참 상태) / (u·소비율)
        let p = forecast(Method::V2Bayes, &kp, &unit, 300.0, &Calib::default(), &mut rng);
        let true_rul = (1.0 - unit.truth) / (1.3 * 0.0015);
        let med = p.rul.median();
        assert!((med - true_rul).abs() < 0.35 * true_rul.max(30.0), "{med} vs {true_rul}");
        let v0 = forecast(Method::V0Physics, &kp, &unit, 300.0, &Calib::default(), &mut rng);
        assert!((v0.rul.median() - true_rul).abs() > (med - true_rul).abs(), "물리만 쓰면 개체 차이를 모른다");
    }
}
