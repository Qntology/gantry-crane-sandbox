# Gantry Crane Sandbox

갠트리 · 천장 크레인의 **하중 흔들림, 축 구동 모터, 브레이크 · 로프 마모, 운전 판정**을 한 시간축에서 시뮬레이션하는
Rust 샌드박스. [endmill-sandbox](https://github.com/sungu-che/endmill-sandbox) 의 파이프라인
(물리 해석 → 시계열 예측 → 결정론적 게이트 + LAYA 조언 → 학습 원장)을 크레인 도메인으로 옮겼다.

> **Spec · Move · Shape · Predict · Decide**

실기 없이 가상 시뮬레이션으로 시작하고, 실측 로그(흔들림 각 · 모터 전류 · 권선 온도 · 굴러가기 속도)가 생기면
보정 명령으로 연결한다. 실데이터 출처는 [docs/data-sources.md](docs/data-sources.md),
식 · 기본값 출처 · 검증은 [docs/physics-and-validation.md](docs/physics-and-validation.md),
**현장 운영 · 내구도 · 사고 예방 결정과 실데이터 보정 결과**는 [docs/site-operation.md](docs/site-operation.md),
포팅 기획은 [docs/plan.md](docs/plan.md) 에 있다.

## 무엇을 하나

| 단계 | 내용 |
| --- | --- |
| 본체 동역학 | 주행 · 횡행 · 권상(가변 로프 길이) · 편행 + 3D 구면 진자 하중. 로프 장력을 구속력으로 푸는 RK4 |
| 구동 | 축별 PI 속도 제어 + 앞먹임, 인버터 전류 한계(150% 60 s), 약계자 토크 한계, 경하중 고속 권상 |
| 모터 | 토크 · 전류 · 손실 · 단자 전력(회생), 권선–프레임 2절점 열 회로 (SEW S3 출력 배수표 재현) |
| 브레이크 · 로프 | 개폐 시퀀스 · 토크 기억 · 비상 정지 미끄럼 제동 마찰일 · 마모 · 온도, 로프 안전율 · Feyrer 굽힘 피로 |
| 흔들림 억제 | 사다리꼴 → 저크 평활 → ZV · ZVD · EI 입력 셰이핑, 각도 센서 LQR 궤환 |
| 운반 사이클 · 근무 | 경유점 사이클 시뮬레이션, 하중 스펙트럼으로 교대 근무(열 · 마모 · 설계 수명) 누적 |
| 시계열 예측 | 물리 사전값 + 잔차 예측 (TTM-R3 또는 Holt 감쇠 추세), 임계 도달 시점 |
| 판정 | 게이트 40여 개 → 규칙 조치 9단계, LAYA 조언은 상향 전용, 작업자 선택 카드 |
| 원장 · 저장 | SDS 원장(Welford + 링버퍼 드리프트, 보정 수축), SQLite 실행 기록 |
| 보정 · 교차검증 | 진자 · 굴러가기 · 권선 열 · Kaggle 모터 온도 · S3 표 보정, 질점 로프 · 이중 진자 해석기와 비교 |
| 현장 프리셋 | 물량 · 하중 스펙트럼 · 환경(부식성 · 분진 · 실내 열원 · 고도) · 기후 · 운영(교대 · 숙련 · 신호수 · 출입 통제 · 정비 · 법정 검사) 묶음. 내장 6곳(컨테이너 RMG · 코일 창고 · 프리캐스트 · 조선소 골리앗 · 기계 공장 · 야적장), 바꿔서 저장 |
| 날씨 | 기상청 1991–2020 평년값 6개 지점 → 시간별 날씨 (바람 · 돌풍 · 비 · 눈 · 결빙 · 뇌전 · 안개 · 체감온도) + 순위값으로 맞춘 태풍 · 강풍 사건 + 예보 오차 |
| 현장 운영 | 한 시간 단위로 며칠 · 몇 년: 날씨 → 알림 · 결정 → 처리량 · 정지 사유 · 부품 사용량 · 위험(사고율) · 폭풍 일주 |
| 내구도 원장 | 부품 16개(로프 · 브레이크 · 모터 · 감속기 · 차륜 · 거더 · 훅 · 도장 · 레일 클램프)의 수명 사용 · 남은 일 · 교체 · 안정성 여유를 자산별 파일로 계속 저장 |
| 사고 예방 결정 | 바람 · 태풍 · 낙뢰 · 시정 · 결빙 · 폭염 · 중량물 · 부품 · 점검 · 연장 근무 알림, Claude Code 식 결정 카드(권장 맨 앞 · 막힌 선택지 · 예상 손실 · 예상 사고) |
| 실데이터 보정 | 기상청 일수 · 순위값, ABB 출력 감소표, 야드 · 코일 크레인 처리량, 로프 · 브레이크 교체 주기와 비교해 계수를 맞춤 (맞춘 항목 평균 오차 28.7% → 3.4%) |

## 빠른 시작

```bash
# 테스트 (단위 103 + 시나리오 3, 약 15초)
cargo test
(cd crates/crane_mbd && cargo test --release)

# 데모: 흔들림 억제 방식 비교 → 사이클 → 8시간 근무 → 예측 → 판정 → 교차검증
cargo run --release --bin crane_demo -- demo

# 시나리오 하나 / 전체 파이프라인 (결과는 out/ 에 JSON · CSV)
cargo run --release --bin crane_demo -- simulate scenarios/gantry_10t_demo.json
cargo run --release --bin crane_demo -- pipeline scenarios/gantry_10t_demo.json --compare --sds out/sds --db out/runs.db

# 물리 엔진만 (candle · SQLite 없이)
cargo test --no-default-features --lib

# 현장 운영: 프리셋 → 90일 운전 (결정 카드마다 묻기) → 내구도 원장 (out/health/<자산>.json, 다시 돌리면 이어 씀)
cargo run --release --bin crane_demo -- site list
cargo run --release --bin crane_demo -- operate --site busan_newport_rmg --days 90 --start 07-01 --asset RMG-01 --policy ask
cargo run --release --bin crane_demo -- health show RMG-01
# 값을 바꿔 내 현장으로 저장
cargo run --release --bin crane_demo -- site save my_yard --from incheon_steel_stockyard --set throughput.moves_per_hour=10
# 실데이터 대조 · 보정 표 (약 3분, --quick 은 짧은 표본)
cargo run --release --bin crane_demo -- fieldcal
# 브라우저 화면 + API (http://localhost:8765)
cargo run --release --bin crane_demo -- serve
```

`crane_demo help` 에 모든 하위 명령이 있다. 리눅스는 `./test_linux.sh`, Windows 는 `test_windows.bat [cpu|vulkan|rocm]`.

### 화면

- **데스크톱 (Tauri v1)**: `npm install` 후 `run.bat` (CPU) · `run_vulkan.bat` · `run_rocm.bat`, 또는 `cd src-tauri && cargo tauri dev`.
- **브라우저 (실행 모드)**: `crane_demo serve` 가 화면과 API 를 함께 연다 (http://localhost:8765). 데스크톱 앱과 같은 명령을 쓴다.
- **브라우저 (파일 모드)**: `npm run dev` 로 `index.html` 을 열고 `crane_demo pipeline … --compare` 가 저장한
  `out/pipeline.json` · `out/compare.json`, `crane_demo operate` 의 `operation.json`, `crane_demo fieldcal` 의 `fieldcal.json` 을
  끌어다 놓는다. 3D 재생, 흔들림 · 모터 차트, 게이트, 근무 예측 띠, 방식 비교, 교차검증, **현장 운영 · 내구도**(알림 · 결정 기록 ·
  부품 수명 · 안정성 여유), **실데이터 보정** 탭이 있다. 현장 운영 칸에서 프리셋을 바꿔 저장하고, 결정 카드는 버튼 · 숫자 키로 고른다.

## 시나리오

`scenarios/` — 데모(갠트리 10 t, 천장 5 t), 바람 + 각도 궤환, 운반 중 비상 정지, 동시 이동, 과부하 11.5 t,
셰이퍼 설계 진동수 +15% 오차. `crane_demo scenario --crane overhead_5t_16m --load 3000 --mode ei` 로 새 시나리오를 만든다.
크레인은 프리셋 이름(`"gantry_10t_20m"`) 또는 전체 제원 JSON 으로 지정한다. 크레인 프리셋: `gantry_10t_20m` · `overhead_5t_16m`
(예시), `rmg_41t_30m` (컨테이너 RMG) · `overhead_40t_30m` (코일) · `gantry_40t_30m` (프리캐스트) · `goliath_900t_120m` (조선소).

## 학습 모델 (선택)

`ml` 기능(기본 켜짐)에서 endmill-sandbox 와 같은 candle 포크로 TTM-R3 · LAYA 를 불러온다. safetensors 폴더를 한 곳에 두고
`--models <폴더>` (CLI) 또는 화면의 모델 폴더에 지정한다. 경로에 `ttm` · `laya` 가 들어간 폴더를 찾는다
(예: `granite-timeseries-ttm-r3/52-16-dec-52-r3/`, `laya-typed-decisions/`). 모델이 없으면 Holt 예측과 규칙 판정만 쓴다.
GPU: `--features cuda | rocm | vulkan`, 장치 강제는 `CRANE_DEVICE=cpu|cuda|rocm|vulkan`.

## 실데이터로 보정하기

```bash
crane_demo calibrate pendulum  log.csv --channel theta_x            # 흔들림 각 → 진동수 · 감쇠비 · 등가 로프 길이
crane_demo calibrate coastdown log.csv --mass 31200 --channel vx    # 구름 저항 · 점성 계수
crane_demo calibrate winding   log.csv                              # 전류 · 속도 · 기준 온도 · 권선 온도 → 열 모델
crane_demo calibrate kaggle    measures_v2.csv --profile 4          # Paderborn Electric Motor Temperature (Kaggle)
crane_demo calibrate s3                                             # 모터 프리셋의 S3 출력 배수 vs SEW 표
crane_demo crosscheck --rope 6 --load 6000 --sling 2                # 정밀 해석기(이중 진자)와 비교
```

열 이름은 영어 · 한국어 별칭과 `_mm` · `_deg` 꼬리표를 알아본다 (`src/ingest.rs`). `data/sample/` 에 형식 확인용 합성 로그가 있다.
`scripts/fetch_open_data.sh` 는 공개 저장소(Crane3DSim, XJTU-SY)를 받고 Kaggle 명령을 안내한다.

## 구조

```text
src/
  crane_profile.rs   제원 · 프리셋 · 한계 · 설계 등급 (ISO 4301 / FEM 표)
  crane_dynamics.rs  본체 동역학 (구속 장력 RK4)
  motor.rs brake.rs rope.rs rope_data.rs environment.rs
  motion.rs antisway.rs   속도 프로파일 · 입력 셰이핑 · LQR
  cyclesim.rs duty.rs     운반 사이클 · 교대 근무
  timeseries.rs           예측 (물리 사전값 + 잔차, TTM-R3 / Holt)
  decision.rs choice.rs   게이트 · LAYA 조언 · 작업자 선택
  sds.rs store.rs         원장 · SQLite
  calib.rs ingest.rs      보정 · 로그 읽기 · 교차검증
  site.rs site_presets.rs 현장 프리셋 · 저장소 · 내장 현장 6곳
  weather.rs climate_data.rs rng.rs   기상청 평년값 · 시간별 날씨 생성 · 예보 · 체감온도
  operation.rs            현장 운영 시뮬레이션 (알림 · 결정 카드 · 처리량 · 위험 · 원장)
  health.rs               부품 내구도 · 안정성 원장
  alerts.rs field_constants.rs   알림 문턱 · 위험 모델 · 출처가 붙은 상수
  fieldcal.rs             실데이터 대조 · 보정 (data/reference/fieldcal_report.*)
  backend.rs              데스크톱 · `crane_demo serve` 공용 명령
  api.rs                  CLI · 데스크톱 공용 파이프라인
  ml/                     TTM-R3 · LAYA · ModernBERT (endmill-sandbox 이식)
  bin/main.rs             crane_demo
crates/crane_mbd/   질점 로프 · 훅 · 하중 다물체 해석기 (교차검증)
src-tauri/          데스크톱 셸 (Tauri v1)
index.html          화면 (단일 파일)
scenarios/ data/sample/ docs/ examples/ tests/ scripts/
```

## 한계

기본 크레인 제원은 예시다. 하중은 질점이고 거더 탄성 결합은 없다. Feyrer 식의 항 배치는 원서 확인이 필요하다.
현장 운영의 사고 기준율 · 일부 위험 배수 · 차륜 마모율 등은 가정이다 (docs/site-operation.md 의 가정 절).
TTM-R3 · LAYA 는 이식 후 빌드만 확인했다. 데스크톱 셸은 리눅스에서 타입 검사만 했다. 자세한 내용은
[docs/physics-and-validation.md](docs/physics-and-validation.md) 의 한계 절.

---

## English summary

A Rust sandbox for gantry/overhead cranes ported from endmill-sandbox: constrained-tension RK4 multibody model
(travel, trolley, variable-length hoist, skew, 3D spherical pendulum), inverter-fed motor electrical/thermal models
(reproduces SEW S3 derating within 0.07), brake sequencing/wear (mayr, INTORQ data), Feyrer rope fatigue, ZV/ZVD/EI
input shaping and LQR anti-sway, transport-cycle and shift-duty simulation, physics-prior time-series forecasting
(TTM-R3 or Holt), 40+ deterministic safety gates with an escalation-only LAYA advisor, an SDS drift ledger, SQLite run
store, calibration tools for real logs, and a lumped-mass rope solver for cross-checks. Site operation: presets that
bundle throughput, load spectrum, environment, KMA 1991–2020 climate and operating conditions; an hourly multi-day
simulator with accident-prevention alerts and Claude-Code-style decision cards; a persistent per-asset durability and
stability ledger for 16 components; and a field calibration against KMA statistics, ABB derating tables, crane
throughput and replacement-interval data (mean error on tuned items 28.7% → 3.4%). CLI: `crane_demo`; desktop:
Tauri v1 + single-file `index.html`; browser: `crane_demo serve`. Defaults that come from literature or standards are listed with sources in
`docs/physics-and-validation.md`; real-data sources are in `docs/data-sources.md`.

## License

Apache-2.0 (endmill-sandbox 와 같음).
