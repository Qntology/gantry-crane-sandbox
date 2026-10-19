//! 듀티(교대 근무) 시뮬레이션.
//!
//! 하중 스펙트럼의 각 구간(정격 대비 하중 비율)마다 운반 사이클을 한 번씩 동역학으로 돌리고,
//! 그 손실 기록(대표 모터 1대의 동손 · 철손+기계손 · 속도)을 근무 시간 동안 이어 붙여 모터 열을 적분한다.
//! 브레이크 라이닝 마모 · 로프 손상 · 전부하 환산 시간은 구간별 사이클 증분을 더한다.
//! 사이클 순서는 시드가 있는 의사 난수로 정해 재현된다.
//!
//! 하중 스펙트럼 계수 Kp = Σ (C_i/C_T)·(P_i/P_max)³ (ISO 4301-1), 전부하 환산 시간 = Σ t_i·(P_i/P_max)³.

use crate::crane_profile::{Axis, CraneSpec};
use crate::cyclesim::{self, AxisLossProfile, CycleReport, Scenario};
use crate::motor::{MotorLosses, MotorThermal};
use crate::timeseries::Series;
use serde::{Deserialize, Serialize};

fn default_seed() -> u64 {
    7
}
fn default_tdt() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBin {
    /// 정격 하중 대비 비율 (0~1.25)
    pub fraction: f64,
    /// 사이클 비율 (합이 1 이 아니어도 정규화)
    pub share: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DutyState {
    pub brake_lining_mm: Vec<f64>,
    pub rope_damage: f64,
    pub full_load_hours: f64,
    pub cycles: f64,
    pub winding_c: Vec<f64>,
    pub frame_c: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DutyConfig {
    pub cycles_per_hour: f64,
    pub hours: f64,
    pub spectrum: Vec<LoadBin>,
    #[serde(default = "default_seed")]
    pub seed: u64,
    /// 열 적분 간격 (s)
    #[serde(default = "default_tdt")]
    pub thermal_dt_s: f64,
    /// 이전 근무에서 이어지는 상태
    #[serde(default)]
    pub initial: Option<DutyState>,
}

impl Default for DutyConfig {
    fn default() -> Self {
        Self {
            // 6 m 행정 · 5 m/min 권상의 데모 사이클(약 6분)이 휴지 시간을 갖도록
            cycles_per_hour: 6.0,
            hours: 8.0,
            spectrum: vec![
                LoadBin { fraction: 1.0, share: 0.1 },
                LoadBin { fraction: 0.6, share: 0.3 },
                LoadBin { fraction: 0.3, share: 0.4 },
                LoadBin { fraction: 0.1, share: 0.2 },
            ],
            seed: default_seed(),
            thermal_dt_s: default_tdt(),
            initial: None,
        }
    }
}

/// 하중 스펙트럼 계수 Kp
pub fn spectrum_factor(bins: &[LoadBin]) -> f64 {
    let total: f64 = bins.iter().map(|b| b.share.max(0.0)).sum();
    if total <= 0.0 {
        return 0.0;
    }
    bins.iter().map(|b| b.share.max(0.0) / total * b.fraction.clamp(0.0, 2.0).powi(3)).sum()
}

/// Kp 의 명목 등급 (ISO 4301-1:1986 Q1–Q4 / FEM 1.001 L1–L4 의 명목값 0.125 · 0.25 · 0.5 · 1.0 기준)
pub fn spectrum_class(kp: f64) -> &'static str {
    if kp <= 0.125 {
        "Q1/L1 (light)"
    } else if kp <= 0.25 {
        "Q2/L2 (moderate)"
    } else if kp <= 0.5 {
        "Q3/L3 (heavy)"
    } else {
        "Q4/L4 (very heavy)"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinRun {
    pub fraction: f64,
    pub load_kg: f64,
    pub share: f64,
    pub cycle_time_s: f64,
    pub energy_kwh: f64,
    pub regen_kwh: f64,
    pub rope_damage: f64,
    pub brake_wear_mm: Vec<f64>,
    pub hoist_run_s: f64,
    pub run_time_s: Vec<f64>,
    pub starts: Vec<u32>,
    pub max_residual_sway_mm: f64,
    pub peak_torque_ratio: Vec<f64>,
    #[serde(skip)]
    pub losses: Vec<AxisLossProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DutyRow {
    pub index: usize,
    pub t_end_h: f64,
    pub bin: usize,
    pub load_kg: f64,
    /// 이 사이클 동안의 권선 최고 온도 [주행, 횡행, 권상]
    pub peak_winding_c: Vec<f64>,
    pub energy_kwh_cum: f64,
    pub brake_lining_mm: Vec<f64>,
    pub rope_damage: f64,
    pub full_load_hours: f64,
}

/// 권선 최고 온도의 물리 사전값: T(t) = T_ss − (T_ss − T_0)·exp(−(t − t_0)/τ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalPrior {
    pub axis: Axis,
    pub t0_h: f64,
    pub start_c: f64,
    pub steady_peak_c: f64,
    pub tau_h: f64,
}

impl ThermalPrior {
    pub fn at(&self, t_h: f64) -> f64 {
        let dt = (t_h - self.t0_h).max(0.0);
        self.steady_peak_c - (self.steady_peak_c - self.start_c) * (-dt / self.tau_h.max(1e-6)).exp()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DutyReport {
    pub crane: String,
    pub scenario: String,
    pub cycles: usize,
    pub hours: f64,
    pub kp: f64,
    pub spectrum_class: String,
    pub bins: Vec<BinRun>,
    pub rows: Vec<DutyRow>,
    pub series: Vec<Series>,
    pub thermal_priors: Vec<ThermalPrior>,
    pub max_winding_c: Vec<f64>,
    pub insulation_limit_c: Vec<f64>,
    pub mean_cdf: Vec<f64>,
    pub rated_cdf: Vec<f64>,
    pub starts_per_hour: Vec<f64>,
    pub rated_starts_per_hour: Vec<f64>,
    pub energy_kwh: f64,
    pub regen_kwh: f64,
    pub rope_damage: f64,
    pub brake_lining_mm: Vec<f64>,
    pub brake_lining_min_mm: Vec<f64>,
    pub brake_lining_new_mm: Vec<f64>,
    pub full_load_hours: f64,
    pub design_full_load_hours: f64,
    /// 설계 수명 사용 비율 (전부하 환산 시간 / 설계 시간)
    pub swp_used: f64,
    pub total_cycles: f64,
    pub design_cycles: f64,
    pub final_state: DutyState,
    pub notes: Vec<String>,
}

struct Rng(u64);

impl Rng {
    fn next_f64(&mut self) -> f64 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        ((x.wrapping_mul(0x2545F4914F6CDD1D) >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

/// 걸기 경유점 · 시작 하중을 `load_kg` 로 바꾼 시나리오
pub fn scenario_with_load(base: &Scenario, load_kg: f64) -> (Scenario, bool) {
    let mut s = base.clone();
    let mut touched = false;
    if s.start.load_kg > 0.0 {
        s.start.load_kg = load_kg;
        touched = true;
    }
    for wp in s.waypoints.iter_mut() {
        if let Some(m) = wp.set_load_kg {
            if m > 0.0 {
                wp.set_load_kg = Some(load_kg);
                touched = true;
            }
        }
    }
    (s, touched)
}

fn downsample(p: &AxisLossProfile, dt_out: f64) -> AxisLossProfile {
    let k = ((dt_out / p.dt.max(1e-6)).round() as usize).max(1);
    let avg = |v: &Vec<f64>| -> Vec<f64> { v.chunks(k).map(|c| c.iter().sum::<f64>() / c.len() as f64).collect() };
    AxisLossProfile { axis: p.axis, dt: p.dt * k as f64, copper_w: avg(&p.copper_w), other_w: avg(&p.other_w), omega: avg(&p.omega) }
}

fn bin_run(spec: &CraneSpec, r: &CycleReport, fraction: f64, load_kg: f64, share: f64, dt_out: f64) -> BinRun {
    BinRun {
        fraction,
        load_kg,
        share,
        cycle_time_s: r.cycle_time_s,
        energy_kwh: r.axes.iter().map(|a| a.energy_in_kwh).sum(),
        regen_kwh: r.axes.iter().map(|a| a.energy_regen_kwh).sum(),
        rope_damage: r.rope.damage,
        brake_wear_mm: Axis::all().iter().map(|a| spec.drive(*a).brake.lining_new_mm - r.axes[a.index()].brake_lining_mm).collect(),
        hoist_run_s: r.axes[2].run_time_s,
        run_time_s: r.axes.iter().map(|a| a.run_time_s).collect(),
        starts: r.axes.iter().map(|a| a.starts).collect(),
        max_residual_sway_mm: r.moves.iter().map(|m| m.residual_sway_mm).fold(0.0, f64::max),
        peak_torque_ratio: r.axes.iter().map(|a| a.peak_torque_ratio).collect(),
        losses: r.loss_profiles.iter().map(|p| downsample(p, dt_out)).collect(),
    }
}

pub fn run(base: &Scenario, cfg: &DutyConfig) -> Result<DutyReport, String> {
    let spec = base.crane.resolve()?;
    if cfg.spectrum.is_empty() {
        return Err("하중 스펙트럼이 비어 있습니다".into());
    }
    if !(cfg.cycles_per_hour > 0.0 && cfg.hours > 0.0) {
        return Err("cycles_per_hour 와 hours 는 양수여야 합니다".into());
    }
    let mut notes = Vec::new();
    let rated = spec.structure.rated_load_kg;
    let hook = spec.structure.hook_mass_kg;
    let amb = base.env.ambient_c;
    let tdt = cfg.thermal_dt_s.clamp(0.05, 5.0);

    // 1) 하중 구간별 사이클 한 번씩
    let mut bins = Vec::new();
    let mut any_load = false;
    let share_total: f64 = cfg.spectrum.iter().map(|b| b.share.max(0.0)).sum::<f64>().max(1e-12);
    for b in cfg.spectrum.iter() {
        let load = (b.fraction.max(0.0) * rated).round();
        let (sc, touched) = scenario_with_load(base, load);
        any_load |= touched;
        let r = cyclesim::simulate(&sc)?;
        bins.push(bin_run(&spec, &r, b.fraction, load, b.share.max(0.0) / share_total, tdt));
    }
    if !any_load {
        notes.push("시나리오에 하중 걸기 경유점(set_load_kg > 0)이나 시작 하중이 없어 스펙트럼이 적용되지 않았습니다".into());
    }

    // 2) 사이클 순서
    let period_s = 3600.0 / cfg.cycles_per_hour;
    let n_cycles = (cfg.hours * cfg.cycles_per_hour).round().max(1.0) as usize;
    let mut rng = Rng(cfg.seed.max(1).wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let pick = |u: f64| -> usize {
        let mut acc = 0.0;
        for (i, b) in bins.iter().enumerate() {
            acc += b.share;
            if u <= acc {
                return i;
            }
        }
        bins.len() - 1
    };

    // 3) 상태
    let init = cfg.initial.clone().unwrap_or_default();
    let mut thermal: Vec<MotorThermal> = (0..3)
        .map(|k| MotorThermal {
            winding_c: init.winding_c.get(k).cloned().unwrap_or(amb),
            frame_c: init.frame_c.get(k).cloned().unwrap_or(amb),
        })
        .collect();
    let mut lining: Vec<f64> = Axis::all()
        .iter()
        .map(|a| init.brake_lining_mm.get(a.index()).cloned().unwrap_or(spec.drive(*a).brake.lining_new_mm))
        .collect();
    let mut rope_damage = init.rope_damage;
    let mut flh = init.full_load_hours;
    let mut energy = 0.0;
    let mut regen = 0.0;
    let mut t_s = 0.0;
    let mut rows = Vec::with_capacity(n_cycles);
    let mut max_w = vec![amb; 3];
    let mut run_sum = vec![0.0; 3];
    let mut starts_sum = vec![0.0; 3];
    let p_max = rated + hook;

    for idx in 0..n_cycles {
        let bi = pick(rng.next_f64());
        let b = &bins[bi];
        let mut peak = vec![f64::MIN; 3];
        for a in Axis::all() {
            let k = a.index();
            let d = spec.drive(a);
            let lp = &b.losses[k];
            for i in 0..lp.copper_w.len() {
                let l = MotorLosses { copper_w: lp.copper_w[i], iron_w: lp.other_w[i], mech_w: 0.0 };
                thermal[k].step(&d.motor, &l, lp.omega[i], amb, lp.dt);
                peak[k] = peak[k].max(thermal[k].winding_c);
            }
            run_sum[k] += b.run_time_s[k];
            starts_sum[k] += b.starts[k] as f64;
        }
        let idle = (period_s - b.cycle_time_s).max(0.0);
        if idle > 0.0 {
            for a in Axis::all() {
                let k = a.index();
                let d = spec.drive(a);
                let zero = MotorLosses::default();
                thermal[k].step(&d.motor, &zero, 0.0, amb, idle);
                peak[k] = peak[k].max(thermal[k].winding_c);
            }
        }
        t_s += b.cycle_time_s + idle;
        for k in 0..3 {
            lining[k] = (lining[k] - b.brake_wear_mm[k]).max(0.0);
            max_w[k] = max_w[k].max(peak[k]);
        }
        rope_damage += b.rope_damage;
        energy += b.energy_kwh;
        regen += b.regen_kwh;
        flh += b.hoist_run_s / 3600.0 * ((b.load_kg + hook) / p_max).powi(3);
        rows.push(DutyRow {
            index: idx,
            t_end_h: t_s / 3600.0,
            bin: bi,
            load_kg: b.load_kg,
            peak_winding_c: peak.clone(),
            energy_kwh_cum: energy,
            brake_lining_mm: lining.clone(),
            rope_damage,
            full_load_hours: flh,
        });
    }

    // 4) 예측용 시계열 (시간축 = 근무 시간 h)
    let new_mm: Vec<f64> = Axis::all().iter().map(|a| spec.drive(*a).brake.lining_new_mm).collect();
    let mk = |key: &str, unit: &str, f: &dyn Fn(&DutyRow) -> f64| -> Series {
        let mut s = Series::new(key, unit).with_time_unit("h");
        for r in rows.iter() {
            s.push(r.t_end_h, f(r));
        }
        s
    };
    let series = vec![
        mk("travel_winding_peak_c", "°C", &|r| r.peak_winding_c[0]),
        mk("trolley_winding_peak_c", "°C", &|r| r.peak_winding_c[1]),
        mk("hoist_winding_peak_c", "°C", &|r| r.peak_winding_c[2]),
        mk("rope_damage", "", &|r| r.rope_damage),
        // 브레이크는 '마모량'(증가)으로 내보내 임계 도달 예측이 같은 방향이 되게 한다
        mk("brake_wear_travel_mm", "mm", &|r| new_mm[0] - r.brake_lining_mm[0]),
        mk("brake_wear_trolley_mm", "mm", &|r| new_mm[1] - r.brake_lining_mm[1]),
        mk("brake_wear_hoist_mm", "mm", &|r| new_mm[2] - r.brake_lining_mm[2]),
        mk("energy_kwh", "kWh", &|r| r.energy_kwh_cum),
    ];

    // 5) 권선 최고 온도의 물리 사전값 — 평균 손실의 정상상태 + 사이클 내 맥동
    let thermal_priors = Axis::all()
        .iter()
        .map(|a| {
            let k = a.index();
            let d = spec.drive(*a);
            let net = d.motor.thermal_network();
            let (mut pcu, mut po, mut kc, mut tt) = (0.0, 0.0, 0.0, 0.0);
            for b in bins.iter() {
                let lp = &b.losses[k];
                let idle = (period_s - b.cycle_time_s).max(0.0);
                let mut e_cu = 0.0;
                let mut e_o = 0.0;
                let mut c = 0.0;
                for i in 0..lp.copper_w.len() {
                    e_cu += lp.copper_w[i] * lp.dt;
                    e_o += lp.other_w[i] * lp.dt;
                    c += d.motor.cooling_factor(lp.omega[i]) * lp.dt;
                }
                c += d.motor.cooling_factor(0.0) * idle;
                let span = b.cycle_time_s + idle;
                pcu += b.share * e_cu;
                po += b.share * e_o;
                kc += b.share * c;
                tt += b.share * span;
            }
            let tt = tt.max(1e-9);
            let (pcu, po, kbar) = (pcu / tt, po / tt, (kc / tt).max(0.05));
            let r_fa = net.r_fa / kbar;
            let t_frame = amb + (pcu + po) * r_fa;
            let t_mean = t_frame + pcu * net.r_wf;
            // 맥동: 마지막 구간들의 사이클 최고 온도 − 그 시점 평균 추정
            let ripple = rows.last().map(|r| (r.peak_winding_c[k] - thermal[k].winding_c).max(0.0)).unwrap_or(0.0);
            ThermalPrior {
                axis: *a,
                t0_h: rows.first().map(|r| r.t_end_h).unwrap_or(0.0),
                start_c: rows.first().map(|r| r.peak_winding_c[k]).unwrap_or(amb),
                steady_peak_c: t_mean + ripple,
                tau_h: d.motor.thermal.tau_frame_s / kbar / 3600.0,
            }
        })
        .collect();

    let hours = t_s / 3600.0;
    let total_cycles = init.cycles + n_cycles as f64;
    let final_state = DutyState {
        brake_lining_mm: lining.clone(),
        rope_damage,
        full_load_hours: flh,
        cycles: total_cycles,
        winding_c: thermal.iter().map(|t| t.winding_c).collect(),
        frame_c: thermal.iter().map(|t| t.frame_c).collect(),
    };
    Ok(DutyReport {
        crane: spec.name.clone(),
        scenario: base.name.clone(),
        cycles: n_cycles,
        hours,
        kp: spectrum_factor(&cfg.spectrum),
        spectrum_class: spectrum_class(spectrum_factor(&cfg.spectrum)).into(),
        series,
        thermal_priors,
        max_winding_c: max_w,
        insulation_limit_c: Axis::all().iter().map(|a| spec.drive(*a).motor.insulation_limit_c()).collect(),
        mean_cdf: run_sum.iter().map(|r| r / t_s.max(1e-9)).collect(),
        rated_cdf: Axis::all().iter().map(|a| spec.drive(*a).motor.duty.cdf_percent / 100.0).collect(),
        starts_per_hour: starts_sum.iter().map(|s| s / hours.max(1e-9)).collect(),
        rated_starts_per_hour: Axis::all().iter().map(|a| spec.drive(*a).motor.duty.starts_per_hour).collect(),
        energy_kwh: energy,
        regen_kwh: regen,
        rope_damage,
        brake_lining_mm: lining,
        brake_lining_min_mm: Axis::all().iter().map(|a| spec.drive(*a).brake.lining_min_mm).collect(),
        brake_lining_new_mm: Axis::all().iter().map(|a| spec.drive(*a).brake.lining_new_mm).collect(),
        full_load_hours: flh,
        design_full_load_hours: spec.class.design_full_load_hours,
        swp_used: flh / spec.class.design_full_load_hours.max(1e-9),
        total_cycles,
        design_cycles: spec.class.design_cycles,
        final_state,
        bins,
        rows,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::antisway::AntiSwayMode;

    #[test]
    fn spectrum_factor_and_class() {
        let kp = spectrum_factor(&DutyConfig::default().spectrum);
        assert!((kp - 0.1758).abs() < 1e-4, "{kp}");
        assert_eq!(spectrum_class(kp), "Q2/L2 (moderate)");
        assert_eq!(spectrum_class(1.0), "Q4/L4 (very heavy)");
    }

    #[test]
    fn short_shift_heats_motors_and_accumulates_wear() {
        let base = cyclesim::demo_scenario("overhead_5t_16m", 4000.0, AntiSwayMode::Zvd);
        let cfg = DutyConfig { cycles_per_hour: 20.0, hours: 1.0, ..Default::default() };
        let r = run(&base, &cfg).unwrap();
        assert_eq!(r.cycles, 20);
        assert_eq!(r.bins.len(), 4);
        // 무거운 구간일수록 권상 에너지가 크다
        assert!(r.bins[0].energy_kwh > r.bins[3].energy_kwh);
        assert!(r.max_winding_c[2] > base.env.ambient_c + 1.0);
        assert!(r.rope_damage > 0.0);
        assert!(r.full_load_hours > 0.0);
        assert_eq!(r.series.len(), 8);
        assert_eq!(r.series[2].len(), 20);
        // 물리 사전값이 마지막 값에서 크게 벗어나지 않는다
        let p = &r.thermal_priors[2];
        let last = r.rows.last().unwrap();
        assert!((p.at(last.t_end_h) - last.peak_winding_c[2]).abs() < 15.0, "{} vs {}", p.at(last.t_end_h), last.peak_winding_c[2]);
    }
}
