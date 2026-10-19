//! 실행 기록 저장소 (SQLite) — 엔드밀 `store/rdb.rs` 의 자리. `store` 기능에서만 빌드된다.
//!
//! 표: runs (실행) · metrics (지표) · series (시계열 점) · decisions (판정)

use crate::cyclesim::CycleReport;
use crate::decision::DecisionReport;
use crate::duty::DutyReport;
use crate::timeseries::Series;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct Store {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRow {
    pub id: i64,
    pub created_unix: i64,
    pub kind: String,
    pub name: String,
    pub crane: String,
    pub detail: String,
}

fn now_unix() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn e2s(e: rusqlite::Error) -> String {
    e.to_string()
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(dir) = path.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
        }
        let s = Self { conn: Connection::open(path).map_err(e2s)? };
        s.migrate()?;
        Ok(s)
    }

    pub fn in_memory() -> Result<Self, String> {
        let s = Self { conn: Connection::open_in_memory().map_err(e2s)? };
        s.migrate()?;
        Ok(s)
    }

    fn migrate(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS runs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    created_unix INTEGER NOT NULL,
                    kind TEXT NOT NULL,
                    name TEXT NOT NULL,
                    crane TEXT NOT NULL,
                    detail TEXT NOT NULL DEFAULT ''
                 );
                 CREATE TABLE IF NOT EXISTS metrics (
                    run_id INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                    key TEXT NOT NULL,
                    value REAL NOT NULL,
                    unit TEXT NOT NULL DEFAULT ''
                 );
                 CREATE TABLE IF NOT EXISTS series (
                    run_id INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                    key TEXT NOT NULL,
                    t REAL NOT NULL,
                    y REAL NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS decisions (
                    run_id INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                    rule_action TEXT NOT NULL,
                    final_action TEXT NOT NULL,
                    report_json TEXT NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS metrics_run ON metrics(run_id);
                 CREATE INDEX IF NOT EXISTS series_run_key ON series(run_id, key);",
            )
            .map_err(e2s)
    }

    fn new_run(&self, kind: &str, name: &str, crane: &str, detail: &str) -> Result<i64, String> {
        self.conn
            .execute("INSERT INTO runs (created_unix, kind, name, crane, detail) VALUES (?1, ?2, ?3, ?4, ?5)", params![now_unix(), kind, name, crane, detail])
            .map_err(e2s)?;
        Ok(self.conn.last_insert_rowid())
    }

    fn put_metrics(&self, run: i64, items: &[(String, f64, &str)]) -> Result<(), String> {
        let tx = self.conn.unchecked_transaction().map_err(e2s)?;
        {
            let mut st = tx.prepare("INSERT INTO metrics (run_id, key, value, unit) VALUES (?1, ?2, ?3, ?4)").map_err(e2s)?;
            for (k, v, u) in items {
                if v.is_finite() {
                    st.execute(params![run, k, v, u]).map_err(e2s)?;
                }
            }
        }
        tx.commit().map_err(e2s)
    }

    pub fn put_series(&self, run: i64, s: &Series) -> Result<(), String> {
        let tx = self.conn.unchecked_transaction().map_err(e2s)?;
        {
            let mut st = tx.prepare("INSERT INTO series (run_id, key, t, y) VALUES (?1, ?2, ?3, ?4)").map_err(e2s)?;
            for (t, y) in s.t.iter().zip(s.y.iter()) {
                st.execute(params![run, s.key, t, y]).map_err(e2s)?;
            }
        }
        tx.commit().map_err(e2s)
    }

    pub fn save_cycle(&self, r: &CycleReport) -> Result<i64, String> {
        let run = self.new_run("cycle", &r.scenario, &r.crane, &r.antisway)?;
        let mut m: Vec<(String, f64, &str)> = vec![
            ("cycle_time_s".into(), r.cycle_time_s, "s"),
            ("load_peak_kg".into(), r.load_peak_kg, "kg"),
            ("max_skew_mm".into(), r.max_skew_mm, "mm"),
            ("rope_damage".into(), r.rope.damage, ""),
            ("rope_min_safety_factor".into(), r.rope.min_safety_factor, ""),
        ];
        for a in r.axes.iter() {
            let k = a.axis.key();
            m.push((format!("{k}.energy_in_kwh"), a.energy_in_kwh, "kWh"));
            m.push((format!("{k}.energy_regen_kwh"), a.energy_regen_kwh, "kWh"));
            m.push((format!("{k}.peak_current_a"), a.peak_current_a, "A"));
            m.push((format!("{k}.rms_current_a"), a.rms_current_a, "A"));
            m.push((format!("{k}.peak_winding_c"), a.peak_winding_c, "°C"));
            m.push((format!("{k}.peak_torque_ratio"), a.peak_torque_ratio, ""));
            m.push((format!("{k}.brake_lining_mm"), a.brake_lining_mm, "mm"));
        }
        for (i, mv) in r.moves.iter().enumerate() {
            m.push((format!("move{}.residual_sway_mm", i + 1), mv.residual_sway_mm, "mm"));
            m.push((format!("move{}.duration_s", i + 1), mv.duration_s, "s"));
        }
        self.put_metrics(run, &m)?;
        // 시계열은 1 Hz 정도로 줄여 저장
        let step = ((1.0 / r.samples.t.get(1).map(|t1| t1 - r.samples.t[0]).unwrap_or(1.0)).round() as usize).max(1);
        let pick = |key: &str, v: &Vec<f64>| -> Result<(), String> {
            let mut s = Series::new(key, "");
            for i in (0..v.len()).step_by(step) {
                s.push(r.samples.t[i], v[i]);
            }
            self.put_series(run, &s)
        };
        pick("sway_mm", &r.samples.sway_mm)?;
        pick("l_m", &r.samples.l)?;
        pick("hoist_winding_c", &r.samples.axes[2].winding_c)?;
        pick("hoist_current_a", &r.samples.axes[2].current_a)?;
        Ok(run)
    }

    pub fn save_duty(&self, d: &DutyReport) -> Result<i64, String> {
        let run = self.new_run("duty", &d.scenario, &d.crane, &d.spectrum_class)?;
        let mut m: Vec<(String, f64, &str)> = vec![
            ("cycles".into(), d.cycles as f64, ""),
            ("hours".into(), d.hours, "h"),
            ("kp".into(), d.kp, ""),
            ("energy_kwh".into(), d.energy_kwh, "kWh"),
            ("rope_damage".into(), d.rope_damage, ""),
            ("full_load_hours".into(), d.full_load_hours, "h"),
            ("swp_used".into(), d.swp_used, ""),
        ];
        for (k, v) in ["travel", "trolley", "hoist"].iter().zip(d.max_winding_c.iter()) {
            m.push((format!("{k}.max_winding_c"), *v, "°C"));
        }
        self.put_metrics(run, &m)?;
        for s in d.series.iter() {
            self.put_series(run, s)?;
        }
        Ok(run)
    }

    pub fn save_decision(&self, run: i64, rep: &DecisionReport) -> Result<(), String> {
        let json = serde_json::to_string(rep).map_err(|e| e.to_string())?;
        self.conn
            .execute("INSERT INTO decisions (run_id, rule_action, final_action, report_json) VALUES (?1, ?2, ?3, ?4)", params![run, rep.rule_action.key(), rep.final_action.key(), json])
            .map_err(e2s)?;
        Ok(())
    }

    pub fn list_runs(&self, limit: usize) -> Result<Vec<RunRow>, String> {
        let mut st = self.conn.prepare("SELECT id, created_unix, kind, name, crane, detail FROM runs ORDER BY id DESC LIMIT ?1").map_err(e2s)?;
        let rows = st
            .query_map(params![limit as i64], |r| Ok(RunRow { id: r.get(0)?, created_unix: r.get(1)?, kind: r.get(2)?, name: r.get(3)?, crane: r.get(4)?, detail: r.get(5)? }))
            .map_err(e2s)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(e2s)
    }

    pub fn metrics(&self, run: i64) -> Result<Vec<(String, f64, String)>, String> {
        let mut st = self.conn.prepare("SELECT key, value, unit FROM metrics WHERE run_id = ?1 ORDER BY key").map_err(e2s)?;
        let rows = st.query_map(params![run], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(e2s)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(e2s)
    }

    pub fn series(&self, run: i64, key: &str) -> Result<Series, String> {
        let mut st = self.conn.prepare("SELECT t, y FROM series WHERE run_id = ?1 AND key = ?2 ORDER BY t").map_err(e2s)?;
        let rows = st.query_map(params![run, key], |r| Ok((r.get::<_, f64>(0)?, r.get::<_, f64>(1)?))).map_err(e2s)?;
        let mut s = Series::new(key, "");
        for row in rows {
            let (t, y) = row.map_err(e2s)?;
            s.push(t, y);
        }
        Ok(s)
    }

    /// 같은 키의 지표를 실행 순서대로 모은 시계열 (사이클 간 추세)
    pub fn metric_history(&self, crane: &str, kind: &str, key: &str) -> Result<Series, String> {
        let mut st = self
            .conn
            .prepare("SELECT runs.id, metrics.value FROM metrics JOIN runs ON runs.id = metrics.run_id WHERE runs.crane = ?1 AND runs.kind = ?2 AND metrics.key = ?3 ORDER BY runs.id")
            .map_err(e2s)?;
        let rows = st.query_map(params![crane, kind, key], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))).map_err(e2s)?;
        let mut s = Series::new(key, "").with_time_unit("run");
        for row in rows {
            let (id, v) = row.map_err(e2s)?;
            s.push(id as f64, v);
        }
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::antisway::AntiSwayMode;
    use crate::cyclesim::{demo_scenario, simulate};

    #[test]
    fn roundtrip_cycle_and_history() {
        let st = Store::in_memory().unwrap();
        let r = simulate(&demo_scenario("overhead_5t_16m", 2000.0, AntiSwayMode::Zv)).unwrap();
        let id1 = st.save_cycle(&r).unwrap();
        let id2 = st.save_cycle(&r).unwrap();
        assert!(id2 > id1);
        let runs = st.list_runs(10).unwrap();
        assert_eq!(runs.len(), 2);
        let m = st.metrics(id1).unwrap();
        assert!(m.iter().any(|(k, _, _)| k == "hoist.energy_in_kwh"));
        let s = st.series(id1, "sway_mm").unwrap();
        assert!(s.len() > 10);
        let h = st.metric_history("overhead_5t_16m", "cycle", "cycle_time_s").unwrap();
        assert_eq!(h.len(), 2);
    }
}
