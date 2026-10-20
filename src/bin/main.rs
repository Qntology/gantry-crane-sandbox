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
use crane_model::backend::Backend;
use crane_model::climate_data;
use crane_model::fieldcal;
use crane_model::health::{self, HealthStore};
use crane_model::ingest;
use crane_model::motor;
use crane_model::operation::{self, DecisionPolicy, OpSim, OperationConfig, OperationReport, StepOutcome};
use crane_model::site::{self, SiteStore};
use crane_model::weather;
use std::collections::HashMap;
use std::io::{BufRead, Read, Write as IoWrite};
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

현장 프리셋 · 운영 · 내구도:
  crane_demo site list [--presets DIR]                 내장 · 저장한 현장 프리셋
  crane_demo site show <id>                           프리셋 JSON
  crane_demo site save <새 id> --from <id> [--name 이름] [--set 경로=값 ...] [--file preset.json]
                                                      값을 바꿔 사용자 프리셋으로 저장 (예: --set throughput.moves_per_hour=25)
  crane_demo site delete <id>
  crane_demo climate list | show <id> | check <id> [--years N]   기후 프리셋 · 생성 날씨 vs 평년값
  crane_demo operate --site <id> [--asset 이름] [--days N] [--start MM-DD] [--seed N]
                     [--policy ask|recommended|conservative|risky] [--ledger DIR] [--fresh] [--no-incidents] [--out DIR]
                                                      현장 운영 시뮬레이션 (ask: 결정 카드마다 묻는다)
  crane_demo health list | show <asset> | replace <asset> <부품> | delete <asset> [--ledger DIR]
  crane_demo fieldcal [--out DIR]                     실데이터 대조 · 오차 표
  crane_demo serve [--port 8765] [--data DIR]          브라우저 화면 + API (http://localhost:8765)

MODE: off | zv | zvd | ei | closed_loop
";

struct Args {
    pos: Vec<String>,
    opt: HashMap<String, String>,
    /// 같은 옵션을 여러 번 줄 수 있는 것 (--set)
    multi: Vec<(String, String)>,
}

fn parse_args() -> Args {
    let mut pos = Vec::new();
    let mut opt = HashMap::new();
    let mut multi = Vec::new();
    let mut it = std::env::args().skip(1).peekable();
    while let Some(a) = it.next() {
        if let Some(k) = a.strip_prefix("--") {
            let v = match it.peek() {
                Some(n) if !n.starts_with("--") => it.next().unwrap(),
                _ => "true".into(),
            };
            multi.push((k.to_string(), v.clone()));
            opt.insert(k.to_string(), v);
        } else {
            pos.push(a);
        }
    }
    Args { pos, opt, multi }
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
        "site" => cmd_site(&a),
        "climate" => cmd_climate(&a),
        "operate" => cmd_operate(&a, &out),
        "health" => cmd_health(&a),
        "fieldcal" => cmd_fieldcal(&a, &out),
        "serve" => cmd_serve(&a),
        "crosscheck" => {
            let spec = crane_profile::preset(a.s("crane").unwrap_or("gantry_10t_20m")).ok_or("크레인 프리셋이 없습니다")?;
            let c = calib::crosscheck(&spec, a.f("rope").unwrap_or(8.0), a.f("load").unwrap_or(6000.0), a.f("sling").unwrap_or(0.0));
            println!("{}", serde_json::to_string_pretty(&c).unwrap());
            Ok(())
        }
        other => Err(format!("모르는 명령: {other}\n\n{HELP}")),
    }
}

fn presets_dir(a: &Args) -> PathBuf {
    PathBuf::from(a.s("presets").unwrap_or("presets"))
}

fn ledger_dir(a: &Args) -> PathBuf {
    PathBuf::from(a.s("ledger").unwrap_or("out/health"))
}

fn cmd_site(a: &Args) -> Result<(), String> {
    let store = SiteStore::new(&presets_dir(a));
    match a.pos.get(1).map(|s| s.as_str()).unwrap_or("list") {
        "list" => {
            println!("{:<28} {:<14} {:<6} 이름", "id", "용도", "종류");
            for s in store.list() {
                println!("{:<28} {:<14} {:<6} {}", s.id, s.application, if s.builtin { "내장" } else { "사용자" }, s.name);
            }
            println!("\n사용자 프리셋 폴더: {}", store.dir().display());
            Ok(())
        }
        "show" => {
            let id = a.pos.get(2).ok_or("프리셋 id 를 주세요")?;
            println!("{}", serde_json::to_string_pretty(&store.get(id)?).unwrap());
            Ok(())
        }
        "save" => {
            let id = a.pos.get(2).ok_or("저장할 새 id 를 주세요")?;
            let mut p = match a.s("file") {
                Some(f) => serde_json::from_str(&std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}"))?).map_err(|e| format!("{f}: {e}"))?,
                None => store.get(a.s("from").ok_or("--from <기존 id> 또는 --file 이 필요합니다")?)?,
            };
            let sets: Vec<(String, String)> = a
                .multi
                .iter()
                .filter(|(k, _)| k == "set")
                .map(|(_, v)| {
                    let (k, val) = v.split_once('=').ok_or_else(|| format!("--set '{v}' 는 경로=값 꼴이어야 합니다"))?;
                    Ok((k.trim().to_string(), val.trim().to_string()))
                })
                .collect::<Result<_, String>>()?;
            p = site::with_overrides(&p, &sets)?;
            p.id = id.clone();
            if let Some(n) = a.s("name") {
                p.name = n.to_string();
            }
            let path = store.save(&p)?;
            println!("저장: {} ({} 항목 변경)", path.display(), sets.len());
            Ok(())
        }
        "delete" => {
            let id = a.pos.get(2).ok_or("지울 프리셋 id 를 주세요")?;
            store.delete(id)?;
            println!("삭제: {id}");
            Ok(())
        }
        other => Err(format!("site 다음에 list | show | save | delete 중 하나를 주세요 ({other})")),
    }
}

fn cmd_climate(a: &Args) -> Result<(), String> {
    match a.pos.get(1).map(|s| s.as_str()).unwrap_or("list") {
        "list" => {
            for c in climate_data::presets() {
                println!("{:<14} {:<28} 연평균 {:.1} °C · 출처: {}", c.id, c.name, c.annual_mean_temp(), c.source);
            }
            Ok(())
        }
        "show" => {
            let id = a.pos.get(2).ok_or("기후 id 를 주세요")?;
            let c = climate_data::preset(id).ok_or(format!("기후 '{id}' 가 없습니다"))?;
            println!("{}", serde_json::to_string_pretty(&c).unwrap());
            Ok(())
        }
        "check" => {
            let id = a.pos.get(2).ok_or("기후 id 를 주세요")?;
            let c = climate_data::preset(id).ok_or(format!("기후 '{id}' 가 없습니다"))?;
            let years = a.f("years").unwrap_or(5.0).max(1.0) as u32;
            let w = weather::generate(&c, &weather::WeatherOptions { start_doy: 0, days: 365 * years, height_m: 10.0, seed: a.f("seed").unwrap_or(1.0) as u64 });
            let st = weather::monthly_stats(&w, 10.0, c.alpha);
            println!("{} — 생성 {} 년 vs 평년값 (괄호)", c.name, years);
            println!("{:>3} {:>14} {:>14} {:>14} {:>12} {:>12} {:>12}", "월", "기온 °C", "풍속 m/s", "강수 mm", "강수일", "뇌전일", "안개일");
            for (m, s) in st.iter().enumerate() {
                let n = &c.months[m];
                println!(
                    "{:>3} {:>6.1} ({:>5.1}) {:>6.2} ({:>5.2}) {:>6.0} ({:>5.0}) {:>5.1} ({:>4.1}) {:>5.1} ({:>4.1}) {:>5.1} ({:>4.1})",
                    m + 1,
                    s.temp_mean_c,
                    n.temp_mean_c,
                    s.wind_mean_10m_mps,
                    n.wind_mean_mps,
                    s.precip_mm,
                    n.precip_mm,
                    s.rain_days,
                    n.rain_days,
                    s.thunder_days,
                    n.thunder_days,
                    s.fog_days,
                    n.fog_days
                );
            }
            Ok(())
        }
        other => Err(format!("climate 다음에 list | show | check 중 하나를 주세요 ({other})")),
    }
}

fn print_card(d: &operation::OpDecision) {
    let p = &d.prompt;
    println!("\n┌─ [{}] {} ───────────────────────────────", p.header, d.at);
    println!("│ {}", p.question);
    for a in p.attention.iter() {
        println!("│  ! {a}");
    }
    let mut n = 0;
    for o in p.options.iter() {
        if o.enabled {
            n += 1;
            println!("│  {}) {}{} — {}", n, o.label, if o.recommended { " (권장)" } else { "" }, o.detail);
        } else {
            println!("│  ✕ {} — 막힘: {}", o.label, o.blocked_by.join(", "));
        }
    }
    print!("└─ 번호 [Enter = 권장, a = 남은 결정 모두 권장]: ");
    let _ = std::io::stdout().flush();
}

/// 결정 카드를 보여 주고 고른 선택지 key 를 돌려준다. None 이면 남은 결정을 모두 권장으로
fn ask_card(d: &operation::OpDecision) -> Option<String> {
    print_card(d);
    let mut line = String::new();
    let n = std::io::stdin().lock().read_line(&mut line).unwrap_or(0);
    let t = line.trim();
    if n == 0 || t.eq_ignore_ascii_case("a") {
        println!("(남은 결정은 권장으로 고릅니다)");
        return None;
    }
    if t.is_empty() {
        return Some(d.prompt.recommended.clone());
    }
    let enabled: Vec<&crane_model::choice::ChoiceOption> = d.prompt.options.iter().filter(|o| o.enabled).collect();
    if let Ok(k) = t.parse::<usize>() {
        if k >= 1 && k <= enabled.len() {
            return Some(enabled[k - 1].key.clone());
        }
    }
    if let Some(o) = d.prompt.options.iter().find(|o| o.key == t) {
        return Some(o.key.clone());
    }
    println!("알 수 없는 선택 '{t}' — 권장으로 고릅니다");
    Some(d.prompt.recommended.clone())
}

fn print_operation(r: &OperationReport, ledger_path: Option<&Path>) {
    let k = &r.kpi;
    println!("\n■ 현장 운영: {} · 자산 {} · 크레인 {} · {} 일 ({} 시작) · 정책 {}", r.site_name, r.asset_id, r.crane, k.days, r.start_date, r.policy);
    println!(
        "  계획 {:.0} h · 운전 {:.0} h · 가동률 {:.1}% · 이동 {:.0} / 수요 {:.0} (남은 {:.0}) · 운전 시간당 {:.1} 회 · 에너지 {:.0} kWh (회생 {:.0})",
        k.scheduled_h,
        k.operating_h,
        k.availability * 100.0,
        k.moves,
        k.demand_moves,
        k.backlog_moves,
        k.moves_per_op_hour,
        k.energy_kwh,
        k.regen_kwh
    );
    if !k.downtime_by_reason.is_empty() {
        let d: Vec<String> = k.downtime_by_reason.iter().map(|(r, h)| format!("{r} {h:.0} h")).collect();
        println!("  정지 · 손실: {}", d.join(" · "));
    }
    let al: Vec<String> = k.alerts_by_level.iter().map(|(l, n)| format!("{l} {n}")).collect();
    println!("  알림 {} 건 ({}) · 결정 {} 건 (작업자 {}) · 결정으로 줄인 기대 사고 {:.3}", r.alerts.len(), al.join(", "), k.decisions, k.decisions_by_operator, k.risk_avoided);
    println!("  사고 {} 건 (기대 {:.3}) · 위험 지수 평균 {:.0} · 최대 {:.0} · 정격 초과 차단 {:.0} 회", k.incidents, k.expected_incidents, k.risk_mean, k.risk_max, k.refused_overweight);
    for i in r.incidents.iter().take(5) {
        println!("    {} [{}] {} — {}", i.at, i.severity, i.kind, i.detail);
    }
    let important: Vec<&crane_model::alerts::Alert> = r.alerts.iter().filter(|a| a.level >= crane_model::alerts::AlertLevel::Warning).collect();
    if !important.is_empty() {
        println!("  주요 알림 (경고 이상, 최근 8건):");
        for a in important.iter().rev().take(8).rev() {
            println!("    {} [{}] {} — {}", a.at, a.level_label, a.title, a.message);
        }
    }
    let d = &r.durability;
    println!(
        "■ 부품 내구도 · 안정성 (누적 {:.0} 일 · 운전 {:.0} h · 이동 {:.0}){}",
        d.days,
        d.operating_hours,
        d.moves,
        ledger_path.map(|p| format!(" — 원장 {}", p.display())).unwrap_or_default()
    );
    println!("  {:<24} {:<8} {:>9} {:>9} {:>5} {:>8} {:>8}", "부품", "상태", "수명 사용", "남은 일", "교체", "여유", "최소 여유");
    for row in d.rows.iter() {
        println!(
            "  {:<24} {:<8} {:>8.1}% {:>9} {:>5} {:>8} {:>8}",
            row.label,
            row.status_label,
            row.life_used * 100.0,
            row.rul_days.map(|v| if v > 99_999.0 { "> 10만".into() } else { format!("{v:.0}") }).unwrap_or_else(|| "—".into()),
            row.replacements,
            row.margin.map(|v| format!("{v:.2}")).unwrap_or_else(|| "—".into()),
            row.margin_min.map(|v| format!("{v:.2}")).unwrap_or_else(|| "—".into())
        );
    }
    for n in r.notes.iter() {
        println!("  메모: {n}");
    }
}

fn cmd_operate(a: &Args, out: &Path) -> Result<(), String> {
    let id = a.s("site").or(a.pos.get(1).map(|s| s.as_str())).ok_or("--site <id> 가 필요합니다 (crane_demo site list)")?;
    let preset = SiteStore::new(&presets_dir(a)).get(id)?;
    let policy = match a.s("policy") {
        Some(p) => DecisionPolicy::from_key(p).ok_or(format!("모르는 정책: {p}"))?,
        None => DecisionPolicy::Recommended,
    };
    let (start_month, start_day) = match a.s("start") {
        Some(s) => {
            let (m, d) = s.split_once('-').ok_or("--start 는 MM-DD 꼴입니다")?;
            (Some(m.parse().map_err(|_| "--start 월")?), Some(d.parse().map_err(|_| "--start 일")?))
        }
        None => (None, None),
    };
    let cfg = OperationConfig {
        asset_id: a.s("asset").map(String::from).unwrap_or_else(|| format!("{}-01", preset.id)),
        days: a.f("days").unwrap_or(30.0) as u32,
        start_month,
        start_day,
        seed: a.f("seed").unwrap_or(1.0) as u64,
        policy,
        incidents: !a.opt.contains_key("no-incidents"),
        fresh: a.opt.contains_key("fresh"),
    };
    let store = HealthStore::new(&ledger_dir(a));
    let ledger = if cfg.fresh { None } else { store.load(&cfg.asset_id)? };
    if let Some(l) = &ledger {
        println!("원장 이어 쓰기: {} (누적 {:.0} 일, 실행 {} 회)", l.asset_id, l.day, l.runs);
    }
    println!("사이클 증분표 계산 중 (하중 구간 {} 개 × 정상 · 감속)…", preset.loads.spectrum.len());
    let mut sim = OpSim::new(preset, cfg, ledger)?;
    let mut interactive = policy == DecisionPolicy::Ask;
    loop {
        match sim.run(usize::MAX / 4) {
            StepOutcome::Finished => break,
            StepOutcome::Decision { decision } => {
                let key = if interactive {
                    match ask_card(&decision) {
                        Some(k) => k,
                        None => {
                            interactive = false;
                            decision.prompt.recommended.clone()
                        }
                    }
                } else {
                    decision.prompt.recommended.clone()
                };
                if let Err(e) = sim.decide(&decision.id, &key) {
                    println!("  {e} — 권장으로 고릅니다");
                    sim.decide(&decision.id, &decision.prompt.recommended)?;
                }
                if let Some(d) = sim.report().decisions.last() {
                    println!("  → {} : {}", d.prompt.options.iter().find(|o| o.chosen).map(|o| o.label.clone()).unwrap_or_default(), d.outcome.clone().unwrap_or_default());
                }
            }
            StepOutcome::Paused { .. } => {}
        }
    }
    let r = sim.report();
    let lp = store.save(sim.ledger())?;
    print_operation(&r, Some(&lp));
    let p = write(out, "operation.json", &serde_json::to_string(&r).map_err(|e| e.to_string())?)?;
    println!("저장: {} (화면에서 열 수 있습니다)", p.display());
    Ok(())
}

fn cmd_health(a: &Args) -> Result<(), String> {
    let store = HealthStore::new(&ledger_dir(a));
    match a.pos.get(1).map(|s| s.as_str()).unwrap_or("list") {
        "list" => {
            for id in store.list() {
                if let Ok(Some(l)) = store.load(&id) {
                    let worst = l.critical(1).first().map(|c| format!("{} {:.0}%", c.label, c.life_used * 100.0)).unwrap_or_default();
                    println!("{:<24} {:<18} {:<26} {:>6.0} 일 {:>8.0} 이동 · 가장 닳은 부품: {}", l.asset_id, l.crane, l.site, l.day, l.moves, worst);
                }
            }
            Ok(())
        }
        "show" => {
            let id = a.pos.get(2).ok_or("자산 이름을 주세요")?;
            let l = store.load(id)?.ok_or(format!("자산 '{id}' 원장이 없습니다"))?;
            println!("{}", serde_json::to_string_pretty(&health::evaluate(&l)).unwrap());
            Ok(())
        }
        "replace" => {
            let id = a.pos.get(2).ok_or("자산 이름을 주세요")?;
            let comp = a.pos.get(3).ok_or("부품 id 를 주세요 (예: rope_hoist)")?;
            let mut l = store.load(id)?.ok_or(format!("자산 '{id}' 원장이 없습니다"))?;
            let spec = crane_profile::preset(&l.crane)
                .or_else(|| SiteStore::new(&presets_dir(a)).get(&l.site).ok().and_then(|s| s.spec().ok()))
                .ok_or("원장의 크레인 제원을 찾지 못했습니다")?;
            let day = l.day;
            l.replace(comp, &spec, day, 0, "수동 교체 (crane_demo health replace)")?;
            store.save(&l)?;
            println!("{id}: {comp} 교체 기록");
            Ok(())
        }
        "delete" => {
            let id = a.pos.get(2).ok_or("자산 이름을 주세요")?;
            store.delete(id)?;
            println!("삭제: {id}");
            Ok(())
        }
        other => Err(format!("health 다음에 list | show | replace | delete 중 하나를 주세요 ({other})")),
    }
}

fn cmd_fieldcal(a: &Args, out: &Path) -> Result<(), String> {
    let o = if a.opt.contains_key("quick") { fieldcal::CalOptions::quick() } else { fieldcal::CalOptions::default() };
    println!("실데이터 대조 · 보정 중 (기후 {} 년 · 운영 {} 일)…", o.climate_years, o.sim_days);
    let r = fieldcal::run(&o);
    let mut group = String::new();
    for b in r.benchmarks.iter() {
        if b.group != group {
            group = b.group.clone();
            println!("\n■ {}", match group.as_str() {
                "weather" => "날씨 (기상청 관측 vs 생성 날씨)",
                "motor" => "모터 (제조사 표 vs 열 모델)",
                "throughput" => "처리량",
                "durability" => "내구도",
                _ => "법규 확인",
            });
            println!("  {:<40} {:>10} {:>10} {:>10} {:>9} {:>9}  계수 · 출처", "항목", "실데이터", "보정 전", "보정 뒤", "전 오차", "뒤 오차");
        }
        println!(
            "  {:<40} {:>10.3} {:>10.3} {:>10.3} {:>8.1}% {:>8.1}%  {}{}",
            b.label,
            b.reference,
            b.before,
            b.after,
            b.error_before_pct,
            b.error_after_pct,
            b.knob.as_ref().map(|k| format!("[{k}] ")).unwrap_or_default(),
            b.source
        );
    }
    println!(
        "\n맞춘 항목 평균 절대 오차: 보정 전 {:.1}% → 보정 뒤 {:.1}% (모든 항목 보정 뒤 {:.1}%)",
        r.mean_abs_error_before_pct, r.mean_abs_error_after_pct, r.mean_abs_error_all_after_pct
    );
    println!("맞춘 계수: {}", serde_json::to_string(&r.fitted).unwrap());
    for n in r.notes.iter() {
        println!("메모: {n}");
    }
    let p = write(out, "fieldcal.json", &serde_json::to_string_pretty(&r).unwrap())?;
    println!("저장: {}", p.display());
    Ok(())
}

const INDEX_HTML: &str = include_str!("../../index.html");

fn http_reply(stream: &mut std::net::TcpStream, status: &str, ctype: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: content-type\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn serve_one(stream: &mut std::net::TcpStream, backend: &std::sync::Mutex<Backend>) -> Result<(), String> {
    // 브라우저가 미리 열어 두고 보내지 않는 연결이 다른 요청을 막지 않게
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(10)));
    let mut reader = std::io::BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut first = String::new();
    reader.read_line(&mut first).map_err(|e| e.to_string())?;
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("/").to_string();
    let mut len = 0usize;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h).map_err(|e| e.to_string())? == 0 || h.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                len = v.trim().parse().unwrap_or(0);
            }
        }
    }
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).map_err(|e| e.to_string())?;
    match (method.as_str(), path.as_str()) {
        ("OPTIONS", _) => http_reply(stream, "204 No Content", "text/plain", b""),
        ("GET", "/") | ("GET", "/index.html") => http_reply(stream, "200 OK", "text/html; charset=utf-8", INDEX_HTML.as_bytes()),
        ("POST", p) if p.starts_with("/api/") => {
            let cmd = &p[5..];
            let args: serde_json::Value = if body.is_empty() { serde_json::json!({}) } else { serde_json::from_slice(&body).map_err(|e| e.to_string())? };
            let result = match backend.lock() {
                Ok(mut b) => b.call(cmd, &args),
                Err(e) => Err(e.to_string()),
            };
            let reply = match result {
                Ok(v) => serde_json::json!({"ok": true, "data": v}),
                Err(e) => serde_json::json!({"ok": false, "error": e}),
            };
            http_reply(stream, "200 OK", "application/json; charset=utf-8", reply.to_string().as_bytes());
        }
        _ => http_reply(stream, "404 Not Found", "text/plain", b"not found"),
    }
    Ok(())
}

fn cmd_serve(a: &Args) -> Result<(), String> {
    let port = a.f("port").unwrap_or(8765.0) as u16;
    let data = PathBuf::from(a.s("data").unwrap_or("out/app"));
    let mut backend = Backend::new(&data);
    if let Some(m) = a.s("models") {
        backend.models = Models::load(Path::new(m));
    }
    let backend = std::sync::Arc::new(std::sync::Mutex::new(backend));
    let listener = std::net::TcpListener::bind(("127.0.0.1", port)).map_err(|e| format!("포트 {port}: {e}"))?;
    println!("화면: http://localhost:{port}  (데이터 폴더 {}) — 끝내려면 Ctrl+C", data.display());
    for s in listener.incoming() {
        match s {
            Ok(mut stream) => {
                let b = backend.clone();
                std::thread::spawn(move || {
                    if let Err(e) = serve_one(&mut stream, &b) {
                        if !e.contains("10060") && !e.contains("timed out") && !e.contains("시간") {
                            eprintln!("요청 오류: {e}");
                        }
                    }
                });
            }
            Err(e) => eprintln!("연결 오류: {e}"),
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("오류: {e}");
        std::process::exit(1);
    }
}
