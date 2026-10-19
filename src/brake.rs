//! 스프링 작동 · 전자 해제식(fail-safe) 크레인 브레이크.
//!
//! - 시퀀스: 해제 명령 → `release_delay_s` 뒤 해제, 체결 명령 → `set_delay_s` 뒤 체결.
//!   정상 정지에서는 인버터가 먼저 속도를 0 으로 만든 뒤 체결하므로 마찰일이 거의 없고,
//!   비상 정지(STO)처럼 움직이는 중에 체결되면 제동 토크 × 미끄럼 속도만큼 마찰일이 생긴다.
//! - 마모: 라이닝 두께 감소 = 마찰일 / (마모 0.1 mm 당 마찰일) × 0.1 mm.
//!   mayr ROBA-stop-S (100–800 Nm): 0.1 mm 당 44–95 MJ, 회전자 마모 한계까지 308–1,900 MJ.
//! - 1회 허용 마찰일: INTORQ BFK458 식 Q_perm = Q_E·(1 − e^(−S_hü/S_h)) (S_h = 시간당 제동 횟수).
//! - 온도: 집중 열용량 하나 + 주변으로의 열저항, 표면 허용 온도 130 °C (INTORQ BFK458).
//!
//! 크기별 값은 위 카탈로그 범위의 양 끝을 토크로 선형 보간한 **예시**다. 실제 브레이크 크기의 값이나
//! 라이닝 두께 실측으로 교체한다 (docs/data-sources.md).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrakeParams {
    /// 모터 축 기준 제동(유지) 토크, 모터 1대당 (Nm)
    pub holding_torque_nm: f64,
    pub set_delay_s: f64,
    pub release_delay_s: f64,
    /// 라이닝(마찰판) 두께: 신품 / 교체 한계 (mm)
    pub lining_new_mm: f64,
    pub lining_min_mm: f64,
    /// 마모 0.1 mm 당 마찰일 (MJ)
    pub work_per_0p1mm_mj: f64,
    /// 1회 허용 마찰일 상한 Q_E (J) 과 전환 빈도 S_hü (회/h)
    pub q_e_j: f64,
    pub s_transition_per_h: f64,
    /// 열용량 (J/K), 주변으로의 열저항 (K/W), 허용 표면 온도 (°C)
    pub heat_capacity_j_per_k: f64,
    pub cooling_r_k_per_w: f64,
    pub max_temp_c: f64,
}

impl BrakeParams {
    /// 모터 정격 토크의 `factor` 배 유지 토크를 갖는 예시 브레이크
    pub fn example(rated_motor_torque_nm: f64, factor: f64) -> Self {
        let t = rated_motor_torque_nm * factor;
        // mayr ROBA-stop-S 100–800 Nm 범위 양 끝을 토크로 선형 보간
        let s = ((t - 100.0) / 700.0).clamp(0.0, 1.0);
        let w01 = 44.0 + (95.0 - 44.0) * s;
        let total = 308.0 + (1900.0 - 308.0) * s;
        let usable_mm = total / w01 * 0.1;
        // INTORQ BFK458: Q_E 3–120 kJ, S_hü 79 → 15 회/h (크기가 클수록 Q_E 는 크고 S_hü 는 작다)
        let r = ((t.max(4.0) / 4.0).ln() / (600.0f64 / 4.0).ln()).clamp(0.0, 1.0);
        Self {
            holding_torque_nm: t,
            // 개폐 시간은 수백 ms 규모: Innomotics CR 81 KFB 63 120 / 342 ms, Hubbell 드럼 브레이크 해제 0.2 s · 체결 0.35–0.45 s
            set_delay_s: 0.3,
            release_delay_s: 0.2,
            lining_new_mm: 1.0 + usable_mm,
            lining_min_mm: 1.0,
            work_per_0p1mm_mj: w01,
            q_e_j: 3_000.0 + (120_000.0 - 3_000.0) * r,
            s_transition_per_h: 79.0 + (15.0 - 79.0) * r,
            heat_capacity_j_per_k: (t * 40.0).max(800.0),
            cooling_r_k_per_w: (2.0 / (t / 50.0 + 1.0)).max(0.05),
            max_temp_c: 130.0,
        }
    }

    /// 시간당 제동 `stops_per_hour` 회일 때 1회 허용 마찰일 (J)
    pub fn permitted_work_per_stop_j(&self, stops_per_hour: f64) -> f64 {
        let sh = stops_per_hour.max(1e-6);
        self.q_e_j * (1.0 - (-self.s_transition_per_h / sh).exp())
    }

    /// 교체 한계까지 남은 마찰일 (MJ) — 라이닝 `lining_mm` 에서
    pub fn remaining_work_mj(&self, lining_mm: f64) -> f64 {
        ((lining_mm - self.lining_min_mm).max(0.0) / 0.1) * self.work_per_0p1mm_mj
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrakePhase {
    Released,
    Setting,
    Set,
    Releasing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrakeState {
    pub phase: BrakePhase,
    pub timer_s: f64,
    pub lining_mm: f64,
    pub temp_c: f64,
    pub work_total_j: f64,
    pub work_this_stop_j: f64,
    pub max_stop_work_j: f64,
    pub max_temp_c: f64,
    /// 체결 횟수
    pub operations: u64,
    /// 움직이는 중에 체결되어 마찰일이 생긴 횟수
    pub dynamic_stops: u64,
}

impl BrakeState {
    pub fn new(p: &BrakeParams, ambient_c: f64) -> Self {
        Self {
            phase: BrakePhase::Set,
            timer_s: 0.0,
            lining_mm: p.lining_new_mm,
            temp_c: ambient_c,
            work_total_j: 0.0,
            work_this_stop_j: 0.0,
            max_stop_work_j: 0.0,
            max_temp_c: ambient_c,
            operations: 0,
            dynamic_stops: 0,
        }
    }

    pub fn command_release(&mut self) {
        match self.phase {
            BrakePhase::Set | BrakePhase::Setting => {
                self.phase = BrakePhase::Releasing;
                self.timer_s = 0.0;
            }
            _ => {}
        }
    }

    pub fn command_set(&mut self) {
        match self.phase {
            BrakePhase::Released | BrakePhase::Releasing => {
                self.phase = BrakePhase::Setting;
                self.timer_s = 0.0;
                self.work_this_stop_j = 0.0;
            }
            _ => {}
        }
    }

    /// 시퀀스 타이머를 진행한다
    pub fn update(&mut self, p: &BrakeParams, dt: f64) {
        match self.phase {
            BrakePhase::Releasing => {
                self.timer_s += dt;
                if self.timer_s >= p.release_delay_s {
                    self.phase = BrakePhase::Released;
                }
            }
            BrakePhase::Setting => {
                self.timer_s += dt;
                if self.timer_s >= p.set_delay_s {
                    self.phase = BrakePhase::Set;
                    self.operations += 1;
                }
            }
            _ => {}
        }
    }

    /// 체결 완료 상태인가
    pub fn engaged(&self) -> bool {
        self.phase == BrakePhase::Set
    }

    /// 패드가 디스크를 잡고 있는가 — 체결 완료, 또는 해제 명령 뒤 아직 열리지 않은 동안
    pub fn holding(&self) -> bool {
        matches!(self.phase, BrakePhase::Set | BrakePhase::Releasing)
    }

    /// 모터가 브레이크에 맞서 토크를 낼 수 있는가 (해제 완료)
    pub fn released(&self) -> bool {
        self.phase == BrakePhase::Released
    }

    /// 미끄럼 마찰일 `work_j` 를 더한다 (마모·온도)
    pub fn add_friction_work(&mut self, p: &BrakeParams, work_j: f64) {
        if !(work_j.is_finite() && work_j > 0.0) {
            return;
        }
        if self.work_this_stop_j == 0.0 {
            self.dynamic_stops += 1;
        }
        self.work_total_j += work_j;
        self.work_this_stop_j += work_j;
        self.max_stop_work_j = self.max_stop_work_j.max(self.work_this_stop_j);
        let dmm = work_j / (p.work_per_0p1mm_mj.max(1e-6) * 1e6) * 0.1;
        self.lining_mm = (self.lining_mm - dmm).max(0.0);
        self.temp_c += work_j / p.heat_capacity_j_per_k.max(1e-6);
        self.max_temp_c = self.max_temp_c.max(self.temp_c);
    }

    /// 주변으로 식는다
    pub fn cool(&mut self, p: &BrakeParams, ambient_c: f64, dt: f64) {
        let tau = (p.heat_capacity_j_per_k * p.cooling_r_k_per_w).max(1e-6);
        let k = (-dt / tau).exp();
        self.temp_c = ambient_c + (self.temp_c - ambient_c) * k;
    }

    /// 남은 라이닝 사용 여유 (1 = 신품, 0 = 교체 한계)
    pub fn remaining_fraction(&self, p: &BrakeParams) -> f64 {
        let usable = (p.lining_new_mm - p.lining_min_mm).max(1e-9);
        ((self.lining_mm - p.lining_min_mm) / usable).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_delays() {
        let p = BrakeParams::example(72.0, 2.0);
        let mut s = BrakeState::new(&p, 20.0);
        assert!(s.engaged());
        s.command_release();
        s.update(&p, 0.1);
        assert!(!s.released());
        assert!(s.holding());
        s.update(&p, 0.1);
        assert!(s.released());
        s.command_set();
        s.update(&p, 0.2);
        assert!(!s.engaged());
        s.update(&p, 0.1);
        assert!(s.engaged());
        assert_eq!(s.operations, 1);
    }

    #[test]
    fn example_follows_catalogue_ranges() {
        // 100 Nm 이하: 0.1 mm 당 44 MJ, 마모 한계까지 308 MJ / 800 Nm 이상: 95 MJ, 1,900 MJ (mayr ROBA-stop-S)
        let small = BrakeParams::example(50.0, 1.0);
        assert!((small.work_per_0p1mm_mj - 44.0).abs() < 1e-9);
        assert!((small.remaining_work_mj(small.lining_new_mm) - 308.0).abs() < 1e-6);
        let big = BrakeParams::example(400.0, 2.0);
        assert!((big.remaining_work_mj(big.lining_new_mm) - 1900.0).abs() < 1e-6);
        // INTORQ: 제동이 잦을수록 1회 허용 마찰일이 줄어든다
        assert!(small.permitted_work_per_stop_j(10.0) > small.permitted_work_per_stop_j(200.0));
        assert!(small.permitted_work_per_stop_j(0.01) <= small.q_e_j);
    }

    #[test]
    fn wear_and_heat_follow_work() {
        let p = BrakeParams::example(72.0, 2.0);
        let mut s = BrakeState::new(&p, 20.0);
        s.add_friction_work(&p, 1.0e6); // 1 MJ
        let expect = 0.1 / p.work_per_0p1mm_mj;
        assert!((p.lining_new_mm - s.lining_mm - expect).abs() < 1e-12);
        assert!(s.temp_c > 20.0);
        let hot = s.temp_c;
        s.cool(&p, 20.0, 3600.0);
        assert!(s.temp_c < hot);
        assert_eq!(s.dynamic_stops, 1);
    }
}
