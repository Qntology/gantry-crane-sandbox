# 포팅 기획 요약

[endmill-sandbox](https://github.com/sungu-che/endmill-sandbox) 의 파이프라인
**해석식 물리 → 경로 시뮬레이션 → 시계열 예측 → 게이트 판정 + 조언 모델 → 보정 출력 → 학습 원장** 을
갠트리 크레인으로 옮겼다. 흐름은 그대로 두고 각 단계의 도메인 계산만 바꿨다.

## 개념 대응

| 엔드밀 | 갠트리 크레인 |
| --- | --- |
| 공구 마모(VB) 누적 | 로프 굽힘 피로 · 브레이크 라이닝 마모 · 설계 수명(SWP) 누적 |
| 채터(재생 진동) | 하중 흔들림(진자), 하중 상하 진동 vs 거더 고유진동수 |
| 스핀들 동력 · 토크 한계 | 축별 모터 토크 · 전류 · 권선 온도 · 운전율 한계 |
| 공구경로 패턴 | 경유점 운반 사이클 (순차 · 동시 이동) |
| 이송 오버라이드 G-code | 축별 속도 프로파일 (`MoveProgram`, 셰이퍼 포함) |
| 코팅 × 소재 × 냉각 행렬 | 하중 스펙트럼 × 로프 길이 × 바람 × 듀티 |
| MPM 절삭 해석기 교차검증 | 질점 로프 · 이중 진자 해석기(`crane_mbd`) 교차검증 |

## 모듈 매핑

| 엔드밀 모듈 | 처리 | 크레인 모듈 |
| --- | --- | --- |
| `timeseries.rs`, `ml/ttm.rs` | 이식 (모델을 `SeriesModel` 트레이트로 일반화) | `timeseries.rs`, `ml/ttm.rs` |
| `ml/laya.rs`, `ml/modernbert.rs` | 그대로 이식 | 같은 이름, `decision::Advisor` 구현 |
| `choice.rs` | 이식 | `choice.rs` |
| `decision.rs` | 구조 유지, 게이트 · 조치 교체 | `decision.rs` (게이트 40여 개, 조치 9단계) |
| `sds.rs` | 축약 이식, 원장 교체 | `sds.rs` (모터 · 브레이크 · 로프 · 흔들림 · 구조 · 결정) |
| `store/` (SQLite + LanceDB) | SQLite 만 | `store.rs` (LanceDB 는 다음 단계) |
| `profile.rs` | 교체 | `crane_profile.rs` |
| `physics.rs`, `tribology.rs`, `cutting.rs` | 대체 | `crane_dynamics.rs`, `motor.rs`, `brake.rs`, `rope.rs` |
| `loadsim.rs` | 대체 | `cyclesim.rs`, `duty.rs` |
| `gcode.rs` | 대체 | `motion.rs` + `MoveProgram` |
| `crates/endmill_mpm`, `physcal.rs` | 대체 | `crates/crane_mbd`, `calib.rs` |
| (없음) | 신규 | `antisway.rs` (ZV · ZVD · EI · LQR), `environment.rs`, `ingest.rs`, `api.rs` |
| `src-tauri`, `index.html` | 다시 작성 | 같은 구조 (Tauri v1) |

## 결정 계층

하드 인터록(정격 하중 제한 · 리밋 · STO/SBC)은 실제로는 드라이브와 안전 PLC 의 몫이다. 샌드박스는 같은 기준으로
판정해 보여 줄 뿐이며, 흔들림 억제 · 예측 · LAYA 조언은 그 위의 조언 계층이다. LAYA 는 규칙보다 보수적인 쪽으로만,
'점검' 단계까지만 올릴 수 있고, 작업자는 규칙 판단보다 덜 보수적인 조치를 고를 수 없다.

## 단계

| 단계 | 내용 | 다음 단계로 가는 조건 |
| --- | --- | --- |
| 1 (이 레포) | 물리 · 구동 · 흔들림 억제 · 사이클 · 근무 · 예측 · 게이트 · 원장 · 보정 · 정밀 해석기 · CLI · 데스크톱 셸 | 자동 테스트 통과, 합성 데이터에서 보정 도구가 파라미터 회복 |
| 2 실데이터 보정 | Aalto 크레인 로그로 흔들림 진동수 · 축 토크 검증, Kaggle 모터 온도로 열 회로, Crane3DSim 으로 마찰 · 감쇠, Feyrer 원서로 로프 상수 | 흔들림 진동수 · 토크가 실측과 ±5% 안, 권선 온도 RMS 3 K 이하 |
| 3 학습 모델 | TTM-R3 가중치로 실측 잔차 예측, TSPulse 이상 탐지, LAYA 질문 조정, LanceDB 사이클 서명 검색 | 예측이 Holt 기준선보다 나음 |
| 4 제어 · 구조 | MPC 흔들림 억제, 권상 중 시변 셰이퍼, 거더 탄성 결합, 구조 피로(EN 13001-3-1) | — |
