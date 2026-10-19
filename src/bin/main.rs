//! crane_demo — 갠트리 크레인 샌드박스 명령줄 도구.
//!
//! 하위 명령은 `crane_demo help` 로 본다.

use crane_model::antisway::AntiSwayMode;
use crane_model::api::{self, Models, PipelineConfig};
use crane_model::calib;
use crane_model::crane_profile;
use crane_model::cyclesim::{self, CycleReport, Scenario};
use crane_model::decision::{self, DecisionInput};
use crane_model::duty::{self, DutyConfig};
use crane_model::ingest;
use crane_model::motor;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const HELP: &str = "\
crane_demo — 갠트리 크레인 샌드박스

사용:
  crane_demo [demo]                                   기본 시나리오로 전체 흐름을 한 번 돌린다
  crane_demo presets                                  크레인 · 모터 프리셋 (JSON)
  crane_demo scenario [--crane NAME] [--load KG] [--mode MODE]   시나리오 JSON 을 출력한다
  crane_demo simulate <scenario.json> [--mode MODE] [--out DIR]   사이클 하나
  crane_demo compare <scenario.json>                  흔들림 억제 방식 비교
  crane_demo duty <scenario.json> [--hours H] [--cph N] [--out DIR]
  crane_demo pipeline <scenario.json> [--hours H] [--cph N] [--forecast-h H] [--wind V]
                      [--models DIR] [--sds DIR] [--db FILE] [--sling M] [--compare] [--out DIR]
  crane_demo calibrate pendulum <log.csv> [--channel theta_x]
  crane_demo calibrate coastdown <log.csv> --mass KG [--channel vx]
  crane_demo calibrate winding <log.csv>              (current_a, speed_rpm, ambient_c 또는 coolant_c, winding_c 열)
  crane_demo calibrate kaggle <measures_v2.csv> [--profile N] [--ref coolant|ambient] [--max-rows N]
  crane_demo calibrate s3 [--motor NAME]              SEW S3 출력 배수표 재현 확인
  crane_demo crosscheck [--crane NAME] [--rope M] [--load KG] [--sling M]

MODE: off | zv | zvd | ei | closed_loop
";

struct Args {
    pos: Vec<String>,
    opt: HashMap<String, String>,
}

fn parse_args() -> Args {
    let mut pos = Vec::new();
    let mut opt = HashMap::new();
    let mut it = std::env::args().skip(1).peekable();
    while let Some(a) = it.next() {
        if let Some(k) = a.strip_prefix("--") {
            let v = match it.peek() {
                Some(n) if !n.starts_with("--") => it.next().unwrap(),
                _ => "true".into(),
            };
            opt.insert(k.to_string(), v);
        } else {
            pos.push(a);
        }
    }
    Args { pos, opt }
}

impl Args {
    fn f(&self, k: &str) -> Option<f64> {
        self.opt.get(k).and_then(|v| v.parse().ok())
    }
    fn s(&self, k: &str) -> Option<&str> {
        self.opt.get(k).map(|s| s.as_str())
    }
}

fn mode_of(s: &str) -> Result<AntiSwayMode, String> {
    AntiSwayMode::all().into_iter().find(|m| m.key() == s.trim()).ok_or_else(|| format!("모르는 흔들림 억제 방식: {s}"))
}

fn read_scenario(path: &str) -> Result<Scenario, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&txt).map_err(|e| format!("{path}: {e}"))
}

fn write(dir: &Path, name: &str, content: &str) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let p = dir.join(name);
    std::fs::write(&p, content).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(p)
}

fn print_cycle(r: &CycleReport) {
    println!("■ 사이클: {} · 크레인 {} · 흔들림 억제 {}", r.scenario, r.crane, r.antisway);
    println!("  사이클 시간 {:.1} s, 최대 하중 {:.0} kg / 정격 {:.0} kg", r.cycle_time_s, r.load_peak_kg, r.rated_load_kg);
    println!("  {:<10} {:>8} {:>10} {:>10} {:>10} {:>9}", "이동", "시간 s", "최대 mm", "잔류 mm", "끝 mm", "안정 s");
    for m in r.moves.iter() {
        println!("  {:<10} {:>8.1} {:>10.0} {:>10.0} {:>10.0} {:>9.1}", m.label, m.duration_s, m.max_sway_mm, m.residual_sway_mm, m.final_sway_mm, m.settle_time_s);
    }
    println!("  {:<6} {:>9} {:>9} {:>9} {:>9} {:>9} {:>8}", "축", "토크비", "피크 A", "RMS A", "kWh", "회생kWh", "권선°C");
    for a in r.axes.iter() {
        println!(
            "  {:<6} {:>9.2} {:>9.1} {:>9.1} {:>9.3} {:>9.3} {:>8.1}",
            a.axis.label(),
            a.peak_torque_ratio,
            a.peak_current_a,
            a.rms_current_a,
            a.energy_in_kwh,
            a.energy_regen_kwh,
            a.peak_winding_c
        );
    }
    println!(
        "  로프: 줄당 최대 {:.1} kN, 안전율 {:.1} (요구 {:.1}), 행정 {} 회, 손상 {:.2e}",
        r.rope.max_tension_per_fall_kn, r.rope.min_safety_factor, r.rope.required_safety_factor, r.rope.strokes, r.rope.damage
    );
    println!("  편행 최대 {:.1} mm, 바람 최대 {:.1} m/s, 로프 느슨함 {} 회", r.max_skew_mm, r.max_wind_mps, r.slack_events);
    for n in r.notes.iter() {
        println!("  메모: {n}");
    }
}

fn print_decision(rep: &decision::DecisionReport) {
    println!("■ 판정: 규칙 '{}' → 최종 '{}'", rep.rule_action_label, rep.final_action_label);
    for g in rep.gates.iter().filter(|g| g.severity > 0) {
        let s = ["통과", "관찰", "초과", "위험"][g.severity.min(3) as usize];
        println!("  [{}] {} = {:.3} {} (한계 {:.3}) — {}", s, g.label, g.value, g.unit, g.limit, g.message);
    }
    if rep.gates.iter().all(|g| g.severity == 0) {
        println!("  모든 게이트 통과 ({} 개)", rep.gates.len());
    }
}

fn save_cycle(out: &Path, r: &CycleReport) -> Result<(), String> {
    let p1 = write(out, "cycle.json", &serde_json::to_string_pretty(r).map_err(|e| e.to_string())?)?;
    let p2 = write(out, "samples.csv", &r.samples.to_csv())?;
    let p3 = write(out, "motion_program.json", &serde_json::to_string_pretty(&r.program).map_err(|e| e.to_string())?)?;
    println!("  저장: {} · {} · {}", p1.display(), p2.display(), p3.display());
    Ok(())
}

fn duty_cfg(a: &Args) -> DutyConfig {
    let mut d = DutyConfig::default();
    if let Some(h) = a.f("hours") {
        d.hours = h;
    }
    if let Some(c) = a.f("cph") {
        d.cycles_per_hour = c;
    }
    d
}

fn cmd_demo() -> Result<(), String> {
    println!("══════════════════════════════════════════");
    println!("  갠트리 크레인 샌드박스 데모");
    println!("══════════════════════════════════════════\n");
    let sc = cyclesim::demo_scenario("gantry_10t_20m", 8000.0, AntiSwayMode::Zvd);
    println!("■ 흔들림 억제 방식 비교 (8 t 하중, 갠트리 10 t × 20 m)");
    println!("  {:<12} {:>9} {:>11} {:>9} {:>9} {:>8}", "방식", "사이클 s", "잔류 mm", "끝 mm", "안정 s", "kWh");
    for r in api::compare(&sc, &AntiSwayMode::all())? {
        println!("  {:<12} {:>9.1} {:>11.0} {:>9.0} {:>9.1} {:>8.3}", r.mode, r.cycle_time_s, r.max_residual_sway_mm, r.max_final_sway_mm, r.max_settle_time_s, r.energy_kwh);
    }
    println!();
    let cfg = PipelineConfig { duty: Some(DutyConfig { hours: 8.0, cycles_per_hour: 6.0, ..Default::default() }), forecast_hours: 8.0, ..Default::default() };
    let rep = api::run_pipeline(&sc, &cfg, &Models::none())?;
    print_cycle(&rep.cycle);
    if let Some(d) = &rep.duty {
        println!(
            "■ 근무 {:.1} h · {} 사이클 · Kp {:.3} ({}) · 권상 권선 최고 {:.1} °C · 로프 손상 {:.2e} · 전부하 환산 {:.2} h",
            d.hours, d.cycles, d.kp, d.spectrum_class, d.max_winding_c[2], d.rope_damage, d.full_load_hours
        );
    }
    for f in rep.forecasts.iter().filter(|f| f.threshold.is_some()) {
        let th = f.threshold.as_ref().unwrap();
        let when = th.p50_t.map(|t| format!("p50 {:.1} h", t)).unwrap_or_else(|| "수평선 안 도달 없음".into());
        println!("  예측 {:<24} {:<12} 임계 {:.3} → {}", f.key, f.method, th.limit, when);
    }
    print_decision(&rep.decision);
    if let Some(c) = &rep.crosscheck {
        println!(
            "■ 교차검증: 로프 {:.1} m, 단진자 {:.4} Hz vs 정밀 해석 {:.4} Hz (비 {:.4}), 상하 진동 {:.2} Hz, 정적 늘음 {:.1} mm",
            c.rope_length_m, c.analytic_freq_hz, c.mbd_freq_hz, c.freq_ratio, c.bounce_freq_hz, c.static_stretch_mm
        );
    }
    for n in rep.notes.iter() {
        println!("  메모: {n}");
    }
    Ok(())
}

fn cmd_calibrate(a: &Args) -> Result<(), String> {
    let what = a.pos.get(1).map(|s| s.as_str()).unwrap_or("");
    match what {
        "s3" => {
            let ms: Vec<motor::MotorParams> = match a.s("motor") {
                Some(n) => vec![motor::preset(n).ok_or(format!("모터 프리셋 {n} 없음"))?],
                None => motor::presets(),
            };
            for m in ms {
                let pts: Vec<String> = motor::SEW_S3_FACTORS.iter().map(|(c, k)| format!("{:.0}% {:.3} (SEW {:.2})", c * 100.0, motor::s3_power_factor(&m, *c, 600.0), k)).collect();
                println!("{:<14} {}", m.name, pts.join(" · "));
            }
            Ok(())
        }
        "pendulum" => {
            let path = a.pos.get(2).ok_or("로그 파일을 주세요")?;
            let log = ingest::load_crane_log(&std::fs::read_to_string(path).map_err(|e| e.to_string())?, 0.01)?;
            let ch = a.s("channel").unwrap_or("theta_x");
            let (t, y) = log.pair(ch).ok_or(format!("'{ch}' 채널이 없습니다"))?;
            let r = calib::fit_pendulum(&t, &y)?;
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            Ok(())
        }
        "coastdown" => {
            let path = a.pos.get(2).ok_or("로그 파일을 주세요")?;
            let m = a.f("mass").ok_or("--mass KG 가 필요합니다")?;
            let log = ingest::load_crane_log(&std::fs::read_to_string(path).map_err(|e| e.to_string())?, 0.01)?;
            let ch = a.s("channel").unwrap_or("vx");
            let (t, v) = log.pair(ch).ok_or(format!("'{ch}' 채널이 없습니다"))?;
            println!("{}", serde_json::to_string_pretty(&calib::fit_coastdown(&t, &v, m)?).unwrap());
            Ok(())
        }
        "winding" => {
            let path = a.pos.get(2).ok_or("로그 파일을 주세요")?;
            let log = ingest::load_crane_log(&std::fs::read_to_string(path).map_err(|e| e.to_string())?, 1.0)?;
            let get = |k: &str| log.get(k).cloned().ok_or(format!("'{k}' 채널이 없습니다"));
            let tref = log.get("coolant_c").or(log.get("ambient_c")).cloned().ok_or("ambient_c 또는 coolant_c 채널이 필요합니다")?;
            let r = calib::fit_winding(&log.t, &get("current_a")?, &get("speed_rpm")?, &tref, &get("winding_c")?, 3000)?;
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            Ok(())
        }
        "kaggle" => {
            let path = a.pos.get(2).ok_or("measures_v2.csv 경로를 주세요")?;
            let f = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
            let profile = a.f("profile").map(|v| v as i64);
            let max_rows = a.f("max-rows").map(|v| v as usize);
            let profs = ingest::load_motor_temperature(std::io::BufReader::new(f), profile, max_rows)?;
            let use_ambient = a.s("ref") == Some("ambient");
            for (id, p) in profs.iter().take(if profile.is_some() { 1 } else { 3 }) {
                // 2 Hz → 10 s 로 줄여 맞춘다
                let k = 20usize;
                let pick = |v: &Vec<f64>| -> Vec<f64> { v.chunks(k).map(|c| c.iter().sum::<f64>() / c.len() as f64).collect() };
                let t: Vec<f64> = (0..p.t.len().div_ceil(k)).map(|i| i as f64 * 10.0).collect();
                let tref = if use_ambient { pick(&p.ambient_c) } else { pick(&p.coolant_c) };
                let r = calib::fit_winding(&t, &pick(&p.current_a), &pick(&p.speed), &tref, &pick(&p.winding_c), 3000)?;
                println!(
                    "profile {id}: {} 점, 권선 시간상수 {:.0} s, 프레임 {:.0} s, RMS 오차 {:.2} K",
                    r.samples, r.tau_w_s, r.tau_f_s, r.rms_error_k
                );
                println!("{}", serde_json::to_string(&r).unwrap());
            }
            Ok(())
        }
        _ => Err("calibrate 다음에 pendulum | coastdown | winding | kaggle | s3 중 하나를 주세요".into()),
    }
}

fn run() -> Result<(), String> {
    let a = parse_args();
    let cmd = a.pos.first().map(|s| s.as_str()).unwrap_or("demo");
    let out = PathBuf::from(a.s("out").unwrap_or("out"));
    match cmd {
        "help" | "-h" => {
            print!("{HELP}");
            Ok(())
        }
        "demo" => cmd_demo(),
        "presets" => {
            println!("{}", serde_json::to_string_pretty(&api::presets_json()).unwrap());
            Ok(())
        }
        "scenario" => {
            let crane = a.s("crane").unwrap_or("gantry_10t_20m");
            let load = a.f("load").unwrap_or(8000.0);
            let mode = mode_of(a.s("mode").unwrap_or("zvd"))?;
            println!("{}", serde_json::to_string_pretty(&cyclesim::demo_scenario(crane, load, mode)).unwrap());
            Ok(())
        }
        "simulate" => {
            let mut sc = read_scenario(a.pos.get(1).ok_or("시나리오 파일을 주세요")?)?;
            if let Some(m) = a.s("mode") {
                sc.antisway.mode = mode_of(m)?;
            }
            let r = cyclesim::simulate(&sc)?;
            print_cycle(&r);
            let spec = sc.crane.resolve()?;
            let rep = decision::evaluate(&DecisionInput { spec: &spec, cycle: Some(&r), duty: None, forecasts: &[], wind_mps: None, antisway_mode: sc.antisway.mode, drift: &[] });
            print_decision(&rep);
            save_cycle(&out, &r)
        }
        "compare" => {
            let sc = read_scenario(a.pos.get(1).ok_or("시나리오 파일을 주세요")?)?;
            let rows = api::compare(&sc, &AntiSwayMode::all())?;
            println!("{}", serde_json::to_string_pretty(&rows).unwrap());
            Ok(())
        }
        "duty" => {
            let sc = read_scenario(a.pos.get(1).ok_or("시나리오 파일을 주세요")?)?;
            let r = duty::run(&sc, &duty_cfg(&a))?;
            println!(
                "근무 {:.1} h · {} 사이클 · Kp {:.3} ({}) · 권선 최고 [주행 {:.1}, 횡행 {:.1}, 권상 {:.1}] °C · 에너지 {:.2} kWh · 로프 손상 {:.2e}",
                r.hours, r.cycles, r.kp, r.spectrum_class, r.max_winding_c[0], r.max_winding_c[1], r.max_winding_c[2], r.energy_kwh, r.rope_damage
            );
            let p = write(&out, "duty.json", &serde_json::to_string_pretty(&r).unwrap())?;
            println!("저장: {}", p.display());
            Ok(())
        }
        "pipeline" => {
            let sc = read_scenario(a.pos.get(1).ok_or("시나리오 파일을 주세요")?)?;
            let cfg = PipelineConfig {
                duty: Some(duty_cfg(&a)),
                forecast_hours: a.f("forecast-h").unwrap_or(8.0),
                wind_mps: a.f("wind"),
                sds_dir: a.s("sds").map(String::from),
                sling_length_m: a.f("sling").unwrap_or(0.0),
                ..Default::default()
            };
            let models = match a.s("models") {
                Some(dir) => Models::load(Path::new(dir)),
                None => Models::none(),
            };
            for n in models.notes.iter() {
                println!("모델: {n}");
            }
            let rep = api::run_pipeline(&sc, &cfg, &models)?;
            print_cycle(&rep.cycle);
            print_decision(&rep.decision);
            let p = write(&out, "pipeline.json", &serde_json::to_string(&rep).unwrap())?;
            write(&out, "samples.csv", &rep.cycle.samples.to_csv())?;
            if a.opt.contains_key("compare") {
                let rows = api::compare(&sc, &AntiSwayMode::all())?;
                write(&out, "compare.json", &serde_json::to_string_pretty(&rows).unwrap())?;
            }
            println!("저장: {} (뷰어 index.html 에서 열 수 있습니다)", p.display());
            #[cfg(feature = "store")]
            if let Some(db) = a.s("db") {
                let st = crane_model::store::Store::open(Path::new(db))?;
                let id = st.save_cycle(&rep.cycle)?;
                if let Some(d) = &rep.duty {
                    st.save_duty(d)?;
                }
                st.save_decision(id, &rep.decision)?;
                println!("DB 기록: run {id} → {db}");
            }
            Ok(())
        }
        "calibrate" => cmd_calibrate(&a),
        "crosscheck" => {
            let spec = crane_profile::preset(a.s("crane").unwrap_or("gantry_10t_20m")).ok_or("크레인 프리셋이 없습니다")?;
            let c = calib::crosscheck(&spec, a.f("rope").unwrap_or(8.0), a.f("load").unwrap_or(6000.0), a.f("sling").unwrap_or(0.0));
            println!("{}", serde_json::to_string_pretty(&c).unwrap());
            Ok(())
        }
        other => Err(format!("모르는 명령: {other}\n\n{HELP}")),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("오류: {e}");
        std::process::exit(1);
    }
}
