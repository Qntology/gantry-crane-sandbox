//! 통계 동역학 원장 (SDS) — endmill-sandbox `sds.rs` 를 크레인 원장으로 옮긴 축약판.
//!
//! - 원장: 모터 · 브레이크 · 로프 · 흔들림 · 구조 · 결정. 원장마다 JSON 파일 하나.
//! - 값마다 Welford 누적 통계(개수 · 평균 · M2) + 최근 값 링버퍼 → 드리프트 z = (최근 평균 − 누적 평균) / 표준편차.
//! - 보정 계수: 측정/예측 비율의 평균을 표본 수로 수축 (1 + n/(n + n0)·(평균 − 1)).
//! - 게이트 통과율, 규칙–조언 · 작업자–조언 일치율.

use crate::cyclesim::CycleReport;
use crate::decision::Gate;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

pub const SDS_RECIPE: &str = "crane-sds-v1:welford+ring/ledger-split/calib-shrink";
pub const RING_LEN: usize = 12;
pub const MIN_OBS_DRIFT: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ledger {
    Motor,
    Brake,
    Rope,
    Sway,
    Structure,
    Decision,
}

impl Ledger {
    pub fn as_str(&self) -> &'static str {
        match self {
            Ledger::Motor => "motor",
            Ledger::Brake => "brake",
            Ledger::Rope => "rope",
            Ledger::Sway => "sway",
            Ledger::Structure => "structure",
            Ledger::Decision => "decision",
        }
    }
    pub fn file_name(&self) -> String {
        format!("{}.json", self.as_str())
    }
    pub fn all() -> [Ledger; 6] {
        [Ledger::Motor, Ledger::Brake, Ledger::Rope, Ledger::Sway, Ledger::Structure, Ledger::Decision]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scope {
    pub crane: String,
    /// "travel" · "trolley" · "hoist" · "all"
    pub axis: String,
}

impl Scope {
    pub fn new(crane: &str, axis: &str) -> Self {
        Self { crane: crane.into(), axis: axis.into() }
    }
    fn key(&self, metric: &str) -> String {
        format!("{}/{}/{}", self.crane, self.axis, metric)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Welford {
    pub n: u64,
    pub mean: f64,
    pub m2: f64,
    #[serde(default)]
    pub ring: Vec<f64>,
}

impl Welford {
    pub fn push(&mut self, x: f64, ring_len: usize) {
        if !x.is_finite() {
            return;
        }
        self.n += 1;
        let d = x - self.mean;
        self.mean += d / self.n as f64;
        self.m2 += d * (x - self.mean);
        self.ring.push(x);
        if ring_len > 0 && self.ring.len() > ring_len {
            let excess = self.ring.len() - ring_len;
            self.ring.drain(0..excess);
        }
    }

    pub fn variance(&self) -> f64 {
        if self.n < 2 {
            0.0
        } else {
            self.m2 / (self.n - 1) as f64
        }
    }

    pub fn sd(&self) -> f64 {
        self.variance().max(0.0).sqrt()
    }

    pub fn recent_mean(&self) -> f64 {
        if self.ring.is_empty() {
            return self.mean;
        }
        self.ring.iter().sum::<f64>() / self.ring.len() as f64
    }

    pub fn drift_z(&self) -> f64 {
        let sd = self.sd();
        if sd <= 0.0 || self.ring.is_empty() {
            return 0.0;
        }
        (self.recent_mean() - self.mean) / sd
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GateStat {
    pub evaluated: u64,
    pub failed: u64,
    pub severity_sum: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LedgerFile {
    pub recipe: String,
    pub stats: BTreeMap<String, Welford>,
    #[serde(default)]
    pub gates: BTreeMap<String, GateStat>,
    /// 이름 → (일치, 전체)
    #[serde(default)]
    pub agreements: BTreeMap<String, (u64, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftItem {
    pub ledger: Ledger,
    pub key: String,
    pub z: f64,
    pub n: u64,
    pub mean: f64,
    pub recent: f64,
}

pub struct SdsStore {
    dir: Option<PathBuf>,
    files: HashMap<Ledger, LedgerFile>,
    dirty: bool,
}

impl SdsStore {
    pub fn in_memory() -> Self {
        Self { dir: None, files: HashMap::new(), dirty: false }
    }

    /// 디렉터리의 원장 파일을 읽는다 (없으면 빈 원장)
    pub fn open(dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
        let mut files = HashMap::new();
        for l in Ledger::all() {
            let p = dir.join(l.file_name());
            if p.exists() {
                let txt = std::fs::read_to_string(&p).map_err(|e| format!("{}: {}", p.display(), e))?;
                let f: LedgerFile = serde_json::from_str(&txt).map_err(|e| format!("{}: {}", p.display(), e))?;
                files.insert(l, f);
            }
        }
        Ok(Self { dir: Some(dir.to_path_buf()), files, dirty: false })
    }

    fn file(&mut self, l: Ledger) -> &mut LedgerFile {
        self.dirty = true;
        self.files.entry(l).or_insert_with(|| LedgerFile { recipe: SDS_RECIPE.into(), ..Default::default() })
    }

    pub fn record(&mut self, l: Ledger, scope: &Scope, metric: &str, value: f64) {
        let k = scope.key(metric);
        self.file(l).stats.entry(k).or_default().push(value, RING_LEN);
    }

    pub fn stat(&self, l: Ledger, scope: &Scope, metric: &str) -> Option<&Welford> {
        self.files.get(&l).and_then(|f| f.stats.get(&scope.key(metric)))
    }

    /// 드리프트 z (표본이 충분할 때만)
    pub fn drift(&self, l: Ledger, scope: &Scope, metric: &str) -> Option<f64> {
        let w = self.stat(l, scope, metric)?;
        if w.n < MIN_OBS_DRIFT || w.ring.len() < 3 {
            return None;
        }
        Some(w.drift_z())
    }

    /// 측정/예측 비율을 기록한다 (`calib/<metric>`)
    pub fn record_calibration(&mut self, l: Ledger, scope: &Scope, metric: &str, measured: f64, predicted: f64) {
        if predicted.abs() > 1e-12 && measured.is_finite() {
            self.record(l, scope, &format!("calib/{metric}"), measured / predicted);
        }
    }

    /// 수축된 보정 계수와 표본 수
    pub fn calibration_factor(&self, l: Ledger, scope: &Scope, metric: &str, n0: f64) -> Option<(f64, u64)> {
        let w = self.stat(l, scope, &format!("calib/{metric}"))?;
        if w.n == 0 {
            return None;
        }
        let n = w.n as f64;
        Some((1.0 + n / (n + n0.max(0.0)) * (w.mean - 1.0), w.n))
    }

    pub fn record_gates(&mut self, scope: &Scope, gates: &[Gate]) {
        let f = self.file(Ledger::Decision);
        for g in gates {
            let s = f.gates.entry(scope.key(&g.id)).or_default();
            s.evaluated += 1;
            if g.severity > 0 {
                s.failed += 1;
            }
            s.severity_sum += g.severity as u64;
        }
    }

    /// (게이트, 실패율, 평가 수)
    pub fn gate_rates(&self, scope: &Scope) -> Vec<(String, f64, u64)> {
        let prefix = scope.key("");
        self.files
            .get(&Ledger::Decision)
            .map(|f| {
                f.gates
                    .iter()
                    .filter(|(k, _)| k.starts_with(&prefix))
                    .map(|(k, s)| (k[prefix.len()..].to_string(), s.failed as f64 / s.evaluated.max(1) as f64, s.evaluated))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn record_agreement(&mut self, scope: &Scope, name: &str, agreed: bool) {
        let e = self.file(Ledger::Decision).agreements.entry(scope.key(name)).or_insert((0, 0));
        if agreed {
            e.0 += 1;
        }
        e.1 += 1;
    }

    pub fn agreement_rate(&self, scope: &Scope, name: &str) -> Option<(f64, u64)> {
        let (a, n) = self.files.get(&Ledger::Decision)?.agreements.get(&scope.key(name))?;
        if *n == 0 {
            None
        } else {
            Some((*a as f64 / *n as f64, *n))
        }
    }

    /// 사이클 결과에서 기준선 값을 기록한다
    pub fn ingest_cycle(&mut self, crane: &str, c: &CycleReport) {
        for a in c.axes.iter() {
            let s = Scope::new(crane, a.axis.key());
            self.record(Ledger::Motor, &s, "rms_current_ratio", a.rms_current_a / a.rated_current_a.max(1e-9));
            self.record(Ledger::Motor, &s, "peak_torque_ratio", a.peak_torque_ratio);
            self.record(Ledger::Motor, &s, "energy_kwh", a.energy_in_kwh);
            self.record(Ledger::Motor, &s, "peak_winding_c", a.peak_winding_c);
            self.record(Ledger::Brake, &s, "brake_work_j", a.brake_work_j);
        }
        let all = Scope::new(crane, "all");
        let sway = c.moves.iter().map(|m| m.residual_sway_mm).fold(0.0, f64::max);
        self.record(Ledger::Sway, &all, "residual_sway_mm", sway);
        self.record(Ledger::Rope, &all, "max_tension_per_fall_kn", c.rope.max_tension_per_fall_kn);
        self.record(Ledger::Structure, &all, "max_skew_mm", c.max_skew_mm);
    }

    /// 표본이 충분한 모든 값의 드리프트 — 결정 입력용
    pub fn drift_report(&self, crane: &str) -> Vec<DriftItem> {
        let prefix = format!("{}/", crane);
        let mut out = Vec::new();
        for (l, f) in self.files.iter() {
            for (k, w) in f.stats.iter() {
                if !k.starts_with(&prefix) || k.contains("/calib/") || w.n < MIN_OBS_DRIFT || w.ring.len() < 3 {
                    continue;
                }
                out.push(DriftItem { ledger: *l, key: k[prefix.len()..].to_string(), z: w.drift_z(), n: w.n, mean: w.mean, recent: w.recent_mean() });
            }
        }
        out.sort_by(|a, b| b.z.abs().partial_cmp(&a.z.abs()).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    /// 변경된 원장을 디렉터리에 쓴다
    pub fn flush(&mut self) -> Result<Vec<PathBuf>, String> {
        let Some(dir) = self.dir.clone() else { return Ok(Vec::new()) };
        if !self.dirty {
            return Ok(Vec::new());
        }
        let mut written = Vec::new();
        for (l, f) in self.files.iter() {
            let p = dir.join(l.file_name());
            let txt = serde_json::to_string_pretty(f).map_err(|e| e.to_string())?;
            std::fs::write(&p, txt).map_err(|e| format!("{}: {}", p.display(), e))?;
            written.push(p);
        }
        self.dirty = false;
        Ok(written)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welford_matches_direct_stats() {
        let xs = [3.0, 5.0, 4.0, 8.0, 6.0, 7.0];
        let mut w = Welford::default();
        for x in xs {
            w.push(x, 3);
        }
        let mean = xs.iter().sum::<f64>() / 6.0;
        let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / 5.0;
        assert!((w.mean - mean).abs() < 1e-12);
        assert!((w.variance() - var).abs() < 1e-12);
        assert_eq!(w.ring, vec![8.0, 6.0, 7.0]);
    }

    #[test]
    fn drift_detects_shift_and_shrinks_calibration() {
        let mut s = SdsStore::in_memory();
        let sc = Scope::new("c1", "hoist");
        for i in 0..40 {
            s.record(Ledger::Motor, &sc, "rms_current_ratio", 0.6 + 0.01 * ((i % 5) as f64 - 2.0));
        }
        assert!(s.drift(Ledger::Motor, &sc, "rms_current_ratio").unwrap().abs() < 1.0);
        for _ in 0..12 {
            s.record(Ledger::Motor, &sc, "rms_current_ratio", 0.75);
        }
        assert!(s.drift(Ledger::Motor, &sc, "rms_current_ratio").unwrap() > 1.0);
        for _ in 0..4 {
            s.record_calibration(Ledger::Motor, &sc, "winding", 110.0, 100.0);
        }
        let (k, n) = s.calibration_factor(Ledger::Motor, &sc, "winding", 4.0).unwrap();
        assert_eq!(n, 4);
        assert!((k - 1.05).abs() < 1e-9);
        let items = s.drift_report("c1");
        assert!(!items.is_empty());
    }

    #[test]
    fn persists_and_reloads() {
        let dir = std::env::temp_dir().join(format!("crane_sds_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        {
            let mut s = SdsStore::open(&dir).unwrap();
            s.record(Ledger::Rope, &Scope::new("c", "all"), "x", 2.0);
            s.record_agreement(&Scope::new("c", "all"), "rule_laya", true);
            assert_eq!(s.flush().unwrap().len(), 2);
        }
        let s2 = SdsStore::open(&dir).unwrap();
        assert_eq!(s2.stat(Ledger::Rope, &Scope::new("c", "all"), "x").unwrap().n, 1);
        assert_eq!(s2.agreement_rate(&Scope::new("c", "all"), "rule_laya"), Some((1.0, 1)));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
