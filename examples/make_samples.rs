//! data/sample/ 의 합성 로그를 만든다 (보정 명령 시험용). 실측 데이터가 아니다.
//! 실행: cargo run --release --example make_samples
use crane_model::antisway::AntiSwayMode;
use crane_model::cyclesim::{self, MoveMode, Waypoint};
use std::fmt::Write as _;

fn main() {
    let dir = std::path::Path::new("data/sample");
    std::fs::create_dir_all(dir).unwrap();

    // 1) 흔들림 로그: 셰이핑 없이 이동한 뒤 브레이크로 멈춘 트롤리 아래의 자유 흔들림
    let mut sc = cyclesim::demo_scenario("gantry_10t_20m", 6000.0, AntiSwayMode::Off);
    sc.start.load_kg = 6000.0;
    sc.start.l = 7.0;
    sc.waypoints = vec![Waypoint { label: "move".into(), x: Some(14.0), y: None, l: None, mode: MoveMode::Sequential, dwell_s: 0.0, set_load_kg: None }];
    sc.sim.settle_s = 40.0;
    let r = cyclesim::simulate(&sc).unwrap();
    let end = r.moves[0].end_s + 2.0;
    let mut s = String::from("time_s,bridge_position_m,sway_x_deg,rope_length_m\n");
    for i in 0..r.samples.len() {
        if r.samples.t[i] >= end {
            writeln!(s, "{:.3},{:.5},{:.5},{:.4}", r.samples.t[i], r.samples.x[i], r.samples.theta_x_deg[i], r.samples.l[i]).unwrap();
        }
    }
    std::fs::write(dir.join("pendulum_log.csv"), s).unwrap();

    // 2) 굴러가기(coast-down) 로그: 구동력 없이 감속하는 주행 속도
    let (mu, c, m) = (0.006f64, 200.0f64, 31_200.0f64);
    let mut v = 0.65f64;
    let mut s = String::from("time_s,travel_speed\n");
    let mut t = 0.0;
    while v > 0.0 {
        writeln!(s, "{:.2},{:.5}", t, v).unwrap();
        v -= (mu * 9.81 + c / m * v) * 0.05;
        t += 0.05;
    }
    std::fs::write(dir.join("coastdown_log.csv"), s).unwrap();

    // 3) 권선 온도 로그 (일반 CSV): 10 분 주기 S3 40% 운전
    let mot = crane_model::motor::preset("im_11kw_4p").unwrap();
    let mut th = crane_model::motor::MotorThermal::at_ambient(25.0);
    let w = mot.rated_speed_rad_s();
    let mut s = String::from("time_s,current_a,speed_rpm,ambient_c,winding_c\n");
    for k in 0..(4 * 3600 / 5) {
        let t = k as f64 * 5.0;
        let on = (t % 600.0) < 240.0;
        let (tau, om) = if on { (1.1 * mot.rated_torque_nm(), w) } else { (0.0, 0.0) };
        let l = if on { mot.losses_w(tau, om) } else { crane_model::motor::MotorLosses::default() };
        th.step(&mot, &l, om, 25.0, 5.0);
        let i = if on { mot.current_a(tau, om) } else { 0.0 };
        writeln!(s, "{:.0},{:.3},{:.1},25.0,{:.3}", t, i, om * 60.0 / (2.0 * std::f64::consts::PI), th.winding_c).unwrap();
    }
    std::fs::write(dir.join("winding_log.csv"), s).unwrap();

    // 4) Kaggle Electric Motor Temperature 형식 (열 이름 · 2 Hz 만 같다)
    let mut s = String::from("u_q,coolant,stator_winding,u_d,stator_tooth,motor_speed,i_d,i_q,pm,stator_yoke,ambient,torque,profile_id\n");
    let mut th = crane_model::motor::MotorThermal::at_ambient(30.0);
    for k in 0..(2 * 3600 * 2) {
        let t = k as f64 * 0.5;
        let phase = ((t / 900.0) as i64) % 3;
        let (iq, n) = match phase {
            0 => (30.0, 3000.0),
            1 => (12.0, 1500.0),
            _ => (0.0, 0.0),
        };
        let cur = (4.0f64 * 4.0 + iq * iq).sqrt();
        // 21.5 A 정격 11 kW 모터의 열 회로에 전류를 비례시켜 넣는다 (형식 시험용 합성값)
        let tau = mot.rated_torque_nm() * (iq / 21.5);
        let om = n / 60.0 * 2.0 * std::f64::consts::PI / 2.0;
        let l = mot.losses_w(tau, om);
        th.step(&mot, &l, om, 30.0, 0.5);
        writeln!(s, "0,30,{:.3},0,{:.3},{:.1},-4,{:.3},40,{:.3},25,{:.2},{}", th.winding_c, th.frame_c, n, iq, th.frame_c, tau, 1 + (t / 3600.0) as i64).unwrap();
        let _ = cur;
    }
    std::fs::write(dir.join("kaggle_format_sample.csv"), s).unwrap();
    println!("data/sample 에 합성 로그 4개를 썼습니다");
}
