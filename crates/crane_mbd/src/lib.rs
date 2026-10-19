//! 집중 질량 로프 · 훅 · 하중 다물체 해석기 (교차검증용).
//!
//! 앱의 해석 모델(비신장 무질량 로프 + 질점 하중의 구면 진자)을 **다른 가정으로 푼 두 번째 계산**과 맞대어 보기 위한
//! 연구용 모델이다. 엔드밀 프로젝트의 `endmill_mpm` 자리.
//!
//! 모델
//! - 로프: 걸이점에서 훅까지 `segments` 개의 질점 + 인장 전용 스프링(EA/L0).
//!   축 방향 감쇠는 걸이점–훅 사이(그리고 훅–하중 사이) 감쇠기 하나로 모아 무거운 질점에만 건다.
//!   질점마다 감쇠기를 달면 가벼운 로프 질점에서 명시적 적분이 불안정해지기 때문이다.
//!   줄 수 n 인 권상은 등가 단일 로프로 본다 (EA × n, 단위 길이 질량 × n).
//! - 훅: 마지막 로프 질점에 훅 질량을 더한다.
//! - 하중: 슬링 길이 > 0 이면 훅 아래 별도 질점(슬링 스프링으로 연결) → 훅–하중 이중 진자.
//!   슬링 길이 = 0 이면 하중이 훅에 붙는다 (단진자 + 로프 탄성).
//! - 하중 공기 항력, 중력. 걸이점은 주어진 궤적(시각별 x, y)을 따른다.
//! - 적분: 반암시적(심플렉틱) 오일러. 시간 간격은 가장 빠른 축 모드 주기로 정한다.
//!
//! 한계: 로프 굽힘 강성 · 비틀림 · 시브 마찰 · 드럼 감김 없음. 실험으로 검증된 모델이 아니다.

pub const G: f64 = 9.81;

#[derive(Debug, Clone)]
pub struct MbdParams {
    /// 걸이점 → 훅 로프 길이 (무부하 길이, m)
    pub rope_length_m: f64,
    pub segments: usize,
    /// 등가 로프 단위 길이 질량 (kg/m, 줄 수 반영)
    pub rope_mass_per_m: f64,
    /// 등가 축 강성 EA (N, 줄 수 반영)
    pub ea_n: f64,
    /// 축 방향 감쇠비 (상하 진동 모드 기준)
    pub axial_damping_ratio: f64,
    pub hook_mass_kg: f64,
    /// 훅 → 하중 무게중심 (m). 0 이면 하중이 훅에 붙는다
    pub sling_length_m: f64,
    pub sling_ea_n: f64,
    pub load_mass_kg: f64,
    pub load_cd_area_m2: f64,
    pub air_density: f64,
}

impl MbdParams {
    pub fn total_hanging_mass(&self) -> f64 {
        self.hook_mass_kg + self.load_mass_kg
    }
}

/// 걸이점 궤적 (구간 선형). 비어 있으면 원점에 고정
#[derive(Debug, Clone, Default)]
pub struct Path {
    pub t: Vec<f64>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

impl Path {
    pub fn at(&self, t: f64) -> [f64; 2] {
        if self.t.is_empty() {
            return [0.0, 0.0];
        }
        if t <= self.t[0] {
            return [self.x[0], self.y[0]];
        }
        let n = self.t.len();
        if t >= self.t[n - 1] {
            return [self.x[n - 1], self.y[n - 1]];
        }
        let i = match self.t.binary_search_by(|v| v.partial_cmp(&t).unwrap_or(std::cmp::Ordering::Less)) {
            Ok(i) => return [self.x[i], self.y[i]],
            Err(i) => i - 1,
        };
        let w = (t - self.t[i]) / (self.t[i + 1] - self.t[i]).max(1e-12);
        [self.x[i] + w * (self.x[i + 1] - self.x[i]), self.y[i] + w * (self.y[i + 1] - self.y[i])]
    }
}

#[derive(Debug, Clone)]
pub enum Start {
    /// 정적 평형에서 하중(또는 훅)을 x 방향으로 `angle_rad` 만큼 기울여 놓는다 (전체를 강체처럼 회전)
    Tilted { angle_rad: f64 },
    /// 로프는 연직에서 `rope_rad`, 슬링(훅 → 하중)은 `sling_rad` 만큼 기울인 직선으로 놓는다.
    /// 분절 길이를 보존하므로 축 진동을 거의 일으키지 않는다 — 이중 진자 모드 형상 등
    Angles { rope_rad: f64, sling_rad: f64 },
    /// 정적 평형에서 하중 쪽으로 갈수록 크게 `dz_m` 만큼 내린다 — 상하 진동만 일으킬 때
    Drop { dz_m: f64 },
    /// 정적 평형에서 정지
    Rest,
}

#[derive(Debug, Clone, Default)]
pub struct MbdResult {
    pub t: Vec<f64>,
    /// 하중(또는 훅) 위치 [x, y, z] (걸이점 높이 = 0)
    pub load: Vec<[f64; 3]>,
    pub hook: Vec<[f64; 3]>,
    pub top_tension_n: Vec<f64>,
    pub dt: f64,
    pub steps: usize,
    pub static_stretch_m: f64,
}

struct Model {
    m: Vec<f64>,
    rest: Vec<f64>,
    k: Vec<f64>,
    /// 걸이점–훅 감쇠 계수, 훅–하중 감쇠 계수 (N·s/m)
    c_rope: f64,
    c_sling: f64,
    cd_load: f64,
    load_idx: usize,
    hook_idx: usize,
    rho: f64,
}

fn build(p: &MbdParams) -> Model {
    let n = p.segments.max(1);
    let l0 = p.rope_length_m / n as f64;
    let m_seg = p.rope_mass_per_m * l0;
    let mut m = vec![m_seg; n];
    m[n - 1] = 0.5 * m_seg + p.hook_mass_kg;
    let mut rest = vec![l0; n];
    let mut k = vec![p.ea_n / l0; n];
    let has_sling = p.sling_length_m > 1e-6;
    if has_sling {
        m.push(p.load_mass_kg.max(1e-6));
        rest.push(p.sling_length_m);
        k.push(p.sling_ea_n / p.sling_length_m);
    } else {
        m[n - 1] += p.load_mass_kg;
    }
    // 감쇠: 상하 진동 모드 기준 임계 감쇠의 ζ 배 (로프 전체 강성 EA/L, 매달린 전체 질량)
    let hanging: f64 = m.iter().sum();
    let k_rope = p.ea_n / p.rope_length_m.max(1e-9);
    let c_rope = 2.0 * p.axial_damping_ratio * (k_rope * hanging).sqrt();
    let c_sling = if has_sling { 2.0 * p.axial_damping_ratio * (k[n] * p.load_mass_kg.max(1e-6)).sqrt() } else { 0.0 };
    let load_idx = m.len() - 1;
    Model { m, rest, k, c_rope, c_sling, cd_load: p.load_cd_area_m2, load_idx, hook_idx: n - 1, rho: p.air_density }
}

/// 안정한 시간 간격 (가장 빠른 국소 축 진동 주기의 일부)
pub fn stable_dt(p: &MbdParams) -> f64 {
    let md = build(p);
    let mut wmax: f64 = 0.0;
    for i in 0..md.k.len() {
        let mi = md.m[i].min(if i > 0 { md.m[i - 1] } else { f64::MAX });
        wmax = wmax.max((2.0 * md.k[i] / mi.max(1e-9)).sqrt());
    }
    let mut dt = (0.3 * 2.0 / wmax.max(1e-9)).min(1e-3);
    // 감쇠기가 걸린 질점의 명시적 안정 조건 dt < m/c
    if md.c_rope > 0.0 {
        dt = dt.min(0.5 * md.m[md.hook_idx] / md.c_rope);
    }
    if md.c_sling > 0.0 {
        dt = dt.min(0.5 * md.m[md.hook_idx].min(md.m[md.load_idx]) / md.c_sling);
    }
    dt
}

fn static_positions(md: &Model) -> (Vec<f64>, f64) {
    // 아래 질량의 무게만큼 각 스프링이 늘어난다
    let mut z = Vec::with_capacity(md.m.len());
    let mut acc = 0.0;
    let mut stretch = 0.0;
    for i in 0..md.m.len() {
        let below: f64 = md.m[i..].iter().sum::<f64>() * G;
        let ext = below / md.k[i];
        stretch += ext;
        acc -= md.rest[i] + ext;
        z.push(acc);
    }
    (z, stretch)
}

pub fn run(p: &MbdParams, path: &Path, start: &Start, duration_s: f64, record_dt: f64) -> MbdResult {
    let md = build(p);
    let n = md.m.len();
    let dt = stable_dt(p);
    let (z0, stretch) = static_positions(&md);
    let s0 = path.at(0.0);
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| [s0[0], s0[1], z0[i]]).collect();
    match start {
        Start::Tilted { angle_rad } => {
            let (sa, ca) = (angle_rad.sin(), angle_rad.cos());
            for p in pos.iter_mut() {
                let (x, z) = (p[0] - s0[0], p[2]);
                p[0] = s0[0] + x * ca - z * sa;
                p[2] = x * sa + z * ca;
            }
        }
        Start::Angles { rope_rad, sling_rad } => {
            let n_rope = p.segments.max(1);
            let mut hook = [s0[0], 0.0];
            for (i, q) in pos.iter_mut().enumerate() {
                if i < n_rope {
                    let dist = -z0[i];
                    q[0] = s0[0] + dist * rope_rad.sin();
                    q[2] = -dist * rope_rad.cos();
                    if i == n_rope - 1 {
                        hook = [q[0], q[2]];
                    }
                } else {
                    let dist = z0[n_rope - 1] - z0[i];
                    q[0] = hook[0] + dist * sling_rad.sin();
                    q[2] = hook[1] - dist * sling_rad.cos();
                }
            }
        }
        Start::Drop { dz_m } => {
            let nn = pos.len() as f64;
            for (i, q) in pos.iter_mut().enumerate() {
                q[2] -= dz_m * (i + 1) as f64 / nn;
            }
        }
        Start::Rest => {}
    }
    let mut vel = vec![[0.0f64; 3]; n];
    let steps = (duration_s / dt).ceil() as usize;
    let rec_every = ((record_dt / dt).round() as usize).max(1);
    let mut out = MbdResult { dt, steps, static_stretch_m: stretch, ..Default::default() };
    let mut force = vec![[0.0f64; 3]; n];
    for s in 0..=steps {
        let t = s as f64 * dt;
        let sp = path.at(t);
        let sp_prev = path.at((t - dt).max(0.0));
        let sv = [(sp[0] - sp_prev[0]) / dt, (sp[1] - sp_prev[1]) / dt, 0.0];
        for f in force.iter_mut() {
            *f = [0.0; 3];
        }
        let mut top_t = 0.0;
        for i in 0..n {
            let pa = if i == 0 { [sp[0], sp[1], 0.0] } else { pos[i - 1] };
            let d = [pos[i][0] - pa[0], pos[i][1] - pa[1], pos[i][2] - pa[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-12);
            let u = [d[0] / len, d[1] / len, d[2] / len];
            let ext = len - md.rest[i];
            let tension = if ext > 0.0 { md.k[i] * ext } else { 0.0 }; // 인장 전용
            if i == 0 {
                top_t = tension;
            }
            for k in 0..3 {
                force[i][k] -= tension * u[k];
                if i > 0 {
                    force[i - 1][k] += tension * u[k];
                }
            }
        }
        // 축 방향 감쇠기: 걸이점–훅, 훅–하중
        {
            let h = md.hook_idx;
            let d = [pos[h][0] - sp[0], pos[h][1] - sp[1], pos[h][2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-12);
            let rv = ((vel[h][0] - sv[0]) * d[0] + (vel[h][1] - sv[1]) * d[1] + vel[h][2] * d[2]) / len;
            for k in 0..3 {
                force[h][k] -= md.c_rope * rv * d[k] / len;
            }
            if md.c_sling > 0.0 {
                let l = md.load_idx;
                let d = [pos[l][0] - pos[h][0], pos[l][1] - pos[h][1], pos[l][2] - pos[h][2]];
                let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-12);
                let rv = ((vel[l][0] - vel[h][0]) * d[0] + (vel[l][1] - vel[h][1]) * d[1] + (vel[l][2] - vel[h][2]) * d[2]) / len;
                for k in 0..3 {
                    let f = md.c_sling * rv * d[k] / len;
                    force[l][k] -= f;
                    force[h][k] += f;
                }
            }
        }
        // 항력 (하중) · 중력
        let vl = vel[md.load_idx];
        let sp_l = (vl[0] * vl[0] + vl[1] * vl[1] + vl[2] * vl[2]).sqrt();
        for k in 0..3 {
            force[md.load_idx][k] -= 0.5 * md.rho * md.cd_load * sp_l * vl[k];
        }
        if s % rec_every == 0 {
            out.t.push(t);
            let rel = |q: [f64; 3]| [q[0], q[1], q[2]];
            out.load.push(rel(pos[md.load_idx]));
            out.hook.push(rel(pos[md.hook_idx]));
            out.top_tension_n.push(top_t);
        }
        for i in 0..n {
            for k in 0..3 {
                let a = force[i][k] / md.m[i] - if k == 2 { G } else { 0.0 };
                vel[i][k] += a * dt;
                pos[i][k] += vel[i][k] * dt;
            }
        }
    }
    out
}

/// 걸이점 기준 수평 변위 시계열 (x)
pub fn sway_x(r: &MbdResult, path: &Path, of_load: bool) -> Vec<f64> {
    let src = if of_load { &r.load } else { &r.hook };
    r.t.iter().zip(src.iter()).map(|(t, p)| p[0] - path.at(*t)[0]).collect()
}

/// 영점 교차로 잰 평균 주기 (s)
pub fn mean_period(t: &[f64], y: &[f64]) -> Option<f64> {
    let mean = y.iter().sum::<f64>() / y.len().max(1) as f64;
    let mut cross = Vec::new();
    for i in 1..y.len() {
        let (a, b) = (y[i - 1] - mean, y[i] - mean);
        if a < 0.0 && b >= 0.0 {
            let w = -a / (b - a);
            cross.push(t[i - 1] + w * (t[i] - t[i - 1]));
        }
    }
    if cross.len() < 3 {
        return None;
    }
    Some((cross[cross.len() - 1] - cross[0]) / (cross.len() - 1) as f64)
}

/// 이중 진자(훅 m1, 하중 m2, 길이 l1, l2) 선형 모드 각진동수 (rad/s), 낮은 것부터
pub fn double_pendulum_modes(m1: f64, m2: f64, l1: f64, l2: f64) -> (f64, f64) {
    let r = m2 / m1;
    let a = (1.0 + r) * (1.0 / l1 + 1.0 / l2);
    let disc = (a * a - 4.0 * (1.0 + r) / (l1 * l2)).max(0.0).sqrt();
    ((0.5 * G * (a - disc)).sqrt(), (0.5 * G * (a + disc)).sqrt())
}

/// 모드 형상: 각도 비 θ2/θ1
pub fn double_pendulum_shape(m1: f64, m2: f64, l1: f64, l2: f64, w: f64) -> f64 {
    (m1 + m2) * (G - l1 * w * w) / (m2 * l2 * w * w)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> MbdParams {
        MbdParams {
            rope_length_m: 8.0,
            segments: 8,
            rope_mass_per_m: 0.2,
            ea_n: 2.0e7,
            axial_damping_ratio: 0.0,
            hook_mass_kg: 100.0,
            sling_length_m: 0.0,
            sling_ea_n: 1.0e7,
            load_mass_kg: 4900.0,
            load_cd_area_m2: 0.0,
            air_density: 1.2,
        }
    }

    #[test]
    fn static_stretch_matches_ea() {
        let p = base();
        let r = run(&p, &Path::default(), &Start::Rest, 0.5, 0.01);
        let expect = p.total_hanging_mass() * G * p.rope_length_m / p.ea_n;
        assert!((r.static_stretch_m - expect).abs() / expect < 0.02, "{} vs {}", r.static_stretch_m, expect);
        // 정지 상태가 유지된다
        let z0 = r.load[0][2];
        let z1 = r.load.last().unwrap()[2];
        assert!((z1 - z0).abs() < 1e-4, "{z0} {z1}");
    }

    #[test]
    fn small_swing_matches_simple_pendulum() {
        let p = base();
        let path = Path::default();
        let r = run(&p, &path, &Start::Tilted { angle_rad: 0.02 }, 30.0, 0.005);
        let per = mean_period(&r.t, &sway_x(&r, &path, true)).unwrap();
        let l_eff = p.rope_length_m + r.static_stretch_m;
        let expect = 2.0 * std::f64::consts::PI * (l_eff / G).sqrt();
        assert!((per - expect).abs() / expect < 0.01, "{per} vs {expect}");
    }

    #[test]
    fn vertical_bounce_frequency() {
        let p = base();
        let path = Path::default();
        let r = run(&p, &path, &Start::Drop { dz_m: 0.001 }, 4.0, 0.0005);
        let zs: Vec<f64> = r.load.iter().map(|q| q[2]).collect();
        let f = 1.0 / mean_period(&r.t, &zs).unwrap();
        let expect = (p.ea_n / (p.rope_length_m * p.total_hanging_mass())).sqrt() / (2.0 * std::f64::consts::PI);
        assert!((f - expect).abs() / expect < 0.03, "{f} vs {expect}");
    }

    #[test]
    fn double_pendulum_modes_match_linear_theory() {
        let mut p = base();
        p.rope_length_m = 4.0;
        p.segments = 4;
        p.rope_mass_per_m = 0.5;
        p.ea_n = 2.0e8;
        p.axial_damping_ratio = 0.2;
        p.hook_mass_kg = 200.0;
        p.load_mass_kg = 1000.0;
        p.sling_length_m = 2.0;
        p.sling_ea_n = 2.0e8;
        let (w1, w2) = double_pendulum_modes(p.hook_mass_kg, p.load_mass_kg, p.rope_length_m, p.sling_length_m);
        let path = Path::default();
        for (w, amp) in [(w1, 0.02), (w2, 0.01)] {
            let ratio = double_pendulum_shape(p.hook_mass_kg, p.load_mass_kg, p.rope_length_m, p.sling_length_m, w);
            let r = run(&p, &path, &Start::Angles { rope_rad: amp, sling_rad: amp * ratio }, 40.0, 0.002);
            let per = mean_period(&r.t, &sway_x(&r, &path, true)).unwrap();
            let expect = 2.0 * std::f64::consts::PI / w;
            assert!((per - expect).abs() / expect < 0.02, "mode {w}: {per} vs {expect}");
        }
    }

    #[test]
    fn path_interpolates() {
        let p = Path { t: vec![0.0, 1.0, 2.0], x: vec![0.0, 1.0, 1.0], y: vec![0.0, 0.0, 2.0] };
        assert_eq!(p.at(0.5), [0.5, 0.0]);
        assert_eq!(p.at(1.5), [1.0, 1.0]);
        assert_eq!(p.at(9.0), [1.0, 2.0]);
    }
}
