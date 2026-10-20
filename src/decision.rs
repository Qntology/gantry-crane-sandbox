//! 크레인 결정 엔진 — endmill-sandbox `decision.rs` 의 구조를 그대로 옮겼다.
//!
//! 1. 결정론적 게이트(값 · 한계 · 심각도 · 조치)를 모두 계산한다.
//! 2. 통과하지 못한 게이트 중 가장 보수적인 조치가 규칙 판단이다.
//! 3. LAYA 조언자는 규칙보다 **더 보수적인 쪽으로만** 올릴 수 있다 (점검 단계까지).
//! 4. 작업자는 규칙 판단보다 덜 보수적인 조치를 고를 수 없다 (게이트가 결정권자, 모델은 조언자).
//!
//! 하드 인터록(정격 하중 제한 장치 · 리밋 · STO/SBC)은 실제로는 드라이브와 안전 PLC 가 맡는다.
//! 여기서는 시뮬레이션 결과를 같은 기준으로 판정해 보여 줄 뿐이다.

use crate::antisway::AntiSwayMode;
use crate::crane_profile::{fem_duty_limits, CraneSpec};
use crate::cyclesim::CycleReport;
use crate::duty::DutyReport;
use crate::timeseries::ForecastResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Continue,
    ReduceSpeed,
    ReduceAcceleration,
    EnableAntiSway,
    CoolDown,
    Inspect,
    LimitLoad,
    Park,
    Stop,
}

impl Action {
    pub fn all() -> [Action; 9] {
        [
            Action::Continue,
            Action::ReduceSpeed,
            Action::ReduceAcceleration,
            Action::EnableAntiSway,
            Action::CoolDown,
            Action::Inspect,
            Action::LimitLoad,
            Action::Park,
            Action::Stop,
        ]
    }

    pub fn key(&self) -> &'static str {
        match self {
            Action::Continue => "continue",
            Action::ReduceSpeed => "reduce_speed",
            Action::ReduceAcceleration => "reduce_acceleration",
            Action::EnableAntiSway => "enable_anti_sway",
            Action::CoolDown => "cool_down",
            Action::Inspect => "inspect",
            Action::LimitLoad => "limit_load",
            Action::Park => "park",
            Action::Stop => "stop",
        }
    }

    pub fn from_key(k: &str) -> Option<Action> {
        Action::all().into_iter().find(|a| a.key() == k.trim())
    }

    pub fn label(&self) -> &'static str {
        match self {
            Action::Continue => "계속 운전",
            Action::ReduceSpeed => "속도 낮춤",
            Action::ReduceAcceleration => "가감속 완화",
            Action::EnableAntiSway => "흔들림 억제 사용",
            Action::CoolDown => "모터 냉각 대기",
            Action::Inspect => "점검",
            Action::LimitLoad => "하중 제한",
            Action::Park => "계류 (바람)",
            Action::Stop => "정지",
        }
    }

    /// LAYA 조언자에게 보내는 영어 선택지
    pub fn english(&self) -> &'static str {
        match self {
            Action::Continue => "continue operating the crane as it is",
            Action::ReduceSpeed => "reduce the travel and hoist speeds",
            Action::ReduceAcceleration => "soften the acceleration and deceleration ramps",
            Action::EnableAntiSway => "enable or retune the anti-sway control",
            Action::CoolDown => "pause to let the drive motors cool down",
            Action::Inspect => "stop for an inspection of brakes, rope and drives",
            Action::LimitLoad => "restrict the hoisted load below the rated capacity",
            Action::Park => "park and secure the crane against the wind",
            Action::Stop => "stop the crane immediately",
        }
    }

    /// 선택 카드에 보이는 한 줄 설명
    pub fn detail(&self) -> &'static str {
        match self {
            Action::Continue => "지금 조건 그대로 운전",
            Action::ReduceSpeed => "최고 속도를 낮춰 흔들림·편행·전류를 줄임",
            Action::ReduceAcceleration => "가감속 기울기를 낮춰 토크 포화·흔들림을 줄임",
            Action::EnableAntiSway => "입력 셰이핑 또는 각도 궤환을 켜거나 다시 맞춤",
            Action::CoolDown => "사이클 사이 휴지 시간을 늘려 권선 온도를 낮춤",
            Action::Inspect => "멈추고 브레이크·로프·구동부를 확인",
            Action::LimitLoad => "권상 상승을 막고 하중을 정격 안으로 제한",
            Action::Park => "운전을 멈추고 레일 클램프·계류 장치로 고정",
            Action::Stop => "즉시 정지 — 한계를 넘은 항목이 있음",
        }
    }

    pub fn conservativeness(&self) -> u8 {
        match self {
            Action::Continue => 0,
            Action::ReduceSpeed | Action::ReduceAcceleration | Action::EnableAntiSway => 1,
            Action::CoolDown | Action::Inspect => 2,
            Action::LimitLoad | Action::Park => 3,
            Action::Stop => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate {
    pub id: String,
    pub label: String,
    pub value: f64,
    pub limit: f64,
    pub unit: String,
    /// 0 통과 · 1 관찰 · 2 초과 · 3 위험
    pub severity: u8,
    pub passed: bool,
    pub action: Action,
    pub message: String,
    /// 한계값의 근거
    pub basis: String,
}

#[allow(clippy::too_many_arguments)]
fn gate(id: &str, label: &str, value: f64, limit: f64, unit: &str, severity: u8, action: Action, message: String, basis: &str) -> Gate {
    Gate {
        id: id.into(),
        label: label.into(),
        value,
        limit,
        unit: unit.into(),
        severity,
        passed: severity == 0,
        action: if severity == 0 { Action::Continue } else { action },
        message,
        basis: basis.into(),
    }
}

pub struct DecisionInput<'a> {
    pub spec: &'a CraneSpec,
    pub cycle: Option<&'a CycleReport>,
    pub duty: Option<&'a DutyReport>,
    pub forecasts: &'a [ForecastResult],
    /// 측정 · 예보 풍속 (m/s). 없으면 사이클의 최대 풍속
    pub wind_mps: Option<f64>,
    pub antisway_mode: AntiSwayMode,
    /// SDS 드리프트 (이름, z)
    pub drift: &'a [(String, f64)],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisorAnswer {
    pub labels: Vec<String>,
    pub probabilities: Vec<f32>,
    pub top: String,
    pub top_probability: f32,
    pub act_probability: f32,
}

/// 조언 모델 (LAYA 등)
pub trait Advisor {
    fn ask_choice(&self, instruction: &str, options: &[(String, String)], state: &str) -> Result<AdvisorAnswer, String>;
}

#[cfg(feature = "ml")]
impl Advisor for crate::ml::laya::LayaAdvisor {
    fn ask_choice(&self, instruction: &str, options: &[(String, String)], state: &str) -> Result<AdvisorAnswer, String> {
        let a = crate::ml::laya::LayaAdvisor::ask_choice(self, instruction, options, state)?;
        Ok(AdvisorAnswer { labels: a.labels, probabilities: a.probabilities, top: a.top, top_probability: a.top_probability, act_probability: a.act_probability })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionReport {
    pub gates: Vec<Gate>,
    pub rule_action: Action,
    pub rule_action_label: String,
    pub rule_reasons: Vec<String>,
    /// 속도 · 가감속 조정 제안 배율
    pub speed_scale: Option<f64>,
    pub accel_scale: Option<f64>,
    pub state_text: String,
    pub advisor: Option<AdvisorAnswer>,
    pub advisor_action: Option<Action>,
    pub advisor_error: Option<String>,
    pub advisor_allowed: bool,
    pub final_action: Action,
    pub final_action_label: String,
    pub escalated_by_advisor: bool,
    pub agreement: Option<bool>,
    pub notes: Vec<String>,
    #[serde(default)]
    pub operator_choice: Option<Action>,
    #[serde(default)]
    pub system_action: Option<Action>,
    #[serde(default)]
    pub choice: Option<crate::choice::ChoicePrompt>,
}

fn sev_ge(value: f64, steps: &[(f64, u8)]) -> u8 {
    // steps: (문턱, 심각도) 를 문턱 오름차순으로
    let mut s = 0;
    for (th, sv) in steps {
        if value >= *th {
            s = *sv;
        }
    }
    s
}

pub fn evaluate(inp: &DecisionInput) -> DecisionReport {
    let spec = inp.spec;
    let lim = &spec.limits;
    let mut gates: Vec<Gate> = Vec::new();
    let mut notes = Vec::new();

    if let Some(c) = inp.cycle {
        // 과부하
        let r = c.load_peak_kg / spec.structure.rated_load_kg.max(1e-9);
        let sev = if r >= lim.overload_trip {
            3
        } else if r > 1.0 + 1e-9 {
            2
        } else if r >= lim.overload_warn {
            1
        } else {
            0
        };
        gates.push(gate(
            "overload",
            "정격 하중 비",
            r,
            1.0,
            "×",
            sev,
            if sev >= 2 { Action::LimitLoad } else { Action::Continue },
            format!("최대 하중 {:.0} kg / 정격 {:.0} kg", c.load_peak_kg, spec.structure.rated_load_kg),
            "Schneider Overload_EN15011 90% 경고, Demag 10% 이상 과부하 차단, EN 12077-2",
        ));

        for a in c.axes.iter() {
            let ax = a.axis;
            let key = ax.key();
            // 토크 포화
            let sev = if a.peak_torque_ratio >= 0.999 && a.torque_saturated_s > 0.5 {
                2
            } else if a.peak_torque_ratio >= 0.9 {
                1
            } else {
                0
            };
            gates.push(gate(
                &format!("motor_torque_{key}"),
                &format!("{} 모터 토크 사용률", ax.label()),
                a.peak_torque_ratio,
                1.0,
                "×",
                sev,
                Action::ReduceAcceleration,
                format!("최대 {:.1} Nm, 한계 도달 {:.1} s", a.peak_torque_nm, a.torque_saturated_s),
                "모터 · 인버터 토크 한계",
            ));
            // RMS 전류 (사이클 전체, 열 등가)
            let ri = a.rms_current_a / a.rated_current_a.max(1e-9);
            let sev = sev_ge(ri, &[(0.9, 1), (1.0, 2)]);
            gates.push(gate(
                &format!("motor_current_{key}"),
                &format!("{} RMS 전류 / 정격", ax.label()),
                ri,
                1.0,
                "×",
                sev,
                Action::CoolDown,
                format!("RMS {:.1} A, 최대 {:.1} A (정격 {:.1} A)", a.rms_current_a, a.peak_current_a, a.rated_current_a),
                "모터 정격 전류",
            ));
            let d = spec.drive(ax);
            if a.inverter_overload_peak_s >= d.inverter.overload_s {
                gates.push(gate(
                    &format!("inverter_overload_{key}"),
                    &format!("{} 인버터 과부하 시간", ax.label()),
                    a.inverter_overload_peak_s,
                    d.inverter.overload_s,
                    "s",
                    2,
                    Action::ReduceAcceleration,
                    "과부하 전류 허용 시간을 다 써서 연속 전류로 제한됐습니다".into(),
                    "인버터 과부하 150% 60 s (Yaskawa CR700 등)",
                ));
            }
            // 권선 온도
            let peak = inp.duty.map(|du| du.max_winding_c[ax.index()]).unwrap_or(a.peak_winding_c).max(a.peak_winding_c);
            let ins = a.insulation_limit_c;
            let sev = sev_ge(peak, &[(ins - 2.0 * lim.winding_margin_k, 1), (ins - lim.winding_margin_k, 2), (ins, 3)]);
            gates.push(gate(
                &format!("motor_temp_{key}"),
                &format!("{} 권선 최고 온도", ax.label()),
                peak,
                ins - lim.winding_margin_k,
                "°C",
                sev,
                if sev >= 3 { Action::Stop } else { Action::CoolDown },
                format!("절연 한계 {:.0} °C, 여유 {:.0} K", ins, lim.winding_margin_k),
                "절연 등급 (F종 155 °C)",
            ));
            // 운전율 · 기동 횟수 — 사이클 사이 휴지 시간이 들어간 근무(듀티) 결과로만 판정한다
            if let Some(du) = inp.duty {
                let fem = fem_duty_limits(&spec.class.mechanism_class);
                let cdf_lim = fem.and_then(|f| f.cdf).map(|c| c.min(a.rated_cdf)).unwrap_or(a.rated_cdf);
                let cdf = du.mean_cdf[ax.index()];
                gates.push(gate(
                    &format!("duty_cdf_{key}"),
                    &format!("{} 운전율 (CDF)", ax.label()),
                    cdf,
                    cdf_lim,
                    "",
                    if cdf > cdf_lim + 1e-9 { 1 } else { 0 },
                    Action::CoolDown,
                    format!("통전 시간 비율 {:.0}% / 허용 {:.0}%", cdf * 100.0, cdf_lim * 100.0),
                    "IEC 60034-1 S3 정격, FEM 기구 등급 운전율",
                ));
                let sph = du.starts_per_hour[ax.index()];
                let sph_lim = fem.map(|f| f.max_starts_per_hour.min(a.rated_starts_per_hour)).unwrap_or(a.rated_starts_per_hour);
                gates.push(gate(
                    &format!("starts_{key}"),
                    &format!("{} 기동 횟수", ax.label()),
                    sph,
                    sph_lim,
                    "회/h",
                    if sph > sph_lim { 1 } else { 0 },
                    Action::CoolDown,
                    format!("{:.0} 회/h", sph),
                    "FEM 듀티 표 (M5 240 회/h)",
                ));
            }
            // 브레이크
            let remaining = inp.duty.map(|du| {
                let k = ax.index();
                let usable = (du.brake_lining_new_mm[k] - du.brake_lining_min_mm[k]).max(1e-9);
                (du.brake_lining_mm[k] - du.brake_lining_min_mm[k]) / usable
            });
            let rem = remaining.unwrap_or(a.brake_lining_remaining);
            let sev = if rem <= 0.0 {
                3
            } else if rem <= lim.brake_lining_inspect {
                2
            } else if rem <= lim.brake_lining_warn {
                1
            } else {
                0
            };
            gates.push(gate(
                &format!("brake_lining_{key}"),
                &format!("{} 브레이크 라이닝 여유", ax.label()),
                rem,
                lim.brake_lining_inspect,
                "",
                sev,
                if sev >= 3 { Action::Stop } else { Action::Inspect },
                format!("남은 사용 두께 비율 {:.0}%", rem * 100.0),
                "Konecranes 브레이크 모니터링 (30% 정비 · 15% 점검)",
            ));
            if a.brake_max_stop_work_j > 0.0 {
                let r = a.brake_max_stop_work_j / a.brake_max_work_allowed_j.max(1e-9);
                gates.push(gate(
                    &format!("brake_work_{key}"),
                    &format!("{} 1회 제동 마찰일 / 허용", ax.label()),
                    r,
                    1.0,
                    "×",
                    sev_ge(r, &[(0.8, 1), (1.0, 2)]),
                    Action::Inspect,
                    format!("최대 {:.1} kJ / 허용 {:.1} kJ", a.brake_max_stop_work_j / 1000.0, a.brake_max_work_allowed_j / 1000.0),
                    "INTORQ BFK458 허용 마찰일 식",
                ));
            }
            let sev = sev_ge(a.brake_max_temp_c, &[(0.8 * a.brake_temp_limit_c, 1), (a.brake_temp_limit_c, 2)]);
            if a.brake_dynamic_stops > 0 || sev > 0 {
                gates.push(gate(
                    &format!("brake_temp_{key}"),
                    &format!("{} 브레이크 온도", ax.label()),
                    a.brake_max_temp_c,
                    a.brake_temp_limit_c,
                    "°C",
                    sev,
                    Action::CoolDown,
                    format!("미끄럼 제동 {} 회", a.brake_dynamic_stops),
                    "INTORQ BFK458 표면 130 °C",
                ));
            }
        }

        // 권상 브레이크 토크 여유
        let sev = if c.hoist_brake_factor < 1.0 {
            3
        } else if c.hoist_brake_factor < lim.hoist_brake_factor_min {
            2
        } else {
            0
        };
        gates.push(gate(
            "hoist_brake_factor",
            "권상 브레이크 토크 / 정적 토크",
            c.hoist_brake_factor,
            lim.hoist_brake_factor_min,
            "×",
            sev,
            Action::LimitLoad,
            format!("최대 하중 {:.0} kg 기준", c.load_peak_kg),
            "설정값 (적용 규격의 브레이크 여유로 교체)",
        ));

        // 로프
        let sev = if c.rope.min_safety_factor < 0.8 * c.rope.required_safety_factor {
            3
        } else if c.rope.min_safety_factor < c.rope.required_safety_factor {
            2
        } else {
            0
        };
        gates.push(gate(
            "rope_safety",
            "로프 안전율",
            c.rope.min_safety_factor,
            c.rope.required_safety_factor,
            "",
            sev,
            Action::LimitLoad,
            format!("줄당 최대 장력 {:.1} kN (동적 포함)", c.rope.max_tension_per_fall_kn),
            "ISO 16625 Zp (M5 4.5)",
        ));
        let dmg = inp.duty.map(|d| d.rope_damage).unwrap_or(c.rope.damage);
        let sev = if dmg >= 1.0 {
            3
        } else if dmg >= lim.rope_damage_warn {
            2
        } else {
            0
        };
        gates.push(gate(
            "rope_damage",
            "로프 굽힘 피로 누적 (Miner)",
            dmg,
            1.0,
            "",
            sev,
            if sev >= 3 { Action::Stop } else { Action::Inspect },
            format!("권상 행정 {} 회 기준, 상수: {}", c.rope.strokes, c.rope.fatigue_source),
            "Feyrer 굽힘 피로 + ISO 4309 폐기 검사",
        ));
        if c.slack_events > 0 {
            gates.push(gate(
                "slack_rope",
                "로프 느슨함",
                c.slack_events as f64,
                0.0,
                "회",
                2,
                Action::Inspect,
                "장력이 0 이 된 순간이 있습니다 (착지 · 낙하 · 급정지 확인)".into(),
                "느슨한 로프 감지",
            ));
        }

        // 흔들림 · 위치
        let tol = lim.sway_tolerance_mm;
        let closed = inp.antisway_mode == AntiSwayMode::ClosedLoop;
        let done: Vec<&crate::cyclesim::MoveMetrics> = c.moves.iter().filter(|m| !m.aborted).collect();
        let worst = done.iter().map(|m| if closed { m.final_sway_mm } else { m.residual_sway_mm }).fold(0.0, f64::max);
        let wind_note = if c.max_wind_mps > 3.0 { format!(", 바람 최대 {:.1} m/s (평형점 둘레 흔들림, 정적 편향 제외)", c.max_wind_mps) } else { String::new() };
        let sev = sev_ge(worst, &[(tol, 1), (2.0 * tol, 2)]);
        // 바람이 운전 허용 풍속의 절반 이상이면 남은 흔들림은 돌풍이 만든 것이라 가감속을 바꿔도 줄지 않는다
        // → 정밀 적재는 바람이 잦아질 때까지 보류(계류)
        let wind_driven = c.max_wind_mps >= 0.5 * lim.wind_in_service_mps;
        let act = if inp.antisway_mode == AntiSwayMode::Off {
            Action::EnableAntiSway
        } else if wind_driven {
            Action::Park
        } else {
            Action::ReduceAcceleration
        };
        gates.push(gate(
            "sway",
            "잔류 흔들림",
            worst,
            tol,
            "mm",
            sev,
            act,
            format!("방식 {}, 완료한 이동 {} 건 중 최대{}", inp.antisway_mode.key(), done.len(), wind_note),
            "설정값 (적재 정밀도 요구)",
        ));
        let pos = done.iter().map(|m| m.position_error_mm).fold(0.0, f64::max);
        gates.push(gate(
            "positioning",
            "위치 결정 오차",
            pos,
            lim.position_tolerance_mm,
            "mm",
            sev_ge(pos, &[(lim.position_tolerance_mm, 1)]),
            act,
            "안정 구간 끝 하중 위치 기준".into(),
            "설정값",
        ));

        // 비상 정지
        if c.estop {
            let slips: u64 = c.axes.iter().map(|a| a.brake_dynamic_stops).sum();
            let work = c.axes.iter().map(|a| a.brake_max_stop_work_j).fold(0.0, f64::max);
            gates.push(gate(
                "estop",
                "비상 정지 발생",
                1.0,
                0.0,
                "회",
                2,
                Action::Inspect,
                format!("미끄럼 제동 {} 회, 1회 최대 마찰일 {:.1} kJ — 브레이크 · 로프 · 구조 점검 후 재가동", slips, work / 1000.0),
                "비상 정지 후 점검",
            ));
        }

        // 편행
        let sev = sev_ge(c.max_skew_mm, &[(0.7 * lim.skew_limit_mm, 1), (lim.skew_limit_mm, 2)]);
        gates.push(gate(
            "skew",
            "갠트리 편행",
            c.max_skew_mm,
            lim.skew_limit_mm,
            "mm",
            sev,
            Action::ReduceSpeed,
            "양쪽 주행 끝의 어긋남".into(),
            "설정값",
        ));

        // 하중 상하 진동 (로프 탄성) vs 거더 고유진동수
        let l_min = c.samples.l.iter().cloned().fold(f64::MAX, f64::min);
        let l_max = c.samples.l.iter().cloned().fold(f64::MIN, f64::max);
        if l_min.is_finite() && l_max.is_finite() && spec.structure.girder_fn_hz > 0.0 {
            let m = (c.load_peak_kg + spec.structure.hook_mass_kg).max(1.0);
            let ea = spec.rope.axial_stiffness_ea_n * spec.hoist.reeving_falls;
            let fb = |l: f64| (ea / l.max(0.1) / m).sqrt() / (2.0 * std::f64::consts::PI);
            let (f_hi, f_lo) = (fb(l_min), fb(l_max));
            let fg = spec.structure.girder_fn_hz;
            let closest = if fg >= f_lo && fg <= f_hi {
                1.0
            } else if fg < f_lo {
                f_lo / fg
            } else {
                f_hi / fg
            };
            let dev = (closest - 1.0).abs();
            let sev = if dev <= spec.limits.resonance_band { 1 } else { 0 };
            let l_star = ea / (m * (2.0 * std::f64::consts::PI * fg).powi(2));
            gates.push(gate(
                "resonance",
                "하중 상하 진동 / 거더 고유진동수",
                closest,
                1.0 + spec.limits.resonance_band,
                "×",
                sev,
                // 상하 진동수는 속도로 바뀌지 않는다: 그 로프 길이 근처에서 권상 가감속을 부드럽게
                Action::ReduceAcceleration,
                format!(
                    "로프 {:.1}–{:.1} m 에서 {:.2}–{:.2} Hz, 거더 {:.2} Hz — 로프 {:.1} m 근처에서 권상 시작 · 정지를 부드럽게",
                    l_min, l_max, f_lo, f_hi, fg, l_star
                ),
                "로프 EA·줄 수 기준 상하 진동, 거더 측정 고유진동수",
            ));
        }
    }

    // 바람
    let wind = inp.wind_mps.or(inp.cycle.map(|c| c.max_wind_mps)).unwrap_or(0.0);
    if lim.wind_in_service_mps > 0.0 {
        let sev = if wind >= lim.wind_in_service_mps {
            3
        } else if wind >= 0.8 * lim.wind_in_service_mps {
            1
        } else {
            0
        };
        gates.push(gate(
            "wind",
            "풍속 (3초 돌풍)",
            wind,
            lim.wind_in_service_mps,
            "m/s",
            sev,
            if sev >= 3 { Action::Park } else { Action::ReduceSpeed },
            "운전 허용 풍속 대비".into(),
            "EN 13001-2:2021 표 5 '보통' 운전 풍 3초 돌풍 20 m/s",
        ));
    }

    // 듀티: 사이클 빈도 · 설계 수명
    if let Some(du) = inp.duty {
        if let Some(f) = fem_duty_limits(&spec.class.mechanism_class) {
            let cph = du.cycles as f64 / du.hours.max(1e-9);
            gates.push(gate(
                "cycles_per_hour",
                "시간당 사이클",
                cph,
                f.max_cycles_per_hour,
                "회/h",
                if cph > f.max_cycles_per_hour { 1 } else { 0 },
                Action::CoolDown,
                format!("기구 등급 {}", spec.class.mechanism_class),
                "FEM 듀티 표",
            ));
        }
        let remaining = 1.0 - du.swp_used;
        let sev = if remaining <= 0.0 {
            3
        } else if remaining <= lim.swp_overhaul_remaining {
            2
        } else if remaining <= lim.swp_warn_remaining {
            1
        } else {
            0
        };
        gates.push(gate(
            "swp",
            "남은 설계 수명 (SWP)",
            remaining,
            lim.swp_overhaul_remaining,
            "",
            sev,
            if sev >= 3 { Action::Stop } else { Action::Inspect },
            format!("전부하 환산 {:.1} h / 설계 {:.0} h ({})", du.full_load_hours, du.design_full_load_hours, spec.class.mechanism_class),
            "ISO 4301-1 · ISO 12482, Demag 30% · 10% 경보, Konecranes 10% 종합 정비",
        ));
    }

    // 시계열 예측: 임계 도달이 예측 수평선 안에 있는가
    for f in inp.forecasts.iter() {
        let Some(th) = f.threshold.as_ref() else { continue };
        let sev = if th.already_exceeded || th.p50_t.is_some() {
            2
        } else if th.p90_t.is_some() {
            1
        } else {
            0
        };
        let act = if f.key.contains("winding") { Action::CoolDown } else { Action::Inspect };
        let when = th.p50_t.or(th.p90_t);
        gates.push(gate(
            &format!("forecast_{}", f.key),
            &format!("예측: {}", f.key),
            f.point.last().cloned().unwrap_or(0.0),
            th.limit,
            &f.unit,
            sev,
            act,
            match when {
                Some(t) => format!("{} 방법, 임계 {:.3} 도달 예상 t = {:.2} {}", f.method, th.limit, t, f.t_unit),
                None => format!("{} 방법, 수평선 안 임계 도달 없음", f.method),
            },
            "시계열 예측 (물리 사전값 + 잔차)",
        ));
    }

    // SDS 드리프트
    for (name, z) in inp.drift.iter() {
        let sev = sev_ge(z.abs(), &[(3.0, 1), (4.0, 2)]);
        if sev > 0 {
            gates.push(gate(
                &format!("drift_{name}"),
                &format!("기준선 이탈: {name}"),
                *z,
                3.0,
                "z",
                sev,
                Action::Inspect,
                "최근 값이 누적 평균에서 벗어났습니다".into(),
                "SDS 원장 (Welford + 링버퍼)",
            ));
        }
    }

    // 규칙 판단: 초과(2) · 위험(3) 게이트만 조치를 정한다. 관찰(1) 게이트는 조언으로만 이유에 붙인다
    // (예: 브레이크 여유 30% = 정비 계획, 15% = 점검).
    let binding: Vec<&Gate> = gates.iter().filter(|g| g.severity >= 2).collect();
    let rule_action = binding.iter().map(|g| g.action).max_by_key(|a| (a.conservativeness(), *a)).unwrap_or(Action::Continue);
    let mut reasons: Vec<String> = binding
        .iter()
        .map(|g| format!("{}: {:.3} {} (한계 {:.3}) — {}", g.label, g.value, g.unit, g.limit, g.message))
        .collect();
    reasons.extend(
        gates
            .iter()
            .filter(|g| g.severity == 1)
            .map(|g| format!("(관찰) {}: {:.3} {} (한계 {:.3}) — {} → 권장: {}", g.label, g.value, g.unit, g.limit, g.message, g.action.label())),
    );
    reasons.dedup();
    let speed_scale = if binding.iter().any(|g| g.action == Action::ReduceSpeed) { Some(0.8) } else { None };
    let accel_scale = if binding.iter().any(|g| g.action == Action::ReduceAcceleration) { Some(0.7) } else { None };
    if inp.cycle.is_none() && inp.duty.is_none() {
        notes.push("사이클 · 듀티 결과 없이 판정했습니다".into());
    }
    let st = state_text(inp, &gates);
    DecisionReport {
        rule_action,
        rule_action_label: rule_action.label().into(),
        rule_reasons: reasons,
        speed_scale,
        accel_scale,
        state_text: st,
        gates,
        advisor: None,
        advisor_action: None,
        advisor_error: None,
        advisor_allowed: false,
        final_action: rule_action,
        final_action_label: rule_action.label().into(),
        escalated_by_advisor: false,
        agreement: None,
        notes,
        operator_choice: None,
        system_action: None,
        choice: None,
    }
}

fn state_text(inp: &DecisionInput, gates: &[Gate]) -> String {
    let spec = inp.spec;
    let mut s = format!(
        "Gantry crane state. Rated load {:.0} kg, span {:.0} m, mechanism class {}. Anti-sway mode: {}. ",
        spec.structure.rated_load_kg,
        spec.structure.span_m,
        spec.class.mechanism_class,
        inp.antisway_mode.key()
    );
    for g in gates.iter() {
        let status = match g.severity {
            0 => "ok",
            1 => "watch",
            2 => "over limit",
            _ => "critical",
        };
        let id = g.id.as_str();
        let name = if id == "overload" {
            "load ratio to rated capacity".to_string()
        } else if let Some(a) = id.strip_prefix("motor_temp_") {
            format!("{a} motor winding temperature")
        } else if let Some(a) = id.strip_prefix("motor_current_") {
            format!("{a} motor rms current ratio")
        } else if let Some(a) = id.strip_prefix("motor_torque_") {
            format!("{a} motor torque utilisation")
        } else if let Some(a) = id.strip_prefix("brake_lining_") {
            format!("{a} brake lining remaining fraction")
        } else if id == "rope_damage" {
            "wire rope bending fatigue damage sum".into()
        } else if id == "rope_safety" {
            "wire rope safety factor".into()
        } else if id == "sway" {
            "residual load sway in millimetres".into()
        } else if id == "wind" {
            "wind gust speed".into()
        } else if id == "swp" {
            "remaining safe working period fraction".into()
        } else if id == "skew" {
            "gantry skew in millimetres".into()
        } else {
            id.replace('_', " ")
        };
        s.push_str(&format!(
            "{}: {:.3} {} against limit {:.3} ({}). ",
            name,
            g.value,
            g.unit.replace('°', " deg ").replace('×', "x").replace("회/h", "per hour"),
            g.limit,
            status
        ));
    }
    if let Some(c) = inp.cycle {
        let e: f64 = c.axes.iter().map(|a| a.energy_in_kwh).sum();
        s.push_str(&format!("Cycle time {:.0} s, energy {:.2} kWh. ", c.cycle_time_s, e));
    }
    s
}

/// 작업자가 고를 수 있는가 — 규칙 판단보다 덜 보수적인 조치는 고를 수 없다
pub fn operator_can_choose(rep: &DecisionReport, a: Action) -> bool {
    a.conservativeness() >= rep.rule_action.conservativeness()
}

/// LAYA 조언을 붙인다. 조언은 규칙보다 보수적일 때만, 확률이 충분할 때만 '점검' 단계까지 올린다.
pub fn apply_advisor(rep: &mut DecisionReport, adv: &dyn Advisor, allow_escalation: bool) {
    rep.advisor_allowed = allow_escalation;
    let options: Vec<(String, String)> = Action::all().iter().map(|a| (a.key().to_string(), a.english().to_string())).collect();
    match adv.ask_choice(
        "Which action should the crane operator take next to keep the crane, its drives, brakes and wire rope safe and within limits?",
        &options,
        &rep.state_text,
    ) {
        Ok(ans) => {
            let act = Action::from_key(&ans.top);
            rep.agreement = act.map(|a| a == rep.rule_action);
            if let Some(a) = act {
                if allow_escalation && a.conservativeness() > rep.rule_action.conservativeness() && ans.top_probability >= 0.6 && ans.act_probability >= 0.5 {
                    let escalated = if rep.rule_action.conservativeness() >= Action::Inspect.conservativeness() { rep.rule_action } else { Action::Inspect };
                    if escalated != rep.final_action {
                        rep.final_action = escalated;
                        rep.final_action_label = escalated.label().into();
                        rep.escalated_by_advisor = true;
                    }
                }
            }
            rep.advisor_action = act;
            rep.advisor = Some(ans);
        }
        Err(e) => rep.advisor_error = Some(e),
    }
}

/// 작업자 선택을 반영한다
pub fn apply_operator_choice(rep: &mut DecisionReport, a: Action) -> Result<(), String> {
    if !operator_can_choose(rep, a) {
        return Err(format!("안전 게이트 판단 '{}' 보다 덜 보수적인 '{}' 은 고를 수 없습니다", rep.rule_action.label(), a.label()));
    }
    if rep.system_action.is_none() {
        rep.system_action = Some(rep.final_action);
    }
    rep.operator_choice = Some(a);
    rep.final_action = a;
    rep.final_action_label = a.label().into();
    Ok(())
}

/// 운전 판단 선택 카드
pub fn choice_prompt(rep: &DecisionReport, operator_rate: Option<(f64, u64)>, advisor_rate: Option<(f64, u64)>) -> crate::choice::ChoicePrompt {
    use crate::choice::{push_rate, ChoiceOption, ChoicePrompt};
    let system = rep.system_action.unwrap_or(rep.final_action);
    let prob = |a: Action| rep.advisor.as_ref().and_then(|ans| ans.labels.iter().position(|k| k == a.key()).and_then(|i| ans.probabilities.get(i).copied()));
    let floor_gates: Vec<String> = rep
        .gates
        .iter()
        .filter(|g| g.severity > 0 && g.action == rep.rule_action)
        .map(|g| format!("{} ({})", g.label, ["통과", "관찰", "초과", "위험"][g.severity.min(3) as usize]))
        .collect();
    let detail = |a: Action| -> String {
        match a {
            Action::ReduceSpeed => format!("{} — 속도 ×{:.2}", a.detail(), rep.speed_scale.unwrap_or(0.9)),
            Action::ReduceAcceleration => format!("{} — 가감속 ×{:.2}", a.detail(), rep.accel_scale.unwrap_or(0.8)),
            _ => a.detail().to_string(),
        }
    };
    let options: Vec<ChoiceOption> = Action::all()
        .into_iter()
        .map(|a| {
            let enabled = operator_can_choose(rep, a);
            // 성격: 규칙 판단보다 덜 보수적이면 위험, 멈추는 조치는 중단, 위험을 줄이며 계속하면 안전
            let stance = if a.conservativeness() < rep.rule_action.conservativeness() {
                "danger"
            } else {
                match a {
                    Action::Continue => "proceed",
                    Action::Inspect | Action::Park | Action::Stop => "stop",
                    _ => "safe",
                }
            };
            ChoiceOption {
                key: a.key().into(),
                label: a.label().into(),
                detail: detail(a),
                recommended: a == system,
                rule: a == rep.rule_action,
                advisor_probability: prob(a),
                advisor_top: rep.advisor_action == Some(a),
                enabled,
                blocked_by: if enabled { Vec::new() } else { floor_gates.clone() },
                chosen: rep.operator_choice == Some(a),
                stance: stance.into(),
                outcome: crate::choice::stance_label(stance).into(),
            }
        })
        .collect();
    let mut attention = Vec::new();
    if system.conservativeness() >= Action::Inspect.conservativeness() {
        attention.push(format!("{} 이상 조치가 필요합니다", Action::Inspect.label()));
    } else if system.conservativeness() >= 1 {
        attention.push(format!("운전 조건 조정이 필요합니다 ({})", system.label()));
    }
    if let (Some(false), Some(aa)) = (rep.agreement, rep.advisor_action) {
        attention.push(format!("LAYA 1위({})와 규칙 판단({})이 다릅니다", aa.label(), rep.rule_action.label()));
    }
    if rep.escalated_by_advisor {
        attention.push("LAYA 조언으로 점검까지 상향됐습니다".into());
    }
    let mut agreement = Vec::new();
    push_rate(&mut agreement, "규칙–LAYA", advisor_rate);
    push_rate(&mut agreement, "작업자–LAYA", operator_rate);
    let mut c = ChoicePrompt {
        kind: "operation".into(),
        header: "운전 판단".into(),
        question: "다음 조치를 무엇으로 할까요?".into(),
        options,
        recommended: system.key().into(),
        decided_by: if rep.operator_choice.is_some() {
            "operator".into()
        } else if rep.escalated_by_advisor {
            "rule+laya".into()
        } else {
            "rule".into()
        },
        needs_attention: false,
        attention,
        operator_choice: rep.operator_choice.map(|a| a.key().to_string()),
        advisor_note: match (rep.advisor.as_ref(), rep.advisor_error.as_ref()) {
            (Some(ans), _) => format!(
                "LAYA 1위 {} {:.0}% · 행동 확률 {:.2}",
                Action::from_key(&ans.top).map(|a| a.label()).unwrap_or(ans.top.as_str()),
                ans.top_probability * 100.0,
                ans.act_probability
            ),
            (None, Some(e)) => e.clone(),
            (None, None) => "LAYA 조언 미사용".into(),
        },
        agreement,
        procedure: "인양 · 이동".into(),
        situation: match system.conservativeness() {
            0 => "safe",
            1 => "proceed",
            2 => "danger",
            _ => "stop",
        }
        .into(),
    };
    c.sort_options(|k| Action::from_key(k).map(|a| a.conservativeness()).unwrap_or(9));
    c.settle_attention();
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cyclesim::{self, demo_scenario};

    fn decide(c: &CycleReport, spec: &CraneSpec, mode: AntiSwayMode, wind: Option<f64>) -> DecisionReport {
        evaluate(&DecisionInput { spec, cycle: Some(c), duty: None, forecasts: &[], wind_mps: wind, antisway_mode: mode, drift: &[] })
    }

    #[test]
    fn nominal_cycle_mostly_passes_and_wind_parks() {
        let sc = demo_scenario("gantry_10t_20m", 6000.0, AntiSwayMode::Zvd);
        let spec = sc.crane.resolve().unwrap();
        let c = cyclesim::simulate(&sc).unwrap();
        let rep = decide(&c, &spec, AntiSwayMode::Zvd, None);
        assert!(rep.rule_action.conservativeness() <= 1, "{:?} {:?}", rep.rule_action, rep.rule_reasons);
        let windy = decide(&c, &spec, AntiSwayMode::Zvd, Some(22.0));
        assert_eq!(windy.rule_action, Action::Park);
    }

    #[test]
    fn watch_level_gates_advise_but_do_not_bind() {
        // 정격 하중의 천장 크레인: 공진 관찰 게이트가 걸려도 규칙 판단은 '계속 운전'
        let sc = demo_scenario("overhead_5t_16m", 5000.0, AntiSwayMode::Zvd);
        let spec = sc.crane.resolve().unwrap();
        let c = cyclesim::simulate(&sc).unwrap();
        let rep = decide(&c, &spec, AntiSwayMode::Zvd, None);
        assert!(rep.gates.iter().all(|g| g.severity < 2), "{:?}", rep.rule_reasons);
        assert_eq!(rep.rule_action, Action::Continue);
        assert!(operator_can_choose(&rep, Action::Continue));
        if rep.gates.iter().any(|g| g.severity == 1) {
            assert!(rep.rule_reasons.iter().any(|r| r.starts_with("(관찰)")));
        }
    }

    #[test]
    fn overload_limits_load() {
        let sc = demo_scenario("overhead_5t_16m", 5600.0, AntiSwayMode::Zvd);
        let spec = sc.crane.resolve().unwrap();
        let c = cyclesim::simulate(&sc).unwrap();
        let rep = decide(&c, &spec, AntiSwayMode::Zvd, None);
        let g = rep.gates.iter().find(|g| g.id == "overload").unwrap();
        assert_eq!(g.severity, 3);
        assert!(rep.rule_action.conservativeness() >= Action::LimitLoad.conservativeness());
    }

    struct Scared;
    impl Advisor for Scared {
        fn ask_choice(&self, _: &str, o: &[(String, String)], _: &str) -> Result<AdvisorAnswer, String> {
            Ok(AdvisorAnswer { labels: o.iter().map(|x| x.0.clone()).collect(), probabilities: vec![0.0; o.len()], top: "stop".into(), top_probability: 0.9, act_probability: 0.9 })
        }
    }

    #[test]
    fn advisor_escalates_only_to_inspect_and_operator_cannot_go_below_rule() {
        let sc = demo_scenario("gantry_10t_20m", 3000.0, AntiSwayMode::Zvd);
        let spec = sc.crane.resolve().unwrap();
        let c = cyclesim::simulate(&sc).unwrap();
        let mut rep = decide(&c, &spec, AntiSwayMode::Zvd, Some(22.0));
        assert_eq!(rep.rule_action, Action::Park);
        apply_advisor(&mut rep, &Scared, true);
        // 규칙이 이미 점검 이상이면 조언은 바꾸지 않는다
        assert_eq!(rep.final_action, Action::Park);
        assert!(apply_operator_choice(&mut rep, Action::Continue).is_err());
        assert!(apply_operator_choice(&mut rep, Action::Stop).is_ok());
        let mut calm = decide(&c, &spec, AntiSwayMode::Zvd, Some(0.0));
        let before = calm.rule_action;
        apply_advisor(&mut calm, &Scared, true);
        if before.conservativeness() < Action::Inspect.conservativeness() {
            assert_eq!(calm.final_action, Action::Inspect);
            assert!(calm.escalated_by_advisor);
        }
        let card = choice_prompt(&calm, None, None);
        assert_eq!(card.options.len(), Action::all().len());
        assert!(card.options.iter().any(|o| o.recommended));
    }
}

