//! 모델 관리 — 설정(`settings.json`) · 내려받기(`modelhub`) · 불러온 모델 · 시작 시 자동 준비.
//!
//! 앱이 뜨면(`start`) 없는 모델을 바로 내려받기 시작하고, 다 받은 모델은 백그라운드 스레드가 불러와 다음 실행부터 쓴다.
//! endmill-sandbox 의 '환경·모델 설정'과 같은 명령 이름(`models_catalog` · `models_download` · `settings_set_models` …)을 쓴다.
//! 명령(`call`)은 계산 중인 Backend 잠금과 상관없이 답하므로, 긴 현장 운영 시뮬레이션 중에도 진행률을 볼 수 있다.

use crate::api::{self, ModelStatus, Models, MODEL_KEYS};
use crate::modelhub::{self, DownloadJob, InstallStatus, ModelFile, ModelHub};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    /// 실행에 필요한 모델이 없으면 백그라운드로 받기 시작 (이번 실행은 대체 방식)
    #[serde(default = "yes")]
    pub auto_download: bool,
    /// 앱을 켤 때 없는 모델을 모두 받기 — 크레인 모델은 합쳐 약 0.85 GB 라 기본으로 켠다
    #[serde(default = "yes")]
    pub startup_download: bool,
    /// 받아 둔 모델을 앱이 뜰 때 · 다 받았을 때 바로 불러오기
    #[serde(default = "yes")]
    pub auto_load: bool,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub token: String,
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self { auto_download: true, startup_download: true, auto_load: true, endpoint: String::new(), token: String::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub models: ModelSettings,
}

/// 모델 하나의 불러오기 상태
#[derive(Debug, Clone, Default)]
struct Slot {
    loaded: bool,
    loading: bool,
    /// 작업자가 언로드 · 삭제했으면 자동으로 다시 불러오지 않는다
    held: bool,
    dir: Option<String>,
    detail: String,
    error: Option<String>,
    /// 실패한 폴더 · 크기 — 같은 파일로는 다시 시도하지 않는다 (새로 받거나 '로드'를 누르면 다시)
    failed_sig: Option<(String, u64)>,
}

#[derive(Serialize)]
pub struct ModelEntry {
    pub key: String,
    pub label: String,
    pub repo: String,
    pub revision: String,
    pub dir: String,
    pub used_for: String,
    pub fallback: Option<String>,
    pub files: Vec<ModelFile>,
    pub total_bytes: u64,
    pub installed: bool,
    pub detected_dir: Option<String>,
    pub on_disk_bytes: u64,
    pub partial_bytes: u64,
    pub loaded: bool,
    pub loading: bool,
    pub held: bool,
    pub load_detail: String,
    pub load_error: Option<String>,
    pub job: Option<DownloadJob>,
}

#[derive(Serialize)]
pub struct ModelCatalog {
    pub root: String,
    pub device: String,
    pub endpoint: String,
    pub token_set: bool,
    pub auto_download: bool,
    pub startup_download: bool,
    pub auto_load: bool,
    pub any_active: bool,
    pub ml_feature: bool,
    /// endmill-sandbox 가 이미 받아 둔 모델 폴더 (있으면 '이 폴더 쓰기' 제안)
    pub sibling_root: Option<String>,
    pub entries: Vec<ModelEntry>,
}

pub struct ModelService {
    data_dir: PathBuf,
    hub: Mutex<ModelHub>,
    settings: Mutex<AppSettings>,
    models: Mutex<Models>,
    slots: Mutex<BTreeMap<String, Slot>>,
    started: AtomicBool,
    shutdown: AtomicBool,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match m.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}

fn arg_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn arg_key(args: &Value) -> Result<String, String> {
    let k = arg_str(args, "key").ok_or("인자 'key' 가 없습니다")?;
    if modelhub::spec_of(&k).is_none() {
        return Err(format!("알 수 없는 모델: {k}"));
    }
    Ok(k)
}

fn to_value<T: Serialize>(v: &T) -> Result<Value, String> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

/// 폴더 안 safetensors 크기 합 — 실패한 로드를 같은 파일로 되풀이하지 않기 위한 표지
fn weights_sig(dir: &Path) -> (String, u64) {
    let size = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().extension().map(|x| x == "safetensors").unwrap_or(false))
                .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
                .sum()
        })
        .unwrap_or(0);
    (dir.display().to_string(), size)
}

fn same_dir(a: &Path, b: &Path) -> bool {
    let a = std::fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let b = std::fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    a == b
}

/// root 아래에서 찾은 폴더 중 쓸 수 있는 것 — 받는 도중인 자기 폴더(.part 가 있거나 가중치가 아직 없음)는 뺀다 (endmill 과 같은 규칙)
fn detected_dir(root: &Path, key: &str, own: &InstallStatus) -> Option<PathBuf> {
    let own_dir = Path::new(&own.dir);
    if own.installed {
        return Some(own_dir.to_path_buf());
    }
    let own_incomplete = own.partial_bytes > 0 || !own_dir.join("model.safetensors").exists();
    api::find_model_dirs(root, key).into_iter().find(|d| !(own_incomplete && same_dir(d, own_dir)))
}

fn device_label() -> String {
    #[cfg(feature = "ml")]
    if let Some(d) = crate::ml::device_label_if_ready() {
        return d;
    }
    if cfg!(feature = "ml") {
        "첫 로드 때 정함".into()
    } else {
        "ml 기능 없음".into()
    }
}

impl ModelService {
    /// 설정과 모델 폴더만 읽는다 (내려받기 · 불러오기는 `start`)
    pub fn open(data_dir: &Path) -> Arc<Self> {
        let _ = std::fs::create_dir_all(data_dir);
        let settings: AppSettings = std::fs::read_to_string(data_dir.join("settings.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        let root = std::fs::read_to_string(data_dir.join("models_root.txt"))
            .ok()
            .map(|s| PathBuf::from(s.trim()))
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| data_dir.join("models"));
        let hub = ModelHub::new(&root, Some(&settings.models.endpoint), Some(&settings.models.token));
        let slots = MODEL_KEYS.iter().map(|k| (k.to_string(), Slot::default())).collect();
        Arc::new(Self {
            data_dir: data_dir.to_path_buf(),
            hub: Mutex::new(hub),
            settings: Mutex::new(settings),
            models: Mutex::new(Models::none()),
            slots: Mutex::new(slots),
            started: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
        })
    }

    /// 앱 시작: 없는 모델을 받기 시작하고(설정이 켜져 있으면), 받아 둔 · 다 받은 모델을 불러오는 스레드를 띄운다
    pub fn start(self: &Arc<Self>, download: bool) {
        if self.started.swap(true, Ordering::SeqCst) {
            return;
        }
        if download && lock(&self.settings).models.startup_download {
            self.download_missing();
        }
        let me = self.clone();
        let _ = std::thread::Builder::new().name("model-keeper".into()).spawn(move || {
            while !me.shutdown.load(Ordering::SeqCst) {
                me.keeper_tick();
                std::thread::sleep(Duration::from_millis(400));
            }
        });
    }

    /// 앱 종료: 내려받기를 멈춘다 (받은 부분은 이어받기용으로 남는다)
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let hub = lock(&self.hub);
        for k in MODEL_KEYS {
            hub.cancel(k);
        }
        for k in MODEL_KEYS {
            hub.wait(k, Duration::from_secs(3));
        }
    }

    pub fn root(&self) -> PathBuf {
        lock(&self.hub).root.clone()
    }

    pub fn settings(&self) -> AppSettings {
        lock(&self.settings).clone()
    }

    /// 불러온 모델로 계산한다 (계산하는 동안 다른 스레드가 모델을 바꾸지 못한다)
    pub fn with_models<R>(&self, f: impl FnOnce(&Models) -> R) -> R {
        f(&lock(&self.models))
    }

    /// 쓸 수 있는 모델 폴더 — 내려받기가 끝난 자기 폴더, 아니면 root 아래에서 찾은 다른 폴더 (덜 받은 자기 폴더는 뺀다)
    fn usable_dir(&self, key: &str) -> Option<PathBuf> {
        let spec = modelhub::spec_of(key)?;
        let root = self.root();
        detected_dir(&root, key, &modelhub::install_status(&root, &spec))
    }

    fn job_active(&self, key: &str) -> bool {
        lock(&self.hub).job(key).map(|j| j.active()).unwrap_or(false)
    }

    /// 없는 모델을 모두 받기 시작한다. 시작한 모델 키를 돌려준다.
    pub fn download_missing(&self) -> Vec<String> {
        let mut started = Vec::new();
        for k in MODEL_KEYS {
            if self.usable_dir(k).is_some() || self.job_active(k) {
                continue;
            }
            if lock(&self.hub).start(k).is_ok() {
                started.push(k.to_string());
            }
        }
        started
    }

    fn keeper_tick(&self) {
        if !lock(&self.settings).models.auto_load {
            return;
        }
        for k in MODEL_KEYS {
            if self.shutdown.load(Ordering::SeqCst) {
                return;
            }
            let skip = {
                let slots = lock(&self.slots);
                let s = &slots[k];
                s.loaded || s.loading || s.held
            };
            if skip || self.job_active(k) {
                continue;
            }
            let Some(dir) = self.usable_dir(k) else { continue };
            let failed = lock(&self.slots)[k].failed_sig.clone();
            if failed.as_ref() == Some(&weights_sig(&dir)) {
                continue;
            }
            let _ = self.load_key(k, &dir);
        }
    }

    /// 모델 하나를 불러온다 — 가중치는 잠금 밖에서 읽고, 다 읽은 뒤에 바꿔 끼운다
    fn load_key(&self, key: &str, dir: &Path) -> Result<String, String> {
        {
            let mut slots = lock(&self.slots);
            let s = slots.get_mut(key).ok_or_else(|| format!("알 수 없는 모델: {key}"))?;
            if s.loaded {
                return Ok(s.detail.clone());
            }
            if s.loading {
                return Err("이미 불러오는 중입니다".into());
            }
            s.loading = true;
            s.error = None;
        }
        let r = api::load_model(key, dir);
        // 계산 중이면 모델 잠금을 기다린다 — 그동안 상태 조회가 막히지 않도록 slots 잠금은 그 뒤에 잡는다
        let r = r.map(|(m, detail)| {
            lock(&self.models).insert(m);
            detail
        });
        let mut slots = lock(&self.slots);
        let s = slots.get_mut(key).unwrap();
        s.loading = false;
        match r {
            Ok(detail) => {
                s.loaded = true;
                s.held = false;
                s.dir = Some(dir.display().to_string());
                s.detail = detail.clone();
                s.failed_sig = None;
                Ok(detail)
            }
            Err(e) => {
                s.error = Some(e.clone());
                s.failed_sig = Some(weights_sig(dir));
                Err(e)
            }
        }
    }

    fn unload_key(&self, key: &str) {
        let was = lock(&self.models).unload(key);
        let mut slots = lock(&self.slots);
        if let Some(s) = slots.get_mut(key) {
            *s = Slot { held: true, ..Slot::default() };
        }
        drop(slots);
        if was {
            #[cfg(feature = "ml")]
            crate::ml::flush_device();
        }
    }

    /// 파이프라인 실행 전: 모델이 없고 자동 받기가 켜져 있으면 받기 시작하고, 보고서에 붙일 메모를 돌려준다
    pub fn ensure_for_run(&self) -> Vec<String> {
        let mut notes = Vec::new();
        let auto = lock(&self.settings).models.auto_download;
        for k in MODEL_KEYS {
            let name = if k == "ttm" { "TTM-R3" } else { "LAYA" };
            let (loaded, loading) = {
                let slots = lock(&self.slots);
                (slots[k].loaded, slots[k].loading)
            };
            if loaded {
                continue;
            }
            if loading {
                notes.push(format!("{name} 을(를) 불러오는 중이라 이번 실행은 대체 방식을 썼습니다"));
                continue;
            }
            if let Some(j) = lock(&self.hub).job(k).filter(|j| j.active()) {
                notes.push(format!("{name} 내려받는 중 ({:.0}%) — 이번 실행은 대체 방식을 썼습니다", j.percent()));
                continue;
            }
            if self.usable_dir(k).is_none() && auto && cfg!(feature = "ml") {
                let failed = lock(&self.hub).job(k).map(|j| j.state == "failed").unwrap_or(false);
                if !failed && lock(&self.hub).start(k).is_ok() {
                    notes.push(format!("{name} 모델 파일이 없어 백그라운드 다운로드를 시작했습니다 (이번 실행은 대체 방식)"));
                }
            }
        }
        notes
    }

    /// 작업이 모델을 쓰기 전에 부른다 (endmill-sandbox `models_ensure` 와 같은 답):
    /// ready(로드됨) · loading(불러오는 중) · downloading(받는 중) · missing(없음, 자동 받기 꺼짐) · failed
    pub fn ensure(&self, key: &str, download: Option<bool>) -> Value {
        let respond = |state: &str, detail: String, job: Option<DownloadJob>| json!({"key": key, "state": state, "detail": detail, "job": job});
        let job = lock(&self.hub).job(key);
        if let Some(j) = job.as_ref().filter(|j| j.active()) {
            return respond("downloading", format!("{:.1}%", j.percent()), Some(j.clone()));
        }
        let (loaded, loading, detail) = {
            let slots = lock(&self.slots);
            let s = &slots[key];
            (s.loaded, s.loading, s.detail.clone())
        };
        if loaded {
            return respond("ready", detail, job);
        }
        if loading {
            return respond("loading", "불러오는 중".into(), job);
        }
        if let Some(dir) = self.usable_dir(key) {
            if let Some(s) = lock(&self.slots).get_mut(key) {
                s.held = false;
                s.failed_sig = None;
            }
            return match self.load_key(key, &dir) {
                Ok(d) => respond("ready", d, job),
                Err(e) if lock(&self.slots)[key].loading => respond("loading", e, job),
                Err(e) => respond("failed", e, job),
            };
        }
        if let Some(j) = job.as_ref().filter(|j| j.state == "failed") {
            if !download.unwrap_or(false) {
                return respond("failed", j.error.clone().unwrap_or_default(), Some(j.clone()));
            }
        }
        let auto = lock(&self.settings).models.auto_download;
        if download.unwrap_or(auto) {
            return match lock(&self.hub).start(key) {
                Ok(j) => respond("downloading", "다운로드 시작".into(), Some(j)),
                Err(e) => respond("failed", e, job),
            };
        }
        respond("missing", "모델 파일이 없습니다 (설정 → 모델에서 다운로드)".into(), job)
    }

    /// 화면용 모델 목록 · 상태
    pub fn catalog(&self) -> ModelCatalog {
        let ms = lock(&self.settings).models.clone();
        let (root, endpoint, token_set, raw, any_active) = {
            let hub = lock(&self.hub);
            let raw: Vec<_> = modelhub::catalog()
                .into_iter()
                .map(|spec| {
                    let st = modelhub::install_status(&hub.root, &spec);
                    let job = hub.job(&spec.key);
                    (spec, st, job)
                })
                .collect();
            (hub.root.clone(), hub.endpoint.clone(), hub.token.is_some(), raw, hub.any_active())
        };
        let slots = lock(&self.slots).clone();
        let entries = raw
            .into_iter()
            .map(|(spec, st, job)| {
                let slot = slots.get(&spec.key).cloned().unwrap_or_default();
                let detected = detected_dir(&root, &spec.key, &st).map(|d| d.display().to_string());
                ModelEntry {
                    key: spec.key.clone(),
                    label: spec.label.clone(),
                    repo: spec.repo.clone(),
                    revision: spec.revision.clone(),
                    dir: st.dir.clone(),
                    used_for: spec.used_for.clone(),
                    fallback: spec.fallback.clone(),
                    total_bytes: spec.total_bytes(),
                    files: spec.files.clone(),
                    installed: st.installed,
                    detected_dir: slot.dir.clone().or(detected),
                    on_disk_bytes: st.on_disk_bytes,
                    partial_bytes: st.partial_bytes,
                    loaded: slot.loaded,
                    loading: slot.loading,
                    held: slot.held,
                    load_detail: slot.detail.clone(),
                    load_error: slot.error.clone(),
                    job,
                }
            })
            .collect();
        ModelCatalog {
            root: root.display().to_string(),
            device: device_label(),
            endpoint,
            token_set,
            auto_download: ms.auto_download,
            startup_download: ms.startup_download,
            auto_load: ms.auto_load,
            any_active,
            ml_feature: cfg!(feature = "ml"),
            sibling_root: self.sibling_root().filter(|s| !same_dir(s, &root)).map(|p| p.display().to_string()),
            entries,
        }
    }

    /// 예전 화면 · CLI 가 보는 요약 (`PipelineReport.models` 와 같은 꼴)
    pub fn status(&self) -> ModelStatus {
        let slots = lock(&self.slots).clone();
        let mut notes = vec![format!("장치: {}", device_label())];
        for k in MODEL_KEYS {
            let name = if k == "ttm" { "TTM" } else { "LAYA" };
            let s = &slots[k];
            let job = lock(&self.hub).job(k).filter(|j| j.active());
            notes.push(match (s.loaded, s.loading, job, &s.error) {
                (true, _, _, _) => format!("{name}: {}", s.dir.clone().unwrap_or_default()),
                (_, true, _, _) => format!("{name}: 불러오는 중"),
                (_, _, Some(j), _) => format!("{name}: 내려받는 중 {:.0}%", j.percent()),
                (_, _, _, Some(e)) => format!("{name} 로드 실패: {e}"),
                _ => format!("{name}: 없음 (설정 → 모델에서 받기)"),
            });
        }
        ModelStatus { ttm: slots["ttm"].loaded, advisor: slots["laya"].loaded, ml_feature: cfg!(feature = "ml"), notes }
    }

    /// endmill-sandbox 앱 데이터 폴더에 받아 둔 모델 (같은 Hugging Face 판) — 다시 받지 않고 그 폴더를 쓸 수 있다
    pub fn sibling_root(&self) -> Option<PathBuf> {
        let base = self.data_dir.parent()?.join("com.qntology.endmill-sandbox");
        let root = std::fs::read_to_string(base.join("models_root.txt"))
            .ok()
            .map(|s| PathBuf::from(s.trim()))
            .filter(|p| p.is_dir())
            .unwrap_or_else(|| base.join("models"));
        MODEL_KEYS.iter().any(|k| api::find_model_dir(&root, k).is_some()).then_some(root)
    }

    fn save_settings(&self) -> Result<(), String> {
        let s = lock(&self.settings).clone();
        let p = self.data_dir.join("settings.json");
        std::fs::write(&p, serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", p.display()))
    }

    pub fn set_root(&self, root: &Path, persist: bool) -> Result<(), String> {
        if root.as_os_str().is_empty() {
            return Err("모델 폴더를 적어 주세요".into());
        }
        std::fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
        lock(&self.hub).set_root(root)?;
        if persist {
            let p = self.data_dir.join("models_root.txt");
            std::fs::write(&p, root.display().to_string()).map_err(|e| format!("{}: {e}", p.display()))?;
        }
        for k in MODEL_KEYS {
            let was = lock(&self.models).unload(k);
            lock(&self.slots).insert(k.to_string(), Slot::default());
            if was {
                #[cfg(feature = "ml")]
                crate::ml::flush_device();
            }
        }
        Ok(())
    }

    fn settings_view(&self) -> Value {
        let s = lock(&self.settings).models.clone();
        json!({
            "models": {
                "auto_download": s.auto_download,
                "startup_download": s.startup_download,
                "auto_load": s.auto_load,
                "endpoint": if s.endpoint.trim().is_empty() { modelhub::DEFAULT_ENDPOINT.to_string() } else { s.endpoint.clone() },
                "token_set": !s.token.trim().is_empty(),
            },
            "models_root": self.root().display().to_string(),
            "data_dir": self.data_dir.display().to_string(),
            "ml_feature": cfg!(feature = "ml"),
        })
    }

    /// 모델 · 설정 명령. 이 서비스가 맡지 않는 명령이면 None.
    pub fn call(&self, cmd: &str, args: &Value) -> Option<Result<Value, String>> {
        let r = match cmd {
            "models_catalog" => to_value(&self.catalog()),
            "models_status" => to_value(&self.status()),
            "models_download" => arg_key(args).and_then(|k| {
                if let Some(s) = lock(&self.slots).get_mut(&k) {
                    s.held = false;
                }
                lock(&self.hub).start(&k)?;
                to_value(&self.catalog())
            }),
            "models_prepare" => {
                let started = self.download_missing();
                for k in &started {
                    if let Some(s) = lock(&self.slots).get_mut(k) {
                        s.held = false;
                    }
                }
                to_value(&self.catalog()).map(|mut v| {
                    v["started"] = json!(started);
                    v
                })
            }
            "models_ensure" => arg_key(args).map(|k| self.ensure(&k, args.get("download").and_then(|v| v.as_bool()))),
            "models_cancel" => arg_key(args).and_then(|k| {
                lock(&self.hub).cancel(&k);
                to_value(&self.catalog())
            }),
            "models_load" => (|| {
                // 예전 화면 · CLI: {root} 를 주면 그 폴더로 바꾸고 모두 불러온다
                if let Some(root) = arg_str(args, "root").filter(|r| !r.trim().is_empty()) {
                    self.set_root(Path::new(root.trim()), true)?;
                    for k in MODEL_KEYS {
                        if let Some(d) = self.usable_dir(k) {
                            let _ = self.load_key(k, &d);
                        }
                    }
                    return to_value(&self.status());
                }
                let k = arg_key(args)?;
                if lock(&self.slots)[k.as_str()].loaded {
                    return to_value(&self.catalog());
                }
                let dir = self.usable_dir(&k).ok_or_else(|| format!("{k} 모델 파일이 없습니다 (먼저 다운로드)"))?;
                self.load_key(&k, &dir)?;
                to_value(&self.catalog())
            })(),
            "models_unload" => arg_key(args).and_then(|k| {
                self.unload_key(&k);
                to_value(&self.catalog())
            }),
            "models_delete" => arg_key(args).and_then(|k| {
                self.unload_key(&k);
                lock(&self.hub).delete(&k)?;
                to_value(&self.catalog())
            }),
            "models_set_root" => (|| {
                let root = arg_str(args, "root").ok_or("인자 'root' 가 없습니다")?;
                self.set_root(Path::new(root.trim()), true)?;
                if lock(&self.settings).models.startup_download {
                    self.download_missing();
                }
                to_value(&self.catalog())
            })(),
            "settings_get" => Ok(self.settings_view()),
            "settings_set_models" => (|| {
                let flag = |k: &str, d: bool| args.get(k).and_then(|v| v.as_bool()).unwrap_or(d);
                let endpoint = arg_str(args, "endpoint").map(|e| e.trim().trim_end_matches('/').to_string()).unwrap_or_default();
                if !endpoint.is_empty() && !(endpoint.starts_with("https://") || endpoint.starts_with("http://")) {
                    return Err("모델 서버 주소는 http:// 또는 https:// 로 시작해야 합니다".to_string());
                }
                let (ep, tk, turned_on) = {
                    let mut s = lock(&self.settings);
                    let m = &mut s.models;
                    let was = m.startup_download;
                    m.auto_download = flag("auto_download", m.auto_download);
                    m.startup_download = flag("startup_download", m.startup_download);
                    m.auto_load = flag("auto_load", m.auto_load);
                    m.endpoint = endpoint;
                    if flag("clear_token", false) {
                        m.token.clear();
                    } else if let Some(t) = arg_str(args, "token").map(|t| t.trim().to_string()).filter(|t| !t.is_empty()) {
                        m.token = t;
                    }
                    (m.endpoint.clone(), m.token.clone(), !was && m.startup_download)
                };
                self.save_settings()?;
                lock(&self.hub).configure(Some(&ep), Some(&tk));
                // '시작 시 받기'를 새로 켰으면 다음 시작까지 기다리지 않고 지금 받는다
                if turned_on {
                    self.download_missing();
                }
                Ok(self.settings_view())
            })(),
            _ => return None,
        };
        Some(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("crane_msvc_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn settings_default_to_downloading_on_start_and_persist() {
        let dir = temp("settings");
        let svc = ModelService::open(&dir);
        let s = svc.settings().models;
        assert!(s.startup_download && s.auto_download && s.auto_load);
        let v = svc.call("settings_set_models", &json!({"startup_download": false, "endpoint": "https://mirror.example/"})).unwrap().unwrap();
        assert_eq!(v["models"]["startup_download"], false);
        assert_eq!(v["models"]["endpoint"], "https://mirror.example");
        assert!(svc.call("settings_set_models", &json!({"endpoint": "ftp://x"})).unwrap().is_err());
        let again = ModelService::open(&dir);
        assert!(!again.settings().models.startup_download);
        assert_eq!(again.root(), dir.join("models"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn catalog_reports_missing_models_and_root_change() {
        let dir = temp("catalog");
        let svc = ModelService::open(&dir);
        // 시험에서는 내려받지 않는다 (폴더를 바꾸면 '시작 시 받기'가 없는 모델을 받기 시작한다)
        svc.call("settings_set_models", &json!({"startup_download": false, "auto_download": false})).unwrap().unwrap();
        let cat = svc.catalog();
        assert_eq!(cat.entries.len(), 2);
        assert!(cat.entries.iter().all(|e| !e.installed && !e.loaded && e.detected_dir.is_none()));
        // 손으로 넣어 둔 LAYA (Hugging Face 배치) 는 root 아래에서 찾는다
        let other = dir.join("elsewhere");
        let l = other.join("laya-typed-decisions");
        std::fs::create_dir_all(l.join("encoder")).unwrap();
        std::fs::write(l.join("encoder").join("config.json"), "{}").unwrap();
        std::fs::write(l.join("model.safetensors"), "").unwrap();
        svc.call("models_set_root", &json!({"root": other.display().to_string()})).unwrap().unwrap();
        let cat = svc.catalog();
        let laya = cat.entries.iter().find(|e| e.key == "laya").unwrap();
        assert!(laya.detected_dir.as_deref().unwrap().contains("laya-typed-decisions"));
        // 받는 도중인 자기 폴더(가중치가 아직 .part)는 쓰지 않는다
        let t = other.join("granite-timeseries-ttm-r3").join("52-16-dec-52-r3");
        std::fs::create_dir_all(&t).unwrap();
        std::fs::write(t.join("config.json"), "{}").unwrap();
        std::fs::write(t.join("model.safetensors.part"), "12").unwrap();
        let ttm = svc.catalog().entries.into_iter().find(|e| e.key == "ttm").unwrap();
        assert!(ttm.detected_dir.is_none() && ttm.partial_bytes == 2 && !ttm.installed);
        assert_eq!(ModelService::open(&dir).root(), other);
        assert!(svc.call("models_download", &json!({"key": "nope"})).unwrap().is_err());
        assert!(svc.call("not_mine", &json!({})).is_none());
        let st = svc.status();
        assert!(!st.ttm && !st.advisor && st.notes.len() == 3);
        // ensure: 자동 받기가 꺼져 있으면 없는 모델은 missing, 깨진 LAYA 는 로드를 시도하고 failed
        let e = svc.call("models_ensure", &json!({"key": "ttm"})).unwrap().unwrap();
        assert_eq!(e["state"], "missing");
        let e = svc.call("models_ensure", &json!({"key": "laya", "download": false})).unwrap().unwrap();
        assert_eq!(e["state"], "failed", "{e}");
        assert!(svc.catalog().entries.iter().any(|m| m.key == "laya" && m.load_error.is_some() && !m.loaded));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
