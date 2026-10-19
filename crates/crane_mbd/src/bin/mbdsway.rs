//! 명령줄 실행: 로프 길이 · 하중 · 슬링 길이를 받아 흔들림 주기와 상하 진동을 계산한다.
//! 사용: mbdsway <로프 m> <하중 kg> [슬링 m] [EA N] [훅 kg]

use crane_mbd::*;

fn main() {
    let a: Vec<f64> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let rope = a.first().cloned().unwrap_or(8.0);
    let load = a.get(1).cloned().unwrap_or(5000.0);
    let sling = a.get(2).cloned().unwrap_or(0.0);
    let ea = a.get(3).cloned().unwrap_or(2.9e7);
    let hook = a.get(4).cloned().unwrap_or(100.0);
    let p = MbdParams {
        rope_length_m: rope,
        segments: 8,
        rope_mass_per_m: 2.3,
        ea_n: ea,
        axial_damping_ratio: 0.057,
        hook_mass_kg: hook,
        sling_length_m: sling,
        sling_ea_n: 5.0e7,
        load_mass_kg: load,
        load_cd_area_m2: 0.0,
        air_density: 1.2,
    };
    let path = Path::default();
    let r = run(&p, &path, &Start::Tilted { angle_rad: 0.02 }, 60.0, 0.005);
    let per = mean_period(&r.t, &sway_x(&r, &path, true));
    println!("dt = {:.2e} s, steps = {}", r.dt, r.steps);
    println!("static stretch = {:.2} mm", r.static_stretch_m * 1000.0);
    match per {
        Some(p) => println!("sway period = {:.3} s ({:.4} Hz)", p, 1.0 / p),
        None => println!("sway period: not enough cycles"),
    }
    if sling > 0.0 {
        let (w1, w2) = double_pendulum_modes(hook, load, rope, sling);
        println!("double pendulum linear modes: {:.4} Hz, {:.4} Hz", w1 / (2.0 * std::f64::consts::PI), w2 / (2.0 * std::f64::consts::PI));
    }
}
