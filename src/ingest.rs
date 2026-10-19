//! 실측 로그 읽기 (엔드밀 `ingest.rs` 의 자리).
//!
//! - `parse_csv`: 쉼표 · 세미콜론 · 탭 자동 판별, 따옴표 처리.
//! - `CraneLog`: 열 이름 별칭(영어 · 한국어)으로 표준 채널에 맞춘 크레인 로그. 각도 단위(deg/rad)와
//!   길이 단위(mm/m)는 열 이름의 꼬리표로 판별해 SI(m, rad, s)로 바꾼다.
//! - `load_motor_temperature`: Paderborn "Electric Motor Temperature"(Kaggle, measures_v2.csv) 형식.
//!   열: u_q, coolant, stator_winding, u_d, stator_tooth, motor_speed, i_d, i_q, pm, stator_yoke, ambient, torque, profile_id
//!   (2 Hz). 파일이 커서(약 300 MB) 줄 단위로 읽는다.

use std::collections::BTreeMap;
use std::io::BufRead;

#[derive(Debug, Clone, Default)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

fn detect_sep(line: &str) -> char {
    let c = |ch: char| line.matches(ch).count();
    let (comma, semi, tab) = (c(','), c(';'), c('\t'));
    if tab > comma && tab >= semi {
        '\t'
    } else if semi > comma {
        ';'
    } else {
        ','
    }
}

fn split_line(line: &str, sep: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut q = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            if q && chars.peek() == Some(&'"') {
                cur.push('"');
                chars.next();
            } else {
                q = !q;
            }
        } else if ch == sep && !q {
            out.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(ch);
        }
    }
    out.push(cur.trim().to_string());
    out
}

pub fn parse_csv(text: &str) -> Result<Table, String> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'));
    let head = lines.next().ok_or("빈 CSV 입니다")?;
    let head = head.trim_start_matches('\u{feff}');
    let sep = detect_sep(head);
    let headers = split_line(head, sep);
    let rows: Vec<Vec<String>> = lines.map(|l| split_line(l, sep)).collect();
    Ok(Table { headers, rows })
}

impl Table {
    pub fn index(&self, name: &str) -> Option<usize> {
        let n = name.trim().to_ascii_lowercase();
        self.headers.iter().position(|h| h.trim().to_ascii_lowercase() == n)
    }

    pub fn column_f64(&self, idx: usize) -> Vec<f64> {
        self.rows.iter().map(|r| r.get(idx).and_then(|v| v.replace(',', ".").parse::<f64>().ok()).unwrap_or(f64::NAN)).collect()
    }
}

/// 표준 채널: (이름, 별칭 목록)
const CHANNELS: &[(&str, &[&str])] = &[
    ("t", &["t", "time", "time_s", "t_s", "sec", "seconds", "시간", "시각"]),
    ("x", &["x", "bridge_position", "bridge_pos", "travel_position", "gantry_position", "주행위치", "주행_위치"]),
    ("y", &["y", "trolley_position", "trolley_pos", "횡행위치", "트롤리위치"]),
    ("l", &["l", "rope_length", "hoist_position", "hoist_pos", "cable_length", "로프길이", "권상위치"]),
    ("vx", &["vx", "bridge_speed", "travel_speed", "주행속도"]),
    ("vy", &["vy", "trolley_speed", "횡행속도"]),
    ("vl", &["vl", "hoist_speed", "권상속도"]),
    ("theta_x", &["theta_x", "sway_x", "rope_angle_x", "bridge_rope_angle", "angle_x", "흔들림x", "흔들림_x"]),
    ("theta_y", &["theta_y", "sway_y", "rope_angle_y", "trolley_rope_angle", "angle_y", "흔들림y", "흔들림_y"]),
    ("load_kg", &["load", "load_kg", "mass", "tare", "하중"]),
    ("current_a", &["current", "current_a", "i", "i_rms", "전류"]),
    ("torque_nm", &["torque", "torque_nm", "motor_torque", "토크"]),
    ("speed_rpm", &["speed", "rpm", "motor_speed", "speed_rpm", "회전수"]),
    ("winding_c", &["winding", "winding_c", "winding_temp", "stator_winding", "권선온도"]),
    ("ambient_c", &["ambient", "ambient_c", "주변온도"]),
    ("coolant_c", &["coolant", "coolant_c", "냉각수온도"]),
    ("wind_mps", &["wind", "wind_mps", "wind_speed", "풍속"]),
];

/// 열 이름 → (표준 채널, 단위 배율)
pub fn canonical(header: &str) -> Option<(&'static str, f64)> {
    let h = header.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    // 단위 꼬리표 떼기
    let (base, scale_len, scale_ang) = if let Some(b) = h.strip_suffix("_mm") {
        (b.to_string(), 0.001, 1.0)
    } else if let Some(b) = h.strip_suffix("_m") {
        (b.to_string(), 1.0, 1.0)
    } else if let Some(b) = h.strip_suffix("_deg").or_else(|| h.strip_suffix("[deg]")).or_else(|| h.strip_suffix("_도")) {
        (b.to_string(), 1.0, std::f64::consts::PI / 180.0)
    } else if let Some(b) = h.strip_suffix("_rad") {
        (b.to_string(), 1.0, 1.0)
    } else if let Some(b) = h.strip_suffix("_ms") {
        (b.to_string(), 1.0, 1.0)
    } else {
        (h.clone(), 1.0, 1.0)
    };
    for (name, aliases) in CHANNELS {
        for a in aliases.iter() {
            if base == *a || h == *a {
                let scale = if name.starts_with("theta") { scale_ang } else if ["x", "y", "l"].contains(name) { scale_len } else { 1.0 };
                // 시간이 ms 이면
                if *name == "t" && h.ends_with("_ms") {
                    return Some((name, 0.001));
                }
                return Some((name, scale));
            }
        }
    }
    None
}

#[derive(Debug, Clone, Default)]
pub struct CraneLog {
    pub t: Vec<f64>,
    pub channels: BTreeMap<String, Vec<f64>>,
    pub unknown_columns: Vec<String>,
}

impl CraneLog {
    pub fn get(&self, ch: &str) -> Option<&Vec<f64>> {
        self.channels.get(ch)
    }

    /// 두 열에 모두 값이 있는 행만
    pub fn pair(&self, ch: &str) -> Option<(Vec<f64>, Vec<f64>)> {
        let v = self.channels.get(ch)?;
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for (t, y) in self.t.iter().zip(v.iter()) {
            if t.is_finite() && y.is_finite() {
                a.push(*t);
                b.push(*y);
            }
        }
        Some((a, b))
    }
}

/// 크레인 로그 CSV 를 표준 채널로 읽는다. 시간 열이 없으면 `default_dt` 간격으로 만든다.
pub fn load_crane_log(text: &str, default_dt: f64) -> Result<CraneLog, String> {
    let tb = parse_csv(text)?;
    let mut log = CraneLog::default();
    for (i, h) in tb.headers.iter().enumerate() {
        match canonical(h) {
            Some((name, scale)) => {
                let col: Vec<f64> = tb.column_f64(i).into_iter().map(|v| v * scale).collect();
                if name == "t" {
                    log.t = col;
                } else {
                    log.channels.entry(name.to_string()).or_insert(col);
                }
            }
            None => log.unknown_columns.push(h.clone()),
        }
    }
    if log.t.is_empty() {
        log.t = (0..tb.rows.len()).map(|i| i as f64 * default_dt).collect();
    }
    if log.channels.is_empty() {
        return Err(format!("알아볼 수 있는 열이 없습니다: {}", tb.headers.join(", ")));
    }
    Ok(log)
}

/// 모터 온도 시험 한 세션
#[derive(Debug, Clone, Default)]
pub struct MotorTempProfile {
    pub profile_id: i64,
    pub t: Vec<f64>,
    pub current_a: Vec<f64>,
    pub speed: Vec<f64>,
    pub torque: Vec<f64>,
    pub coolant_c: Vec<f64>,
    pub ambient_c: Vec<f64>,
    pub winding_c: Vec<f64>,
}

/// Kaggle "Electric Motor Temperature" 형식 (measures_v2.csv) 을 세션별로 읽는다.
/// `only_profile` 이 있으면 그 세션만 담는다. 표본 간격은 0.5 s (2 Hz).
pub fn load_motor_temperature<R: BufRead>(reader: R, only_profile: Option<i64>, max_rows: Option<usize>) -> Result<BTreeMap<i64, MotorTempProfile>, String> {
    let mut lines = reader.lines();
    let head = lines.next().ok_or("빈 파일입니다")?.map_err(|e| e.to_string())?;
    let head = head.trim_start_matches('\u{feff}').to_string();
    let sep = detect_sep(&head);
    let hs: Vec<String> = split_line(&head, sep).into_iter().map(|h| h.to_ascii_lowercase()).collect();
    let idx = |n: &str| hs.iter().position(|h| h == n);
    let need = |n: &str| idx(n).ok_or_else(|| format!("'{}' 열이 없습니다 (Kaggle measures_v2.csv 형식이 아닙니다)", n));
    let (i_d, i_q, sp, tq, cool, amb, wind, pid) = (need("i_d")?, need("i_q")?, need("motor_speed")?, need("torque")?, need("coolant")?, need("ambient")?, need("stator_winding")?, need("profile_id")?);
    let mut out: BTreeMap<i64, MotorTempProfile> = BTreeMap::new();
    for (n, line) in lines.enumerate() {
        if max_rows.map(|m| n >= m).unwrap_or(false) {
            break;
        }
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let f = split_line(&line, sep);
        let g = |k: usize| f.get(k).and_then(|v| v.parse::<f64>().ok()).unwrap_or(f64::NAN);
        let p = g(pid) as i64;
        if let Some(o) = only_profile {
            if p != o {
                continue;
            }
        }
        let e = out.entry(p).or_insert_with(|| MotorTempProfile { profile_id: p, ..Default::default() });
        e.t.push(e.t.len() as f64 * 0.5);
        e.current_a.push((g(i_d).powi(2) + g(i_q).powi(2)).sqrt());
        e.speed.push(g(sp));
        e.torque.push(g(tq));
        e.coolant_c.push(g(cool));
        e.ambient_c.push(g(amb));
        e.winding_c.push(g(wind));
    }
    if out.is_empty() {
        return Err("읽은 행이 없습니다".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_with_aliases_units_and_korean_headers() {
        let txt = "시간,trolley_position_mm,sway_x_deg,\"hoist_speed\",unknown\n0,1000,1.0,0.1,a\n0.1,1010,0.5,0.1,b\n";
        let log = load_crane_log(txt, 0.01).unwrap();
        assert_eq!(log.t, vec![0.0, 0.1]);
        assert!((log.get("y").unwrap()[1] - 1.01).abs() < 1e-12);
        assert!((log.get("theta_x").unwrap()[0] - 1.0f64.to_radians()).abs() < 1e-12);
        assert_eq!(log.unknown_columns, vec!["unknown".to_string()]);
    }

    #[test]
    fn semicolon_and_decimal_comma() {
        let t = parse_csv("a;b\n1,5;2\n").unwrap();
        assert_eq!(t.headers, vec!["a", "b"]);
        assert_eq!(t.column_f64(0), vec![1.5]);
    }

    #[test]
    fn kaggle_motor_temperature_format() {
        let txt = "u_q,coolant,stator_winding,u_d,stator_tooth,motor_speed,i_d,i_q,pm,stator_yoke,ambient,torque,profile_id\n\
                   1,20,30,1,25,1000,3,4,30,22,21,10,4\n\
                   1,20,31,1,25,1000,3,4,30,22,21,10,4\n\
                   1,20,40,1,25,1000,6,8,30,22,21,10,7\n";
        let m = load_motor_temperature(std::io::Cursor::new(txt), None, None).unwrap();
        assert_eq!(m.len(), 2);
        let p = &m[&4];
        assert_eq!(p.t, vec![0.0, 0.5]);
        assert_eq!(p.current_a, vec![5.0, 5.0]);
        let only = load_motor_temperature(std::io::Cursor::new(txt), Some(7), None).unwrap();
        assert_eq!(only.len(), 1);
        assert_eq!(only[&7].current_a, vec![10.0]);
    }
}
