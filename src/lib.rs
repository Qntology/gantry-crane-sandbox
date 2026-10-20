//! Gantry Crane Sandbox — 갠트리 크레인 물리 · 구동 · 흔들림 억제 · 마모 · 판정 샌드박스.
//!
//! endmill-sandbox 의 파이프라인(물리 해석 → 시계열 예측 → 결정론적 게이트 + 조언 모델 → 원장)을
//! 크레인 도메인으로 옮긴 것이다.

pub mod alerts;
pub mod antisway;
pub mod api;
pub mod backend;
pub mod brake;
pub mod calib;
pub mod cargo;
pub mod choice;
pub mod climate_data;
pub mod crane_dynamics;
pub mod crane_profile;
pub mod cyclesim;
pub mod decision;
pub mod degradation;
pub mod duty;
pub mod environment;
pub mod era5cal;
pub mod field_constants;
pub mod fieldcal;
pub mod health;
pub mod ingest;
pub mod marine;
pub mod model_service;
pub mod modelhub;
pub mod motion;
pub mod motor;
pub mod operation;
pub mod ports;
pub mod prognostics;
pub mod rng;
pub mod rope;
pub mod rope_data;
pub mod scoring;
pub mod sds;
pub mod site;
pub mod site_presets;
#[cfg(feature = "store")]
pub mod store;
pub mod timeseries;
pub mod weather;
pub mod weather_api;

#[cfg(feature = "ml")]
pub mod ml;
