//! scenarios/*.json 이 모두 읽히고 돌아가며, 시나리오마다 기대한 판정이 나오는지 확인한다.

use crane_model::cyclesim::{self, Scenario};
use crane_model::decision::{self, Action, DecisionInput};

fn load(name: &str) -> Scenario {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scenarios").join(name);
    let txt = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    serde_json::from_str(&txt).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn decide(sc: &Scenario) -> (cyclesim::CycleReport, decision::DecisionReport) {
    let spec = sc.crane.resolve().unwrap();
    let r = cyclesim::simulate(sc).unwrap();
    let d = decision::evaluate(&DecisionInput { spec: &spec, cycle: Some(&r), duty: None, forecasts: &[], wind_mps: None, antisway_mode: sc.antisway.mode, drift: &[] });
    (r, d)
}

#[test]
fn every_scenario_file_parses_and_runs() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scenarios");
    let mut n = 0;
    for e in std::fs::read_dir(&dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().map(|x| x == "json").unwrap_or(false) {
            let sc = load(p.file_name().unwrap().to_str().unwrap());
            let r = cyclesim::simulate(&sc).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            assert!(r.cycle_time_s > 10.0, "{}", p.display());
            assert!(!r.samples.is_empty());
            n += 1;
        }
    }
    assert!(n >= 7, "scenario files: {n}");
}

#[test]
fn demo_passes_overload_limits_estop_inspects() {
    let (_, d) = decide(&load("gantry_10t_demo.json"));
    assert!(d.rule_action.conservativeness() <= 1, "{:?}", d.rule_reasons);
    let (_, d) = decide(&load("gantry_overload_11t.json"));
    assert_eq!(d.rule_action, Action::LimitLoad);
    let (r, d) = decide(&load("gantry_estop_during_travel.json"));
    assert!(r.estop);
    assert!(r.moves.iter().any(|m| m.aborted));
    assert_eq!(d.rule_action, Action::Inspect);
    assert!(r.axes[0].brake_dynamic_stops > 0);
}

#[test]
fn simultaneous_moves_are_faster_and_zvd_is_robust_to_frequency_error() {
    let (seq, _) = decide(&load("gantry_10t_demo.json"));
    let (sim, _) = decide(&load("gantry_simultaneous_moves.json"));
    assert!(sim.cycle_time_s < 0.85 * seq.cycle_time_s, "{} vs {}", sim.cycle_time_s, seq.cycle_time_s);
    let (err, _) = decide(&load("gantry_zvd_freq_error_15pct.json"));
    let worst = err.moves.iter().map(|m| m.residual_sway_mm).fold(0.0, f64::max);
    assert!(worst < 50.0, "ZVD with 15% frequency error left {worst} mm");
}
