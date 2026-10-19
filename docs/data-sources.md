# 실데이터 소스 (2026-10-19 조사)

물리 엔진 모듈별로 보정 · 검증에 쓸 논문, 공개 데이터셋, 현장 데이터, 표준 · 카탈로그를 모은 목록이다.
모든 항목은 조사 중에 페이지를 직접 열어 확인했다. 수치는 각 출처에 적힌 값이다.
**공개된 크레인 전용 모터 데이터셋은 없었고**, 대형 STS · RTG 크레인의 로프 길이 포함 공개 흔들림 시계열도 찾지 못했다.

## 1. 모터 · 드라이브

| 자료 | 유형 | 담긴 값 | 접근 | 보정할 값 |
| --- | --- | --- | --- | --- |
| [Electric Motor Temperature](https://www.kaggle.com/datasets/wkirgsn/electric-motor-temperature) (Paderborn LEA) | 시험대 PMSM 시계열 | 185 h, 2 Hz, u_d · u_q · i_d · i_q · 속도 · 토크 · 냉각수 · 주변 온도 + 자석 · 권선 · 치 · 요크 온도, `profile_id` | CC BY-SA 4.0, Kaggle 로그인 | 열 회로 시간상수 · 열저항 (수냉 차량용 — 방법만 쓰고 값은 환산) |
| [thermal-nn](https://github.com/wkirgsn/thermal-nn) ([arXiv 2103.16323](https://arxiv.org/pdf/2103.16323)) | 코드 | 34 파라미터 LPTN 기준 모델, MSE 3.64 K² | MIT | 열 회로 구조 |
| [Nguyen Phuc 2020](https://www.mdpi.com/1996-1073/13/1/37) | 5.5 kW 인버터 구동 유도전동기 | 권선 PT1000, 2절점 LPTN, 간헐 부하 고정자 오차 0.92 K 이하 | CC BY 3.0 | 2절점 구조 검증 |
| [Gedzurs 2014](https://lbtufb.lbtu.lv/conference/Research-for-Rural-Development/2014/LatviaResearchRuralDevel20th_volume1-219-224.pdf) | 1.1 kW 모터 | 권선 시간상수: 팬 있음 약 8.7분, 없음 약 39분 | 공개 | `standstill_cooling` (0.22 반영) |
| [SEW 26859750](https://download.sew-eurodrive.com/download/pdf/26859750.pdf) | 제조사 자료 | S3 출력 배수: CDF 60% 1.1, 40% 1.15, 25% 1.3, 15% 1.4 | 공개 | 권선 시간상수 (반영, `examples/s3_fit.rs`) |
| [Innomotics CR 81](https://cache.industry.siemens.com/dl/files/976/109760976/att_1275803/v1/Motors-CR-81-complete-English-04-2024.pdf) | 크레인 모터 카탈로그 | 132S 4극 S3-25%: 8.3 kW, 55 Nm, 17 A, cosφ 0.85, J 0.022 kg·m². S3-40% 7.5 kW. KFB 63 브레이크 630 Nm | 공개 | 모터 · 브레이크 프리셋 |
| [Paderborn KAt](https://mb.uni-paderborn.de/en/kat/research/bearing-datacenter/data-sets-and-download) | 시험대 (425 W PMSM, 인버터) | 상전류 2개 + 진동 64 kHz, 베어링 32종 (실손상 14) | CC BY-NC 4.0 | 전류 기반 베어링 이상 탐지 |
| [LIMAN-C](https://data.mendeley.com/datasets/kccmrf3864) | 유도전동기 3상 전류 | 480건, 회전자 바 · 베어링 · 턴 단락, 부하 0–100% | CC BY 4.0 | 고장 측대역 vs 부하 |
| [XJTU-SY](https://github.com/WangBiaoXJTU/xjtu-sy-bearing-datasets) · [NASA IMS](https://data.nasa.gov/dataset/ims-bearings) | 수명 끝까지 진동 | XJTU-SY 15개, 하중 10–12 kN, 수명 41–2,537분 | 공개 | 베어링 열화 곡선 예측 |
| [Felixstowe RTG](https://e-space.mmu.ac.uk/620719/) (Harrison 2018) | 현장 RTG 로거 | 37채널 10 Hz: 3상 V · I · P, DC 버스, 권상 · 횡행 · 주행 · 제동 초퍼 전류, 하중, 1년 62.6 GB | 저자에게 요청 | 축별 전력 · 회생 · 듀티 |
| [Pietrosanti 2016](https://e-space.mmu.ac.uk/620864/8/IEEE_IATRAN.pdf) | 현장 RTG 권상 | 200 kW 8극, 감속비 122.8, 드럼 1.285 m, 약 50 m/min | 공개 원고 | 권상 손실 맵 |
| [Papaioannou 2017](https://e-space.mmu.ac.uk/620872/9/Manuscript.word.pdf) | 현장 RTG | 평균 리프트 22.1 s, 리프트 간격 83 s, 회수 가능 권상 에너지 84–89% | 공개 원고 | 회생 비율 검증 |
| [FEM 듀티 표](https://www.aceindustries.com/fem-duty-information/) | 기구 등급 | M3–M8 최대 기동 150 · 180 · 240 · 300 · 360 · 360 회/h, 사이클 25 · 30 · 40 · 50 · 60 · 60 회/h, 1Bm · 1Am · 2m · 3m 운전율 25 · 30 · 40 · 50% | 공개 | duty 게이트 (반영) |
| [Yaskawa CR700](https://yaskawavn.com/data/upload/CR700Catalogue_YAP-KAEPC710617-24Arev_May2020.pdf) | 크레인 인버터 | 과부하 150% 60 s | 공개 | 인버터 기본값 (반영) |
| [Eaton DA1](https://es-assets.eaton.com/DRIVES/POWERXL/01_APPLICATION_NOTE/English/DA1/AP040032EN_DA1_Hoist_applications.pdf) · [ABB ACS880](https://library.e.abb.com/public/8f301763970840c5a0985d05fb586d34/EN_ACS880_crane_ctrl_prg_FW_F.pdf) | 권상 적용 노트 | 브레이크 해제 지연 0.2 s · 체결 지연 0.5 s (Eaton 기본), 토크 증명 | 공개 | 브레이크 시퀀스 |
| [Zając 2025](https://www.mdpi.com/1996-1073/18/24/6550) | 천장 크레인 실측 | 20 kW 구동, 이동당 90–190 Wh | 공개 | 에너지 검증 목표 |

## 2. 흔들림 · 동역학 · 구조

| 자료 | 유형 | 담긴 값 | 접근 | 보정할 값 |
| --- | --- | --- | --- | --- |
| [Driving Smart Crane With Various Loads](https://ieee-dataport.org/documents/driving-smart-crane-various-loads) | 산업용 천장 크레인 로그 (OPC UA CSV 5.6 MB) | 주행 · 횡행 · 권상 위치 · 속도 · 모터 토크, 로프 각, 하중 0 / 120 / 500 / 1000 kg, 흔들림 억제 켬/끔 | 구독 | 진동수–로프 길이, 마찰, 구동 응답 전체 |
| [Crane3DSim](https://github.com/jrgvicente/Crane3DSim) + [arXiv 2509.13330](https://arxiv.org/html/2509.13330v1) | 실험실 INTECO 3D 크레인 식별 모델 | 위치 의존 쿨롱 + 점성 마찰, 흔들림 감쇠 Dα 2.57e-4 · Dβ 0.0059, Ts 2 ms, 로프 0.13–0.57 m | GPL-3.0 | 마찰 · 흔들림 감쇠 |
| [Mendeley 타워 크레인 궤적](https://data.mendeley.com/datasets/x69hyyv3zw/2) | 실험실 시계열 | 33 시나리오 × 99 s, 100 Hz, 권상 포함, 케이블 0.1–0.8 m | 공개 (라이선스 칸 비어 있음) | 가변 로프 길이 항 |
| [Maghsoudi 2017](https://openresearch.lsbu.ac.uk/download/eab1b01392dc68ecacaead090597e4ccd24eb65d55e740c9faf828af5dea0c72/647163/Maghsoudi%20et%20al_TIMC_2017_39%283%29_361-370.pdf) | 실험실 3D 갠트리 | 하중 1 kg, 트롤리 1.155 kg, 케이블 0.47 m, ζ 0.008, 셰이핑 잔류 4° → 0.8° | 공개 논문 | 셰이퍼 검증 |
| [Manning 2008](https://repository.gatech.edu/server/api/core/bitstreams/9d7cdf7b-378b-403d-8607-d33beba02be9/content) | 10 t 브리지 크레인 실측 | 이중 진자 모드 0.23 Hz · 0.53 Hz, 훅 약 50 kg, 하중 약 53 kg, 걸이 3.5 m | 공개 | `crane_mbd` 이중 진자 |
| [Sorensen 2008](https://skoge.folk.ntnu.no/prost/proceedings/ifac2008/data/papers/3834.pdf) | 30 t 산업용 크레인 | 훅 감쇠비 약 0.01, 케이블 3–10 m, 흔들림 약 90% 감소 | 공개 논문 | `sway_damping_ratio` (반영) |
| [Quay crane sway speed](https://ieee-dataport.org/open-access/quay-crane-spreader-sway-speed-x-axis-10-use-cases) | 현장 STS (Klaipeda) | 하역 10건 X축 흔들림 속도 | 무료 로그인 | 현장 흔들림 크기 · 주기 |
| [Kim, Hong, Sul 2004](https://cogno.pusan.ac.kr/sites/cogno/download/eng/1_56_IJCAS_v.2_n.4_pp.435-449_2004_Kim-Hong-Sul%20(NRL).pdf) | 현장 RTG | 로프 8–20 m, 헤드블록 + 스프레더 12.2 t, 하중 17.2–42.2 t | 공개 논문 | 대형 크레인 프리셋 범위 |
| [Arena 2016 원고](https://strathprints.strath.ac.uk/57168/1/Arena_etal_PIMECJMES2016_Nonlinear_interactions_in_deformable_container_cranes.pdf) | 실물 STS 모델 | 로프 E 9e10 Pa, 붐 1차 약 0.9 Hz, 높이 5–8 m에서 붐–흔들림 2:1 공진 | 공개 원고 | 로프 EA, resonance 게이트 |
| [Uzunov 2010](https://acta.fih.upt.ro/pdf/2010-1/ACTA-2010-1-20.pdf) | 500 kg 로프 호이스트 실측 | 로프 강성 5.296e5 N/m (L 4.535 m), 대수 감쇠율 0.359 | 공개 | `crane_mbd` 로프 감쇠 (반영) |
| [Kohama 2012](https://wcee.nicee.org/wcee/article/WCEE2012_2244.pdf) | 실물 컨테이너 크레인 시험 | 횡방향 구조 감쇠 0.2–0.4% (설계값 2.5%) | 공개 | 구조 감쇠 |
| [Haniszewski 2014](https://delibra.bg.polsl.pl//Content/27245/BCPS_30712_-_Hybrid-analysis-of-v_0000.pdf) | 5 t 단일 거더, 스팬 20 m | 측정 고유진동수 3.8 Hz (거더 끝), 7.5 Hz (권상 방향) | 공개 | `girder_fn_hz` |
| [EN 13001-2:2021 미리보기](https://standards.iteh.ai/catalog/standards/cen/eb7618bc-a831-474b-b2e5-5aa985036ee9/en-13001-2-2021) | 표준 | 운전 풍 '보통': 평균 13.3 m/s, 3초 돌풍 20 m/s, 250 N/m². 매단 하중 c = 2.4, A = 0.0005·m (≥ 0.8 m²) | 유료 | 바람 한계 · 하중 항력 (반영) |

UCI 'Container Crane Controller' 는 퍼지 규칙 15행뿐이라 보정에 쓸 수 없다.

## 3. 로프 · 브레이크 · 하중 스펙트럼 · 구조 피로

| 자료 | 유형 | 담긴 값 | 접근 | 보정할 값 |
| --- | --- | --- | --- | --- |
| [Feyrer 갱신 자료](https://www.ift.uni-stuttgart.de/forschung/img/feyrer/Update-Rope.pdf) ([독일어판](https://www.ift.uni-stuttgart.de/forschung/img/feyrer/Update-Seil2.pdf)) | 로프 굽힘 시험 상수 | 표 3.14: 구조 · 꼬임별 a0–a3 (파단 N, 폐기 N_A). 유효 범위 d ≥ 6 mm, 굽힘 길이 ≥ 10d | 공개 (OCR 품질 낮음) | `rope.fatigue` (반영) |
| [Ryan 승강기 논문](https://liftescalatorlibrary.org/paper_indexing/papers/00000096.pdf) | 교차 확인 | 표 3.14b 폐기 상수 FC 8.567 / −2.894, IWRC 8.056 / −2.577 | 공개 | 상수 검증 |
| [Battelle 1974](https://apps.dtic.mil/sti/pdfs/ADA001673.pdf) | 굽힘 피로 시험 | 로프 288개, 44–487,580 사이클, D/d 10 · 20 · 30, 하중 20 · 40 · 60% | 공개 | 로프 수명 모델 보정 |
| [Freddi 2023](https://www.mas.bg.ac.rs/_media/istrazivanje/fme/vol51/1/7_l._solazzi_et_al.pdf) | 굽힘 시험 | ISO 4309 폐기까지: 6xK31WS-IWRC D/d 12 · 16 · 20 에서 7,360 · 23,140 · 108,950 | 저작권 있음 | 단선 증가 |
| [ISO 16625:2013 미리보기](https://iteh.es/catalog/standards/iso/6512450b-c7b7-48c4-be43-0552cdb7579f/iso-16625-2013) | 표준 | Zp M1–M8: 3.15 · 3.35 · 3.55 · 4.0 · 4.5 · 5.6 · 7.1 · 9.0 | 미리보기 공개 | 최소 안전율 (반영) |
| [ISO 4309:2017](https://kscranegroup.com/wp-content/uploads/2024/09/ISO-04309-2017.pdf) (제3자 사본) | 표준 | 단선 폐기 기준, 직경 감소 폐기: 섬유심 10%, 강심 7.5%, 비자전 5% | 구매 | 로프 점검 |
| [mayr ROBA-stop-S](https://pe.urany.net/assets/img/k_856_v18_en_16_12_2016.pdf) | 브레이크 카탈로그 | 100–800 Nm: 마모 0.1 mm당 44–95 MJ, 회전자 마모까지 308–1,900 MJ | 공개 (제3자 미러) | 라이닝 마모 (반영) |
| [INTORQ BFK458](https://kendrion.com/fileadmin/user_upload/Downloads/Datasheets_Operating_instructions/Industrial_Brakes_INTORQ/operating-instructions-spring-applied-brake-BFK458.pdf) | 브레이크 지침서 | Q_perm = Q_E·(1 − e^(−S_hü/S_h)), Q_E 3–120 kJ, 표면 130 °C | 공개 | 1회 허용 마찰일 · 온도 (반영) |
| [Konecranes 브레이크 모니터링](https://www.konecranes.com/sites/default/files/2025-07/broschure_truconnect_brake_monitoring_2024_deat.pdf) | 원격 감시 기준 | 공극 여유 30% 미만 정비, 15% 미만 점검 | 공개 | 브레이크 게이트 (반영) |
| [Ungureanu 2020](https://czasopisma.pan.pl//Content/115957/PDF/Archiwum-65-1-03-Ungureanu.pdf) | 광산 권상 브레이크 실측 | 열의 약 95% 가 드럼으로, 상용 제동 28–38 °C, 비상 41–105 °C | CC BY-NC | 브레이크 열 모델 |
| [ISO 4301-1:2016](https://dgcrane.com/wp-content/uploads/2023/09/ISO-04301-1-2016.pdf) · [ISO 12482:2014](https://dgcrane.com/wp-content/uploads/2023/09/ISO-12482-2014.pdf) (제3자 사본) | 표준 | U0–U9, Kp 등급, M1–M8 = 100 … 12,500 전부하 시간, Q1–Q4 Kp 0.125 · 0.25 · 0.5 · 1.0, 기록 계수 f1 1.0–1.5 | 구매 | 설계 수명 (반영) |
| [FEM 9.512](https://www.fem-eur.com/wp-content/uploads/2016/03/FEM-9_512_Englisch.pdf) · [FEM 9.755 기록 예](https://cmco.com/globalassets/wrh-diagrams/1certificates/a11868201.pdf) | 기구 등급 | 3m · 4m · 5m = 3,200 · 6,300 · 12,500 전부하 시간, 1Bm 400 h | 공개 | M 등급 표 (반영) |
| [Demag StatusControl](https://statuscontrol.demagcranes.com/help/en/dealer/A$X983043.html) | 원격 감시 기준 | 남은 설계 수명 30% · 10% 경보, 10% 이상 과부하 차단 | 공개 | SWP · 과부하 게이트 (반영) |
| [JNIOSH SRR-No.15-3](https://www.jniosh.johas.go.jp/publication/doc/srr/SRR-No15-3.pdf) | 49 kN, 스팬 16.2 m 크레인 실측 | 거더 응력 대부분 20 MPa 미만, 최고 약 30 MPa, 하강 동하중 계수 1.52 | 공개 (일본어) | 구조 피로 (다음 단계) |
| [EN 13001-3-1 요약](https://sdcverifier.com/articles/standards/en13001-standard) | 표준 요약 | 용접 상세 등급 Δσc 180 · 140 · 80 (m = 3) 등 | 공개 | 구조 피로 (다음 단계) |

Feyrer 식의 항 배치는 OCR 이 깨진 갱신 자료에서 재구성한 것이라 Feyrer 2015 원서로 확인해야 한다.
공개 자기 누설(MFL) 로프 신호 데이터셋은 없다.

## 4. 우선 확보 순서

| 순위 | 자료 | 검증 · 보정할 모듈 | 받는 방법 |
| --- | --- | --- | --- |
| 1 | Aalto Driving Smart Crane (IEEE DataPort) | 동역학 · 구동 · 흔들림 억제 전체 | 구독, 샘플링 주기는 저자에게 문의 |
| 2 | Electric Motor Temperature + thermal-nn | 모터 열 회로 보정 절차 | Kaggle 로그인 |
| 3 | Crane3DSim + arXiv 2509.13330 | 주행 · 횡행 마찰, 흔들림 감쇠 식별 | GitHub |
| 4 | Feyrer 갱신 자료 + Feyrer 2015 원서 | 로프 굽힘 피로 상수, 식 확인 | 갱신 자료 공개, 원서 구매 |
| 5 | Manning 2008 · Sorensen 2008 | 훅–하중 이중 진자 · 감쇠 | 공개 |
| 6 | ISO 4301-1 · ISO 12482 | 설계 수명(SWP) 회계 | 구매 |
| 7 | Felixstowe RTG 10 Hz 로그 | 축별 전력 · 회생 · 듀티 | MMU 저자에게 요청 |
| 8 | Paderborn KAt · LIMAN-C | 전류 기반 고장 탐지 학습 (2단계) | 공개 |
| 9 | EN 13001-2:2021 전문 | 폭풍(비운전) 풍속 조항 | 구매 |

## 5. 이 레포에서 쓰는 방법

| 데이터 | 명령 · 위치 |
| --- | --- |
| Kaggle `measures_v2.csv` | `crane_demo calibrate kaggle measures_v2.csv --profile 4 --ref coolant` → 2단 권선 열 모델 (시간상수 · 이득 · RMS 오차) |
| 권선 온도가 있는 모터 로그 | `crane_demo calibrate winding log.csv` (열: `current_a`, `speed_rpm`, `ambient_c` 또는 `coolant_c`, `winding_c`) |
| 흔들림 각 로그 (Aalto, Mendeley 등) | `crane_demo calibrate pendulum log.csv --channel theta_x` → 진동수 · 감쇠비 · 등가 로프 길이. 열 이름 별칭은 `src/ingest.rs` |
| 구동 없이 굴러가는 구간 | `crane_demo calibrate coastdown log.csv --mass 31200 --channel vx` → 구름 저항 · 점성 계수 |
| S3 출력 배수표 (다른 제조사) | `examples/s3_fit.rs` 의 목표값을 바꿔 `cargo run --release --example s3_fit` |
| Feyrer 상수 | 시나리오 JSON 의 `crane.rope.fatigue` 에 `{"model": "feyrer_update", "a0": …, "a1": …, "a2": …, "a3": …, "source": "…"}` |
| 브레이크 카탈로그 값 | 시나리오 JSON 의 `crane.<axis>.brake` (`work_per_0p1mm_mj`, `q_e_j`, `s_transition_per_h` …) |

`data/sample/` 의 CSV 는 형식 확인용 **합성 데이터**다 (`cargo run --release --example make_samples`).
