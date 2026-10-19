//! Gantry Crane Sandbox — 갠트리 크레인 물리 · 구동 · 흔들림 억제 · 마모 · 판정 샌드박스.
//!
//! endmill-sandbox 의 파이프라인(물리 해석 → 시계열 예측 → 결정론적 게이트 + 조언 모델 → 원장)을
//! 크레인 도메인으로 옮긴 것이다.

pub mod antisway;
pub mod api;
pub mod brake;
pub mod calib;
pub mod choice;
pub mod crane_dynamics;
pub mod crane_profile;
pub mod cyclesim;
pub mod decision;
pub mod duty;
pub mod environment;
pub mod ingest;
pub mod motion;
pub mod motor;
pub mod rope;
pub mod rope_data;
pub mod sds;
#[cfg(feature = "store")]
pub mod store;
pub mod timeseries;

#[cfg(feature = "ml")]
pub mod ml;
