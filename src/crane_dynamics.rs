//! 갠트리 크레인 본체 동역학 — 주행(x) · 횡행(y) · 권상(로프 길이 l) · 편행(ψ) 과 3D 구면 진자 하중.
//!
//! 하중(훅 포함)은 질점, 로프는 질량 없는 비신장 줄로 본다. 하중의 트롤리 걸이점 기준 상대 위치 r(3D)를
//! 상태로 두고 |r| = l 구속을 장력 T(라그랑주 승수)로 푼다. 그래서
//! - 각도 매개화의 특이점(θ = 90°)이 없고,
//! - 권상(가변 l), 트롤리·갠트리 관성, 권상 회전체 관성의 결합이 정확하다.
//!
//! 장력 식 (w = 1/질량, 잠긴 축은 w = 0, u = r/|r|, F_e = 하중 외력):
//! ```text
//! T·(1/m + w_x·u_x² + w_y·u_y² + w_h) = (ṙ·ṙ − l̇²)/l + u·(g + F_e/m) − u_x·w_x·F_x − u_y·w_y·F_y + w_h·F_h
//! ẍ = w_x(F_x + T·u_x),  ÿ = w_y(F_y + T·u_y),  l̈ = w_h(T − F_h),  r̈ = g + F_e/m − (T/m)·u − (ẍ, ÿ, 0)
//! ```
//! F_h 는 권상 구동이 로프를 감아올리는 힘(하중쪽 환산)이다. T < 0 이면 로프가 느슨해진 것으로 보고 0 으로 자른다.
//! 매 스텝 끝에 |r| = l, u·ṙ = l̇ 로 투영해 수치 표류를 없앤다.
//!
//! 편행: 갠트리를 스팬 S 의 강체 보로 보고 양쪽 구동력·주행 저항(바퀴 하중 분배)의 차이가 만드는 회전 ψ 를 푼다.
//! 양 끝 어긋남 δ = ψ·S 가 틈 `skew_gap_m` 을 넘으면 플랜지 안내 강성이 되돌린다.
//!
//! 한계: 로프 탄성·훅–하중 이중 진자·로프 질량은 `crane_mbd` 정밀 해석기에서 다룬다.
//! 하중 감쇠력의 반작용은 트롤리에 걸지 않는다(작은 감쇠비 가정).

use crate::crane_profile::{CraneSpec, G};
use crate::environment::Environment;
use serde::{Deserialize, Serialize};

pub const NS: usize = 14;
pub const IX: usize = 0;
pub const IVX: usize = 1;
pub const IY: usize = 2;
pub const IVY: usize = 3;
pub const IL: usize = 4;
pub const IVL: usize = 5;
pub const IRX: usize = 6;
pub const IVRX: usize = 9;
pub const IPSI: usize = 12;
pub const IVPSI: usize = 13;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhysState {
    pub s: [f64; NS],
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

impl PhysState {
    /// (x, y) 에 트롤리가 있고 로프 길이 l 로 하중이 곧게 매달려 정지한 상태
    pub fn hanging(x: f64, y: f64, l: f64) -> Self {
        let mut s = [0.0; NS];
        s[IX] = x;
        s[IY] = y;
        s[IL] = l;
        s[IRX + 2] = -l;
        Self { s }
    }
    pub fn x(&self) -> f64 {
        self.s[IX]
    }
    pub fn vx(&self) -> f64 {
        self.s[IVX]
    }
    pub fn y(&self) -> f64 {
        self.s[IY]
    }
    pub fn vy(&self) -> f64 {
        self.s[IVY]
    }
    pub fn l(&self) -> f64 {
        self.s[IL]
    }
    pub fn vl(&self) -> f64 {
        self.s[IVL]
    }
    pub fn r(&self) -> [f64; 3] {
        [self.s[IRX], self.s[IRX + 1], self.s[IRX + 2]]
    }
    pub fn vr(&self) -> [f64; 3] {
        [self.s[IVRX], self.s[IVRX + 1], self.s[IVRX + 2]]
    }
    pub fn psi(&self) -> f64 {
        self.s[IPSI]
    }

    /// 하중의 절대 위치 (걸이점 높이 h 기준)
    pub fn load_pos(&self, h: f64) -> [f64; 3] {
        let r = self.r();
        [self.x() + r[0], self.y() + r[1], h + r[2]]
    }

    pub fn load_vel(&self) -> [f64; 3] {
        let v = self.vr();
        [self.vx() + v[0], self.vy() + v[1], v[2]]
    }

    /// (θx, θy, θ) rad — θx: x–z 평면 흔들림, θy: y–z 평면 흔들림, θ: 연직과 이루는 전체 각
    pub fn sway_angles(&self) -> (f64, f64, f64) {
        let r = self.r();
        let n = norm(r).max(1e-12);
        ((r[0]).atan2(-r[2]), (r[1]).atan2(-r[2]), (-r[2] / n).clamp(-1.0, 1.0).acos())
    }

    /// (θ̇x, θ̇y) rad/s
    pub fn sway_rates(&self) -> (f64, f64) {
        let r = self.r();
        let v = self.vr();
        let dx = r[0] * r[0] + r[2] * r[2];
        let dy = r[1] * r[1] + r[2] * r[2];
        // d/dt atan2(a, −z) = (ȧ·(−z) − a·(−ż)) / (a² + z²)
        let tx = if dx > 1e-12 { (v[0] * (-r[2]) + r[0] * v[2]) / dx } else { 0.0 };
        let ty = if dy > 1e-12 { (v[1] * (-r[2]) + r[1] * v[2]) / dy } else { 0.0 };
        (tx, ty)
    }

    /// 하중 수평 변위 (걸이점 기준, m)
    pub fn sway_offset_m(&self) -> f64 {
        let r = self.r();
        (r[0] * r[0] + r[1] * r[1]).sqrt()
    }

    /// 구속 투영: |r| = l, u·ṙ = l̇
    pub fn project(&mut self) {
        let l = self.l().max(1e-6);
        let r = self.r();
        let n = norm(r);
        if n < 1e-9 {
            self.s[IRX] = 0.0;
            self.s[IRX + 1] = 0.0;
            self.s[IRX + 2] = -l;
        } else {
            for k in 0..3 {
                self.s[IRX + k] = r[k] * l / n;
            }
        }
        let u = {
            let r = self.r();
            [r[0] / l, r[1] / l, r[2] / l]
        };
        let v = self.vr();
        let c = self.vl() - dot(v, u);
        for k in 0..3 {
            self.s[IVRX + k] = v[k] + c * u[k];
        }
    }
}

/// 한 스텝 동안 고정되는 입력
#[derive(Debug, Clone, Copy, Default)]
pub struct Inputs {
    /// 주행 구동력 (왼쪽 · 오른쪽 끝, N)
    pub f_travel_left: f64,
    pub f_travel_right: f64,
    /// 횡행 구동력 (N)
    pub f_trolley: f64,
    /// 권상 구동이 감아올리는 힘 (하중쪽 환산, N)
    pub f_hoist: f64,
    /// 체결된 브레이크가 미끄러지며 내는 마찰력 크기 (N, ≥ 0) — [주행, 횡행, 권상]
    pub brake_force: [f64; 3],
    /// 브레이크로 잠긴 축 — [주행, 횡행, 권상]
    pub locked: [bool; 3],
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Aux {
    pub tension_n: f64,
    pub slack: bool,
    /// 축 가속도 [ẍ, ÿ, l̈]
    pub acc: [f64; 3],
    pub wind: [f64; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct Masses {
    /// 하중 + 훅
    pub m: f64,
    /// 주행 축 질량 (갠트리 + 트롤리 + 회전체)
    pub mx: f64,
    /// 횡행 축 질량 (트롤리 + 회전체)
    pub my: f64,
    /// 권상 회전체 환산 질량
    pub mh: f64,
}

pub struct Plant<'a> {
    pub spec: &'a CraneSpec,
    pub env: &'a Environment,
    /// 하중 질량 (훅 제외, kg)
    pub load_kg: f64,
    /// 하중 항력 계수 × 면적 (m²)
    pub load_cd_area: f64,
}

fn smooth_sign(v: f64, eps: f64) -> f64 {
    (v / eps).tanh()
}

impl<'a> Plant<'a> {
    pub fn masses(&self) -> Masses {
        let s = &self.spec.structure;
        Masses {
            m: (self.load_kg + s.hook_mass_kg).max(1.0),
            mx: s.bridge_mass_kg + s.trolley_mass_kg + self.spec.travel.reflected_mass_kg(),
            my: s.trolley_mass_kg + self.spec.trolley.reflected_mass_kg(),
            mh: self.spec.hoist.reflected_mass_kg().max(1.0),
        }
    }

    /// 상태 미분과 장력
    pub fn deriv(&self, t: f64, s: &[f64; NS], u: &Inputs) -> ([f64; NS], Aux) {
        let st = &self.spec.structure;
        let ms = self.masses();
        let m = ms.m;
        let (vx, y, vy) = (s[IVX], s[IY], s[IVY]);
        let l = s[IL].max(1e-3);
        let vl = s[IVL];
        let r = [s[IRX], s[IRX + 1], s[IRX + 2]];
        let vr = [s[IVRX], s[IVRX + 1], s[IVRX + 2]];
        let rn = norm(r).max(1e-9);
        let un = [r[0] / rn, r[1] / rn, r[2] / rn];

        // 하중 외력: 바람 항력 + 흔들림 감쇠
        let wind = self.env.wind_at(t);
        let vload = [vx + vr[0], vy + vr[1]];
        let fd = self.env.drag(self.load_cd_area, [wind[0] - vload[0], wind[1] - vload[1]]);
        let vrad = dot(vr, un);
        let vt = [vr[0] - vrad * un[0], vr[1] - vrad * un[1], vr[2] - vrad * un[2]];
        let wp = (G / l).sqrt();
        let cs = 2.0 * st.sway_damping_ratio * m * wp;
        let fe = [fd[0] - cs * vt[0], fd[1] - cs * vt[1], -cs * vt[2]];

        let wx = if u.locked[0] { 0.0 } else { 1.0 / ms.mx };
        let wy = if u.locked[1] { 0.0 } else { 1.0 / ms.my };
        let wh = if u.locked[2] { 0.0 } else { 1.0 / ms.mh };

        // 주행: 구동 + 구름·점성 저항(양쪽 바퀴 하중 분배) + 갠트리 풍하중 + 미끄럼 제동
        let span = st.span_m.max(1e-3);
        let yr = (y / span).clamp(0.0, 1.0);
        let w_bridge = st.bridge_mass_kg * G;
        let w_tl = (st.trolley_mass_kg + m) * G;
        let n_left = 0.5 * w_bridge + w_tl * (1.0 - yr);
        let n_right = 0.5 * w_bridge + w_tl * yr;
        let sgx = smooth_sign(vx, 0.01);
        let fric_l = -st.rolling_resistance * n_left * sgx - 0.5 * st.viscous_travel * vx;
        let fric_r = -st.rolling_resistance * n_right * sgx - 0.5 * st.viscous_travel * vx;
        let wind_bridge = self.env.drag(st.bridge_cd_area_m2, [wind[0] - vx, 0.0])[0];
        let fbx = -u.brake_force[0] * smooth_sign(vx, 0.002);
        let fx = u.f_travel_left + u.f_travel_right + fric_l + fric_r + wind_bridge + fbx;

        // 횡행
        let n_tr = (st.trolley_mass_kg + m) * G;
        let fric_y = -st.rolling_resistance * n_tr * smooth_sign(vy, 0.01) - st.viscous_trolley * vy;
        let fby = -u.brake_force[1] * smooth_sign(vy, 0.002);
        let fy = u.f_trolley + fric_y + fby;

        // 권상: 미끄럼 제동은 l̇ 반대 방향 → 감아올리는 힘과 같은 꼴
        let fh = u.f_hoist + u.brake_force[2] * smooth_sign(vl, 0.002);

        // 장력
        let a_ext = [fe[0] / m, fe[1] / m, fe[2] / m - G];
        let denom = 1.0 / m + wx * un[0] * un[0] + wy * un[1] * un[1] + wh;
        let num = (dot(vr, vr) - vl * vl) / l + dot(un, a_ext) - un[0] * wx * fx - un[1] * wy * fy + wh * fh;
        let mut tension = num / denom;
        let slack = tension < 0.0;
        if slack {
            tension = 0.0;
        }
        let ax = wx * (fx + tension * un[0]);
        let ay = wy * (fy + tension * un[1]);
        let al = wh * (tension - fh);
        let ar = [
            a_ext[0] - tension * un[0] / m - ax,
            a_ext[1] - tension * un[1] / m - ay,
            a_ext[2] - tension * un[2] / m,
        ];

        // 편행 회전
        let iz = st.bridge_mass_kg * span * span / 12.0 + (st.trolley_mass_kg + m) * (y - 0.5 * span).powi(2) + 1.0;
        let delta = s[IPSI] * span;
        let guide = if delta.abs() > st.skew_gap_m {
            -st.skew_stiffness_n_per_m * (delta.abs() - st.skew_gap_m) * delta.signum()
        } else {
            0.0
        };
        let moment = (u.f_travel_left + fric_l - u.f_travel_right - fric_r) * 0.5 * span + guide * 0.5 * span - st.skew_damping * s[IVPSI];
        let apsi = if u.locked[0] { 0.0 } else { moment / iz };

        let mut d = [0.0; NS];
        d[IX] = vx;
        d[IVX] = ax;
        d[IY] = vy;
        d[IVY] = ay;
        d[IL] = vl;
        d[IVL] = al;
        for k in 0..3 {
            d[IRX + k] = vr[k];
            d[IVRX + k] = ar[k];
        }
        d[IPSI] = if u.locked[0] { 0.0 } else { s[IVPSI] };
        d[IVPSI] = apsi;
        (d, Aux { tension_n: tension, slack, acc: [ax, ay, al], wind })
    }

    /// 고전 RK4 한 스텝 + 구속 투영. 스텝 끝 상태의 보조값을 돌려준다.
    pub fn step(&self, t: f64, st: &mut PhysState, u: &Inputs, dt: f64) -> Aux {
        let s0 = st.s;
        let (k1, _) = self.deriv(t, &s0, u);
        let s1 = add(&s0, &k1, 0.5 * dt);
        let (k2, _) = self.deriv(t + 0.5 * dt, &s1, u);
        let s2 = add(&s0, &k2, 0.5 * dt);
        let (k3, _) = self.deriv(t + 0.5 * dt, &s2, u);
        let s3 = add(&s0, &k3, dt);
        let (k4, _) = self.deriv(t + dt, &s3, u);
        for i in 0..NS {
            st.s[i] = s0[i] + dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
        for (k, idx) in [(0usize, IVX), (1, IVY), (2, IVL)] {
            if u.locked[k] {
                st.s[idx] = 0.0;
            }
        }
        if u.locked[0] {
            st.s[IVPSI] = 0.0;
        }
        st.project();
        self.deriv(t + dt, &st.s, u).1
    }

    /// 운동 에너지 + 위치 에너지 (J) — 시험용
    pub fn energy(&self, st: &PhysState) -> f64 {
        let ms = self.masses();
        let v = st.load_vel();
        0.5 * ms.mx * st.vx().powi(2)
            + 0.5 * ms.my * st.vy().powi(2)
            + 0.5 * ms.mh * st.vl().powi(2)
            + 0.5 * ms.m * dot(v, v)
            + ms.m * G * st.r()[2]
    }
}

fn add(a: &[f64; NS], b: &[f64; NS], h: f64) -> [f64; NS] {
    let mut o = [0.0; NS];
    for i in 0..NS {
        o[i] = a[i] + h * b[i];
    }
    o
}

/// 고정 트롤리에서 단진자 고유진동수 (Hz)
pub fn pendulum_freq_hz(l: f64) -> f64 {
    (G / l.max(1e-6)).sqrt() / (2.0 * std::f64::consts::PI)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crane_profile::gantry_10t_20m;

    fn no_damping_spec() -> CraneSpec {
        let mut c = gantry_10t_20m();
        c.structure.sway_damping_ratio = 0.0;
        c.structure.rolling_resistance = 0.0;
        c.structure.viscous_travel = 0.0;
        c.structure.viscous_trolley = 0.0;
        c.structure.bridge_cd_area_m2 = 0.0;
        c
    }

    /// 영점 교차로 주기를 잰다
    fn period(ts: &[f64], xs: &[f64]) -> f64 {
        let mut cross = Vec::new();
        for i in 1..xs.len() {
            if xs[i - 1] < 0.0 && xs[i] >= 0.0 {
                let w = -xs[i - 1] / (xs[i] - xs[i - 1]);
                cross.push(ts[i - 1] + w * (ts[i] - ts[i - 1]));
            }
        }
        (cross[cross.len() - 1] - cross[0]) / (cross.len() - 1) as f64
    }

    #[test]
    fn locked_trolley_gives_simple_pendulum_period() {
        let spec = no_damping_spec();
        let env = Environment::default();
        let plant = Plant { spec: &spec, env: &env, load_kg: 5000.0, load_cd_area: 0.0 };
        let l = 8.0;
        let mut st = PhysState::hanging(0.0, 10.0, l);
        let a0: f64 = 0.02;
        st.s[IRX] = l * a0.sin();
        st.s[IRX + 2] = -l * a0.cos();
        let u = Inputs { locked: [true, true, true], ..Default::default() };
        let (mut ts, mut xs) = (vec![], vec![]);
        let dt = 0.002;
        let mut t = 0.0;
        let e0 = plant.energy(&st);
        for _ in 0..(40.0 / dt) as usize {
            plant.step(t, &mut st, &u, dt);
            t += dt;
            ts.push(t);
            xs.push(st.r()[0]);
        }
        let p = period(&ts, &xs);
        let expect = 2.0 * std::f64::consts::PI * (l / G).sqrt();
        assert!((p - expect).abs() / expect < 2e-4, "{p} vs {expect}");
        let e1 = plant.energy(&st);
        assert!((e1 - e0).abs() < 1e-6 * (5200.0 * G * l), "energy drift {e0} → {e1}");
        assert!((st.l() - l).abs() < 1e-12);
    }

    #[test]
    fn free_trolley_couples_pendulum_frequency() {
        // 구동력이 없는 자유 트롤리: ω² = g(M + m)/(M·l)
        let spec = no_damping_spec();
        let env = Environment::default();
        let plant = Plant { spec: &spec, env: &env, load_kg: 8000.0, load_cd_area: 0.0 };
        let ms = plant.masses();
        let l = 6.0;
        let mut st = PhysState::hanging(0.0, 10.0, l);
        let a0: f64 = 0.01;
        st.s[IRX + 1] = l * a0.sin();
        st.s[IRX + 2] = -l * a0.cos();
        let u = Inputs { locked: [true, false, true], ..Default::default() };
        let (mut ts, mut ys) = (vec![], vec![]);
        let dt = 0.002;
        let mut t = 0.0;
        for _ in 0..(40.0 / dt) as usize {
            plant.step(t, &mut st, &u, dt);
            t += dt;
            ts.push(t);
            ys.push(st.r()[1]);
        }
        let p = period(&ts, &ys);
        let w = (G * (ms.my + ms.m) / (ms.my * l)).sqrt();
        let expect = 2.0 * std::f64::consts::PI / w;
        assert!((p - expect).abs() / expect < 1e-3, "{p} vs {expect}");
    }

    #[test]
    fn hanging_at_rest_carries_weight() {
        let spec = no_damping_spec();
        let env = Environment::default();
        let plant = Plant { spec: &spec, env: &env, load_kg: 3000.0, load_cd_area: 0.0 };
        let st = PhysState::hanging(0.0, 5.0, 5.0);
        let u = Inputs { locked: [true, true, true], ..Default::default() };
        let (_, aux) = plant.deriv(0.0, &st.s, &u);
        assert!((aux.tension_n - 3200.0 * G).abs() < 1e-6);
    }

    #[test]
    fn unbraked_hoist_without_torque_accelerates_like_atwood() {
        let spec = no_damping_spec();
        let env = Environment::default();
        let plant = Plant { spec: &spec, env: &env, load_kg: 3000.0, load_cd_area: 0.0 };
        let ms = plant.masses();
        let st = PhysState::hanging(0.0, 5.0, 5.0);
        let u = Inputs { locked: [true, true, false], ..Default::default() };
        let (d, aux) = plant.deriv(0.0, &st.s, &u);
        let expect_t = ms.m * ms.mh * G / (ms.m + ms.mh);
        assert!((aux.tension_n - expect_t).abs() / expect_t < 1e-9);
        assert!((d[IVL] - ms.m * G / (ms.m + ms.mh)).abs() < 1e-9);
    }

    #[test]
    fn energy_conserved_with_hoisting_free_axes() {
        let spec = no_damping_spec();
        let env = Environment::default();
        let plant = Plant { spec: &spec, env: &env, load_kg: 2000.0, load_cd_area: 0.0 };
        let l = 7.0;
        let mut st = PhysState::hanging(0.0, 10.0, l);
        st.s[IRX] = l * 0.05f64.sin();
        st.s[IRX + 2] = -l * 0.05f64.cos();
        st.s[IVX] = 0.3;
        st.s[IVY] = -0.2;
        st.project();
        // 감아올리는 힘 = 하중 무게 (내리는 방향 가속 없음) → 에너지는 일만큼 변한다. 힘 0 이면 보존.
        let u = Inputs { locked: [false, false, false], ..Default::default() };
        let e0 = plant.energy(&st);
        let dt = 0.001;
        let mut t = 0.0;
        for _ in 0..2000 {
            plant.step(t, &mut st, &u, dt);
            t += dt;
        }
        let e1 = plant.energy(&st);
        assert!((e1 - e0).abs() / e0.abs().max(1.0) < 1e-6, "{e0} → {e1}");
    }
}
