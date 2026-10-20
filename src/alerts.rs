//! 사고 예방 알림 · 위험 지수.
//!
//! - `AlertPolicy`: 바람 · 낙뢰 · 시정 · 폭염 · 한파 · 중량물 · 피로의 알림 문턱 (현장 프리셋마다 바꿀 수 있다).
//! - `Alert`: 언제 · 얼마나 위험한지 · 무엇을 해야 하는지 · 근거. 같은 알림은 단계가 오를 때만 다시 띄운다.
//! - `RiskModel`: 운전 시간당 사고율 = 기준율 × 조건 배수의 곱. 배수는 문헌(교대 · 피로, 날씨, 정비)에서 왔고
//!   어떤 조건이 얼마나 키웠는지 항목별로 돌려준다. 사고는 이 비율로 시드가 있는 난수에서 뽑는다.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertLevel {
    Info,
    Caution,
    Warning,
    Danger,
    Stop,
}

impl AlertLevel {
    pub fn label(&self) -> &'static str {
        match self {
            AlertLevel::Info => "안내",
            AlertLevel::Caution => "주의",
            AlertLevel::Warning => "경고",
            AlertLevel::Danger => "위험",
            AlertLevel::Stop => "작업 중지",
        }
    }
    pub fn rank(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// 규칙 id (예: "wind", "lightning", "rope_hoist")
    pub id: String,
    pub day: u32,
    pub hour: u8,
    /// "D3 14:00"
    pub at: String,
    pub level: AlertLevel,
    pub level_label: String,
    pub title: String,
    pub message: String,
    /// 권장 조치
    pub action: String,
    /// 문턱의 근거
    pub basis: String,
    pub value: f64,
    pub limit: f64,
    pub unit: String,
    /// 예보로 미리 띄운 알림인가 (선행 시간 h)
    #[serde(default)]
    pub lead_h: Option<u32>,
    /// 이 알림으로 결정 카드가 열렸는가
    #[serde(default)]
    pub decision_id: Option<String>,
    /// 작업 절차 (작업 전 점검 · 줄걸이 · 시험 인양 · 인양 · 이동 · 착지 · 작업 후 · 정비 · 점검)
    #[serde(default)]
    pub procedure: String,
}

/// 알림 문턱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPolicy {
    /// 순간풍속(3초 돌풍, 크레인 높이) 주의 · 경고 · 운전 정지 · 폭풍 대비 (m/s)
    pub wind_caution_mps: f64,
    pub wind_warn_mps: f64,
    pub wind_stop_mps: f64,
    pub storm_mps: f64,
    /// 이 순간풍속을 넘긴 뒤에는 점검 후 재가동 (산업안전보건기준에 관한 규칙 제143조)
    pub post_storm_inspect_mps: f64,
    /// 바람 예보를 보는 시간 · 태풍 예보를 보는 시간 (h)
    pub wind_lookahead_h: u32,
    pub storm_lookahead_h: u32,
    /// 낙뢰 작업 중지 거리 (km) · 마지막 낙뢰 뒤 재개까지 (분)
    pub lightning_km: f64,
    pub lightning_clear_min: f64,
    /// 시정: 주의(신호수 · 감속) · 중지 (m)
    pub visibility_caution_m: f64,
    pub visibility_stop_m: f64,
    /// 체감온도 단계 (관심 · 주의 · 경고 · 위험, °C) — 고용노동부 폭염 대비 근로자 건강보호 가이드
    pub heat_stages_c: Vec<f64>,
    /// 한파: 저온 취성 · 윤활 점검 기온 (°C)
    pub cold_c: f64,
    /// 연속 운전 피로 (h) — 이 시간을 넘는 근무는 교대 · 휴식
    pub fatigue_shift_h: f64,
    /// 법정 · 정기 점검 미리 알림 (일)
    pub inspection_notice_days: f64,
    /// 권상용 와이어로프 법정 최소 안전계수
    pub rope_sf_legal: f64,
    /// 야간 작업 최소 조도 (lx)
    pub lighting_min_lux: f64,
}

impl Default for AlertPolicy {
    fn default() -> Self {
        crate::field_constants::alert_policy()
    }
}

impl AlertPolicy {
    /// 크레인 운전 허용 풍속(제원)에 맞춘 정책 — 정지 문턱을 제원 값으로, 주의 · 경고는 같은 비율로
    pub fn for_wind_limit(&self, wind_in_service_mps: f64) -> AlertPolicy {
        let mut p = self.clone();
        if wind_in_service_mps > 0.0 {
            let k = wind_in_service_mps / self.wind_stop_mps.max(1e-9);
            p.wind_caution_mps *= k;
            p.wind_warn_mps *= k;
            p.wind_stop_mps = wind_in_service_mps;
            // 계류 · 고정 문턱은 운전 정지 문턱보다 낮을 수 없다
            p.storm_mps = p.storm_mps.max(wind_in_service_mps + 2.0);
        }
        p
    }

    /// 체감온도 → 폭염 단계 (0 없음, 1 관심, 2 주의, 3 경고, 4 위험)
    pub fn heat_stage(&self, feels_like_c: f64) -> usize {
        self.heat_stages_c.iter().filter(|t| feels_like_c >= **t).count()
    }

    /// 폭염 단계의 시간당 휴식 (분): 33 °C 이상 2시간마다 20분 (제560조 ③) = 시간당 10분, 35 °C 15분 + 14–17시 옥외 중지,
    /// 38 °C 이상은 옥외 작업 중지 (고용노동부 2026 대책) — 60분
    pub fn heat_rest_min(&self, stage: usize) -> f64 {
        match stage {
            0 | 1 => 0.0,
            2 => 10.0,
            3 => 15.0,
            _ => 60.0,
        }
    }
}

/// 사고율 배수 (운전 시간당 기준 사고율 × 곱)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskModel {
    /// 기준 사고율 (운전 1시간당, 보고 대상 사고 — 재산 피해 · 부상 · 아차사고 중 조사 대상)
    pub base_per_hour: f64,
    /// 바람: 배수 = 1 + a·(돌풍/정지 문턱)^p, 정지 문턱을 넘겨 운전하면 over 배
    pub wind_a: f64,
    pub wind_p: f64,
    pub wind_over: f64,
    pub wet: f64,
    pub icy: f64,
    pub fog: f64,
    pub dense_fog: f64,
    /// 시야 제한 (200 m 이상 · 시정 주의 문턱 500 m 미만)
    #[serde(default = "default_haze")]
    pub haze: f64,
    pub lightning: f64,
    /// 폭염 단계(주의 · 경고 · 위험)에서 휴식 없이 일할 때
    pub heat: Vec<f64>,
    pub cold: f64,
    pub night: f64,
    /// 오후 교대 (14–22시)
    pub afternoon: f64,
    /// 8 h 를 넘긴 근무 1 h 당 증가 (교대 길이 효과)
    pub overtime_per_h: f64,
    pub novice: f64,
    pub expert: f64,
    pub no_signalman: f64,
    pub no_exclusion: f64,
    pub maint_good: f64,
    pub maint_poor: f64,
    pub critical_unplanned: f64,
    pub critical_planned: f64,
    /// 교체 기준을 넘긴 부품이 있을 때 (로프 · 브레이크 · 그 밖)
    pub rope_overdue: f64,
    pub brake_overdue: f64,
    pub component_overdue: f64,
    pub inspection_overdue: f64,
    pub legal_overdue: f64,
    pub low_light: f64,
}

impl Default for RiskModel {
    fn default() -> Self {
        crate::field_constants::risk_model()
    }
}

fn default_haze() -> f64 {
    1.2
}

/// 한 시간 운전 조건 — 위험 지수 계산 입력
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskInputs {
    pub gust_mps: f64,
    pub wind_stop_mps: f64,
    pub wet: bool,
    pub icy: bool,
    pub visibility_m: f64,
    pub thunder_km: Option<f64>,
    pub lightning_km: f64,
    pub heat_stage: usize,
    pub heat_rest: bool,
    pub cold: bool,
    pub night: bool,
    #[serde(default)]
    pub afternoon: bool,
    pub lighting_lux: f64,
    #[serde(default = "default_lux_min")]
    pub lighting_min_lux: f64,
    pub hours_on_shift: f64,
    pub operator: String,
    pub signalman: bool,
    pub exclusion_zone: bool,
    pub maintenance: String,
    /// 이 시간 들기 중 중량물 비율 · 작업계획서 여부
    pub critical_share: f64,
    pub critical_planned: bool,
    pub rope_overdue: bool,
    pub brake_overdue: bool,
    pub other_overdue: bool,
    pub inspection_overdue: bool,
    pub legal_overdue: bool,
    pub speed_scale: f64,
}

fn default_lux_min() -> f64 {
    150.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskResult {
    pub multiplier: f64,
    /// 운전 1시간당 사고율
    pub per_hour: f64,
    /// 0–100 위험 지수 (log 척도: 기준 = 20, 배수 10 = 60, 100 = 100)
    pub index: f64,
    /// (요인, 배수) — 1 보다 큰 것만
    pub factors: Vec<(String, f64)>,
}

impl RiskModel {
    pub fn evaluate(&self, r: &RiskInputs) -> RiskResult {
        let mut f: Vec<(String, f64)> = Vec::new();
        let mut push = |name: &str, k: f64| {
            if (k - 1.0).abs() > 1e-9 {
                f.push((name.to_string(), k));
            }
        };
        if r.wind_stop_mps > 0.0 {
            let x = r.gust_mps / r.wind_stop_mps;
            // 감속 운전은 흔들림 · 제동 거리를 줄인다 — 속도 비율만큼 바람 배수의 초과분을 줄인다
            let k = 1.0 + self.wind_a * x.max(0.0).powf(self.wind_p) * r.speed_scale.clamp(0.3, 1.0);
            push("바람", if x >= 1.0 { k * self.wind_over } else { k });
        }
        if r.icy {
            push("결빙 레일", self.icy);
        } else if r.wet {
            push("젖은 레일", self.wet);
        }
        // 시야: 감속 운전(신호수 · 감속)은 바람과 같이 배수의 초과분을 속도 비율만큼 줄인다 (반응 · 정지 거리)
        let slow = r.speed_scale.clamp(0.3, 1.0);
        let vis = |k: f64| 1.0 + (k - 1.0) * slow;
        if r.visibility_m < 50.0 {
            push("짙은 안개", vis(self.dense_fog));
        } else if r.visibility_m < 200.0 {
            push("시정 저하", vis(self.fog));
        } else if r.visibility_m < 500.0 {
            push("시야 제한", vis(self.haze));
        }
        if let Some(d) = r.thunder_km {
            if d <= r.lightning_km {
                push("낙뢰 근접", self.lightning);
            }
        }
        if r.heat_stage >= 2 && !r.heat_rest {
            let i = (r.heat_stage - 2).min(self.heat.len().saturating_sub(1));
            push("폭염 (휴식 없음)", self.heat.get(i).cloned().unwrap_or(1.0));
        }
        if r.cold {
            push("한파", self.cold);
        }
        if r.night {
            push("야간", self.night);
            if r.lighting_lux < r.lighting_min_lux {
                push("조도 부족", self.low_light);
            }
        } else if r.afternoon {
            push("오후 교대", self.afternoon);
        }
        if r.hours_on_shift > 8.0 {
            push("장시간 근무", 1.0 + self.overtime_per_h * (r.hours_on_shift - 8.0));
        }
        match r.operator.as_str() {
            "novice" => push("초보 운전원", self.novice),
            "expert" => push("숙련 운전원", self.expert),
            _ => {}
        }
        if !r.signalman {
            push("신호수 없음", self.no_signalman);
        }
        if !r.exclusion_zone {
            push("출입 통제 없음", self.no_exclusion);
        }
        match r.maintenance.as_str() {
            "good" => push("정비 양호", self.maint_good),
            "poor" => push("정비 미흡", self.maint_poor),
            _ => {}
        }
        if r.critical_share > 0.0 {
            let k = if r.critical_planned { self.critical_planned } else { self.critical_unplanned };
            push(if r.critical_planned { "중량물 (계획)" } else { "중량물 (계획 없음)" }, 1.0 + (k - 1.0) * r.critical_share.clamp(0.0, 1.0));
        }
        if r.rope_overdue {
            push("로프 교체 기준 초과", self.rope_overdue);
        }
        if r.brake_overdue {
            push("브레이크 교체 기준 초과", self.brake_overdue);
        }
        if r.other_overdue {
            push("부품 교체 기준 초과", self.component_overdue);
        }
        if r.legal_overdue {
            push("법정 안전검사 지남", self.legal_overdue);
        } else if r.inspection_overdue {
            push("정기 점검 지남", self.inspection_overdue);
        }
        let mult: f64 = f.iter().map(|(_, k)| *k).product::<f64>().max(1e-6);
        let index = (20.0 + 40.0 * mult.log10()).clamp(0.0, 100.0);
        f.retain(|(_, k)| *k > 1.0 + 1e-9);
        f.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        RiskResult { multiplier: mult, per_hour: self.base_per_hour * mult, index, factors: f }
    }
}

/// 같은 알림을 매시간 되풀이하지 않도록 활성 단계를 기억한다
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertBook {
    /// (알림 id, 지금 단계)
    pub active: Vec<(String, AlertLevel)>,
    /// (알림 id, 낮은 단계가 이어진 시간)
    #[serde(default)]
    pub calm: Vec<(String, u32)>,
}

impl AlertBook {
    /// 히스테리시스: 오르면 바로 알리고, 내려가거나 풀리는 것은 `hold` 시간 이어져야 반영한다
    pub fn update_held(&mut self, id: &str, level: Option<AlertLevel>, hold: u32) -> bool {
        let cur = self.level(id);
        let lower = match (cur, level) {
            (Some(c), Some(l)) => l < c,
            (Some(_), None) => true,
            _ => false,
        };
        if !lower {
            self.calm.retain(|(k, _)| k != id);
            return self.update(id, level);
        }
        let n = match self.calm.iter_mut().find(|(k, _)| k == id) {
            Some(x) => {
                x.1 += 1;
                x.1
            }
            None => {
                self.calm.push((id.to_string(), 1));
                1
            }
        };
        if n >= hold {
            self.calm.retain(|(k, _)| k != id);
            self.update(id, level)
        } else {
            false
        }
    }

    /// 새 단계 — 단계가 오르면 true (알림을 띄운다), 내려가면 기억만 바꾼다. None 이면 해제.
    pub fn update(&mut self, id: &str, level: Option<AlertLevel>) -> bool {
        let pos = self.active.iter().position(|(k, _)| k == id);
        match (pos, level) {
            (Some(i), Some(l)) => {
                let up = l > self.active[i].1;
                self.active[i].1 = l;
                up
            }
            (None, Some(l)) => {
                self.active.push((id.to_string(), l));
                true
            }
            (Some(i), None) => {
                self.active.remove(i);
                false
            }
            (None, None) => false,
        }
    }

    pub fn level(&self, id: &str) -> Option<AlertLevel> {
        self.active.iter().find(|(k, _)| k == id).map(|(_, l)| *l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calm() -> RiskInputs {
        RiskInputs {
            gust_mps: 0.0,
            wind_stop_mps: 20.0,
            visibility_m: 20_000.0,
            lightning_km: 10.0,
            operator: "standard".into(),
            signalman: true,
            exclusion_zone: true,
            maintenance: "average".into(),
            lighting_lux: 150.0,
            lighting_min_lux: 150.0,
            speed_scale: 1.0,
            ..Default::default()
        }
    }

    #[test]
    fn risk_grows_with_hazards_and_lists_factors() {
        let m = RiskModel::default();
        let base = m.evaluate(&calm());
        assert!((base.multiplier - 1.0).abs() < 1e-9);
        let mut bad = calm();
        bad.gust_mps = 22.0;
        bad.icy = true;
        bad.night = true;
        bad.hours_on_shift = 12.0;
        let r = m.evaluate(&bad);
        assert!(r.multiplier > 3.0, "{}", r.multiplier);
        assert!(r.index > base.index);
        assert_eq!(r.factors[0].0, "바람");
        // 감속하면 바람 배수가 줄어든다
        let mut slow = bad.clone();
        slow.speed_scale = 0.6;
        assert!(m.evaluate(&slow).multiplier < r.multiplier);
    }

    #[test]
    fn alert_book_raises_only_on_escalation() {
        let mut b = AlertBook::default();
        assert!(b.update("wind", Some(AlertLevel::Caution)));
        assert!(!b.update("wind", Some(AlertLevel::Caution)));
        assert!(b.update("wind", Some(AlertLevel::Warning)));
        assert!(!b.update("wind", Some(AlertLevel::Caution)));
        assert_eq!(b.level("wind"), Some(AlertLevel::Caution));
        assert!(!b.update("wind", None));
        assert!(b.level("wind").is_none());
    }

    #[test]
    fn policy_scales_to_crane_wind_limit_and_heat_stages() {
        let p = AlertPolicy::default();
        let q = p.for_wind_limit(25.0);
        assert_eq!(q.wind_stop_mps, 25.0);
        assert!(q.wind_warn_mps > p.wind_warn_mps);
        assert_eq!(p.heat_stage(30.0), 0);
        assert_eq!(p.heat_stage(p.heat_stages_c[1] + 0.1), 2);
        assert!(p.heat_rest_min(3) > p.heat_rest_min(2));
    }
}
