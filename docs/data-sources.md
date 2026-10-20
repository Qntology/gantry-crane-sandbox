# 실데이터 소스 (2026-10-19 조사, 10-20 보강)

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

## 3-1. 현장 운영 · 날씨 · 내구도 · 사고 예방 (2026-10-20 조사)

현장 프리셋 · 날씨 생성기 · 내구도 원장 · 알림 문턱 · 위험 배수 · 실데이터 보정에 쓴 자료다. 모두 조사 중에 열어 확인했다
(저작권 표준은 공개 미리보기 · 요약 · 2차 자료로 확인). 쓰인 곳과 오차는 [site-operation.md](site-operation.md).

### 물량 · 하중

| 자료 | 담긴 값 | 쓴 곳 |
| --- | --- | --- |
| [Productivity Commission 2022](https://assets.pc.gov.au/inquiries/completed/maritime-logistics/report/maritime-logistics-technical-paper.pdf) | STS 총 23–25 · 순 29–35 회/h | 컨테이너 물량 범위 |
| [Konecranes EAS RTG](https://www.konecranes.com/system/files/2025-03/7750_konecranes_eas_rtg_technical_document_v2.pdf) | RTG 기술 처리 능력 30 boxes/h | 처리량 보정 목표 |
| [ABB PT47 (2010)](https://library.e.abb.com/public/c5df00a36f92c0ab852577f2006ffc81/2010%20ABB%20Crane%20Systems%20PT47%20Clara%20Holmgren%20and%20Hans%20Cederqvist.pdf) | ASC 기술 능력의 70%, RTG 40% 미만 사용 | 부산 RMG 수요 |
| [Kalmar RMG data sheet](https://www.kalmarglobal.com/49c3b0/globalassets/media/11038/11038_RMG-Data-sheet-web-singles.pdf) · [Liebherr RTG](https://www-assets.liebherr.com/media/bu-media/lhbu-mcc/downloads-and-brochures/brochures/lcc/liebherr-rtg-cranes-technical-description.pdf) | SWL 41 t, 권상 35/70 · 주행 120 m/min, U7/Q2/A7 · M7 | `rmg_41t_30m` |
| [Bromma YSX40](https://bromma.com/products/ysx40-ysx45/) · [Eurostat 자중](https://www.era.europa.eu/era-railway-terminology-collection/tareweightofcontainer/term/eu_policy) | 스프레더 7.6 t, 자중 2.3 · 3.7 t | 훅 질량 · 빈 컨테이너 |
| [Interpave Heavy Duty Pavements](https://paving.org.uk/download/heavy-duty-pavements) · [Drewry (FreightWaves)](https://www.freightwaves.com/?p=352683) | 60:40 혼합 평균 16 t · P90 22 t, 빈 컨테이너 20.5% | 컨테이너 하중 스펙트럼 |
| [Hoist magazine 2023](https://www.hoistmagazine.com/analysis/running-the-mill-11049595/) · [G Steel](https://www.gsteel.com/en/our-product/coil-mass-dimensions) · [Konecranes 코일 크레인](https://www.konecranes.com/industries/automotive/proven-automotive-lifting-equipment/coil-handling-cranes-for-the-automotive-industry) | 하루 1,000 코일 · 크레인 3대 · 평균 28 t, 코일 7–28 t, M6–M8 | 코일 창고 프리셋 · 처리량 보정 |
| [Konecranes 조선소](https://www.konecranes.com/system/files/2024-04/shipyard_technical_information_2024_v1.pdf) · [KHL Rosyth](https://www.khl.com/news/giant-carrier-crane-up-for-sale/128710.article) · [Koh et al. 2011](https://koreascience.or.kr/article/JAKO201116450096186.pdf) | 700–1,200 t 급 속도 · A3/M5, 블록 약 100 t | 골리앗 프리셋 |
| [Hanson 중공 슬래브](https://www.specifiedby.com/hanson-building-products/hollowcore/Hanson_Hollowcore_Technical_DataSheet.pdf) · [PCI 12DT30](https://ems-www.pci.org/PCI_Docs/Design_Resources/Design_Tables/PCI-DH-2017/PretoppedDoubleTees_12DT30and12LDT30.pdf) · [BFT 2025](https://www.bft-international.com/en/artikel/handover-of-a-75-meter-gantry-crane-to-otto-quast-4297397.html) | 슬래브 2.3–7.2 t, 더블티 17–35 t, 40 t 갠트리 | 프리캐스트 프리셋 |
| [CMAA 등급 (MHI)](https://og.mhi.org/media/members/15194/133620919981313264.pdf) · [CMCO](https://prep.cmco.com/en-us/resources/blog/overhead-crane-classes-what-you-need-to-know) · [Konecranes CXT](https://www.konecranes.com/sites/default/files/download/cxt_uno_tech_specs_a4_en_konecranes_2017.pdf) | C 5–10 회/h, 하중 계수 L2, 속도 | 기계 공장 · 야적장 프리셋 |

### 날씨 · 환경

| 자료 | 담긴 값 | 쓴 곳 |
| --- | --- | --- |
| [기상청 평년값 (pgmNo=188)](https://data.kma.go.kr/climate/average30Years/selectAverage30YearsKoreaList.do?pgmNo=188) | 1991–2020 월 기온 · 최고 · 최저 · 강수량 · 풍속 · 습도 (부산 · 인천 · 울산 · 포항 · 여수 · 서산) | 기후 프리셋 |
| 기상청 일수 ([뇌전 699](https://data.kma.go.kr/climate/thdblt/selectThdbltChart.do?pgmNo=699) · [안개 706](https://data.kma.go.kr/climate/fog/selectFogChart.do?pgmNo=706) · 폭풍 708 · 눈 697 · 한파 733) | 지점별 연 일수 1991–2020 평균 · 월 분포 | 생성 날씨 · 보정 목표 |
| [기상청 순위값 (pgmNo=103)](https://data.kma.go.kr/climate/extremum/selectExtremumList.do?pgmNo=103) | 지점별 최대 순간풍속 상위 10개 (매미 부산 42.7 · 여수 49.2 m/s) | 강풍 사건 굼벨 꼬리 · 10년 순간풍속 |
| [기상청 특보 기준](https://www.weather.go.kr/kma/biz/forecast03.jsp) | 강풍경보 순간 26 m/s, 폭염 · 한파 특보 | 계류 문턱 |
| 위키백과 기후표 ([부산](https://en.wikipedia.org/wiki/Busan) 등, 기상청 평년값 인용) | 월 강수일수 (≥ 0.1 mm) | 기후 프리셋 |
| EN 13001-2 (비공식 사본, 구매 필요) · [CPA T_CIG 2507](https://cpa.uk.net/wp-content/uploads/2025/08/CPA-T_CIG-2507-Planning-Control-of-Lifting-Operations-in-Wind-V1-July-2025.pdf) | 운전 풍 상태 9.4/14 · 13.3/20 · 18.9/28 m/s, 돌풍 = 1.5 × 평균 | 돌풍 계수 · 운전 문턱 |
| [NTSB MAB-21/18](https://www.ntsb.gov/investigations/AccidentReports/Reports/MAB2118.pdf) · [Cal/OSHA §3472](https://www.dir.ca.gov/title8/3472.html) | STS 사전 경보 15.6 m/s, 2단 경보 절차 | 경고 문턱 |
| [Takahashi et al. 2016](https://www.jstage.jst.go.jp/article/mej/3/2/3_15-00679/_article) · [Zhu 2011 (KTH)](https://byv.kth.se/polopoly_fs/1.266908.1600689744!/Menu/general/column-content/attachment/Zhu_licentiate_thesis.pdf) | 젖은 · 노후 레일 정지 마찰 0.081, 점착 건조 0.25–0.30 · 젖음 0.18–0.20 · 이슬 0.09–0.15 · 눈 0.10 | 점착 계수 · 일주 검토 |
| [ABB The Motor Guide](https://content.voltimum.com/uk/133785/2010043092802010043086575376.pdf) | 주변 온도 · 고도 출력 배수표 | 모터 열 여유 보정 |
| [Demag DH 환경 사양](https://www.demagcranes.com/en/node/5361) | 표준 옥외 −10–40 °C | 한파 문턱 |
| [산업안전보건기준에 관한 규칙](https://www.law.go.kr/LSW/lsInfoP.do?lsiSeq=273603) | 제140조(30 m/s 이탈방지) · 제143조(강풍 뒤 점검) · 제559 · 560조(폭염 · 휴식) · 제163조(로프 안전계수 5) · 제38 · 40조 · 제8조(조도) | 알림 · 결정 · 안정성 여유 |
| [고용노동부 2026 폭염 대책 (기사)](https://v.daum.net/v/20260513162103312) | 35 °C 14–17시 · 38 °C 옥외 작업 중지 | 폭염 단계 |
| [USACE EM 385-1-1 (OSHAcademy)](https://www.oshacademy.com/courses/training/502-em385-1-1-safety-health/502-16-14.php) | 낙뢰 16 km 중지 · 30분 뒤 재개 | 낙뢰 문턱 |
| [Šmak et al. 2021](https://europepmc.org/article/PMC/PMC8467660) · [Alcántara et al. 2017](https://europepmc.org/article/PMC/PMC5506973) | ISO 9223 첫해 부식 범위, 장기 지수 내륙 0.49 · 해안 0.73 | 도장 · 거더 부식 |
| [MESYS (ISO 281 ηc)](https://www.mesys.ch/manual/contamination_factor.html) | 오염 계수 범주 | 분진 수명 배수 |

### 내구도 · 점검

| 자료 | 담긴 값 | 쓴 곳 |
| --- | --- | --- |
| [안전검사 고시 별표 2](https://www.law.go.kr/행정규칙/안전검사고시) | 라이닝 마모 50%, 디스크 10%, 플랜지 50%, 훅 국부 마모 5%, 과부하 1.1 배 | 폐기 기준 · 근거 |
| [산업안전보건법 시행규칙](https://www.law.go.kr/법령/산업안전보건법시행규칙) | 안전검사: 설치 후 3년 이내 최초, 그 뒤 2년 (건설현장 6개월) | 법정 검사 일정 |
| [Pintsch Bubenzer SB 28](https://dukebrakes.com/app/uploads/2014/07/SB-28-Operating-Manual.pdf) | 동적 시험이 없으면 2년마다 패드 교체 | 브레이크 보정 목표 |
| [UM Surabaya RTG 로프](https://repository.um-surabaya.ac.id/id/eprint/7453/) · [Chesterton CS062](https://chesterton.com/Files/Product%20Assets/Lubricants/EN350514_IL_MRO_CS062_Lubrication_Extends_Wire_Rope_Life.pdf) | RTG 로프 9개월, 윤활로 교체 6배 감소 | 로프 보정 목표 · 부식 배수 범위 |
| [LEESON (Reliable Plant)](https://www.reliableplant.com/Articles/Print/374) · [Autodesk L10h](https://help.autodesk.com/cloudhelp/2026/ENU/Inventor-Help/files/GUID-D48016B4-90DC-42B2-8D0E-5DA01A36713A.htm) | 10 K 규칙 · 20,000 h, 크레인 L10h 15,000–30,000 h | 모터 절연 · 베어링 |
| [ISO 12482 미리보기](https://cdn.standards.iteh.ai/samples/57481/7f6d045be4e1468cb407258f94d943c5/ISO-12482-2014.pdf) · [ISO 9927-1 미리보기](https://cdn.standards.iteh.ai/samples/51102/4fd7f5bfe0d141e58a2c32a02d751b90/ISO-9927-1-2013.pdf) | 설계 사용 기간 평가 · 점검 주기 | 정비 계획 문턱 · 정기 점검 |
| [Nippon Hoist 점검 기준](https://www.nipponhoist.co.jp/HP_main/HPe_download/HOIST_INSPECTION.pdf) · [Jotun (ISO 12944)](https://www.jotun.com/ww-en/industries/energy/power/about/selecting-the-right-system) | 답면 3%, 레일 머리 10%, 거더 부식 10%, 도장 내구성 L · M · H · VH | 차륜 · 거더 · 도장 |

### 사고 · 사람

| 자료 | 담긴 값 | 쓴 곳 |
| --- | --- | --- |
| [KOSHA 2014 천장크레인 끼임 연구](https://www.codil.or.kr/filebank/original/RK/OTKCRK180892/OTKCRK180892.pdf) | 2003–2012 동력 크레인 사망 954명(천장 346), 2012년 끼임 · 맞음 67.7%, 무선 조종 67.7%, 주변 작업자 50% | 출입 통제 배수 · 기준율 크기 |
| [BLS CFOI](https://www.bls.gov/iif/factsheets/fatal-occupational-injuries-cranes-2011-17.htm) · [CPWR](https://www.elcosh.org/record/document/1781/d000823.pdf) | 미국 크레인 사망 연 42명, 원인 분포 | 사고 종류 |
| [TT Club 야드 장비](https://www.ttclub.com/fileadmin/uploads/tt-club/Documents/files/tt-club/port-equipment-information-papers/recommended-minimum-safety-features-for-container-yard-equipment.pdf) · [TT Club 안벽 크레인](https://www.ttclub.com/media/files/tt-club/port-equipment-information-papers/recommended-minimum-safety-features-for-quay-container-cranes.pdf) | RTG/RMG 운전 22 m/s, 정지 제동 40 m/s, 바람 일주가 예방 가능한 최대 기상 손실 | RMG 제원 · 일주 검토 |
| [방호장치 인증고시](https://www.law.go.kr/LSW/admRulInfoP.do?admRulSeq=2100000199056) | 정격 1.1 배에서 경보 · 권상 정지 | 정격 초과 차단 |
| [Folkard, Lombardi & Tucker 2005](https://www.jstage.jst.go.jp/article/indhealth/43/1/43_1_20/_pdf) · [Dembe et al. 2005](https://stacks.cdc.gov/view/cdc/189921) · [IWH 2012](https://www.iwh.on.ca/node/274) | 야간 1.30 · 오후 1.18, 12 h 1.27, 연장 근무 1.61, 입사 첫 달 3배 이상 | 위험 배수 |
| [Korea JoongAng Daily (매미)](https://www.koreajoongangdaily.com/business/shoddy-work-suspected-in-fallen-port-cranes/11210735) | 부산 크레인 8대 전도 · 3대 손상, 설계 약 50 m/s | 폭풍 일주 사례 |
| [위험성평가 지침](https://www.law.go.kr/LSW/admRulInfoP.do?admRulSeq=2100000251014) | 위험성 3단계 판단법 | 위험 지수 단계 |

확인하지 못한 것: 국내 크레인 등록 대수 · 운전 시간(노출), 2013년 이후 국내 크레인 종류별 사망 통계, 부산항 크레인별 생산성,
크레인 시정 중지 기준, 해양 환경의 로프 수명 감소 정량값. 조사 중 웹 검색 한도에 닿아 더 찾지 못했다.
(KDS 지역별 기본 풍속, ISO 9224 지수, 항만 크레인 가용률 · 고장 빈도는 1.6단계 조사에서 찾았다 — 3-2 절.)

## 3-2. 염해 · 화물 · 시계열 예측 · 날씨 API (2026-10-20 1.6단계 조사)

전체 목록(값 · 인용 · 신뢰도 · 확인 못 한 것)은 [research/stage2](research/stage2/README.md) 의 R1–R5 메모에 있고, 쓰인 곳과 오차는
[marine-and-prognostics.md](marine-and-prognostics.md) · [site-operation.md 7절](site-operation.md#7-실데이터-대조--보정-결과-srcfieldcalrs).
아래는 모델 값에 직접 들어간 것만 추렸다.

### 날씨 API · 기후

| 자료 | 담긴 값 | 쓴 곳 |
| --- | --- | --- |
| [Open-Meteo 보관 API](https://open-meteo.com/en/docs/historical-weather-api) · [대기질 API (CAMS)](https://open-meteo.com/en/docs/air-quality-api) · [해양 API](https://open-meteo.com/en/docs/marine-weather-api) (CC BY 4.0, 키 없음) | ERA5 / ERA5-Land 시간별 기온 · 이슬점 · 습도 · 바람 · 돌풍 · 강수 · 눈, SO₂, 파고 | `weather_api` · `data/climate/ports.json` |
| 기상청 평년값 · 일수 · 순위값 (3-1 절) | 6개 지점 1991–2020 | ERA5 교차검증 (`era5cal`) |
| Alduchov & Eskridge 1996 (J. Appl. Meteor. 35) | Magnus 계수 17.625 · 243.04 °C | 시간 습도 · 이슬점 |
| Dai, Trenberth & Karl 1999 (J. Climate 12) | 구름 · 강수가 일교차를 25–50 % 줄임 | 비 오는 날 일교차 0.6 배 (가정 범위 안) |

### 염해 · 부식

| 자료 | 담긴 값 | 쓴 곳 |
| --- | --- | --- |
| ISO 9223:2012 용량–반응식 ([Paterlini 2024 표 1](https://www.ebi.ac.uk/europepmc/webservices/rest/PMC11679332/fullTextXML) · [Tidblad ISO WG4](https://www.rrcap.ait.ac.th/male/Meeting%20Document/14%20JT%20Classification.pdf)) | 탄소강 · 아연 식, 범주 C1–CX, S · P · τ 등급 | `marine::drf_*` |
| [ISO 9224:2012 미리보기](https://cdn.standards.iteh.ai/samples/53500/48d0ad5886f74a05a0d0faf4a71da09e/ISO-9224-2012.pdf) | b 탄소강 0.523 · 아연 0.813, 20년 뒤 직선 | 장기 손실 |
| [Alcántara et al. 2017](https://www.mdpi.com/1996-1944/10/4/406) | 해안 장기 지수 평균 0.73 (0.37–0.98), 바다 바람 3 m/s 문턱 | 고염분 b · 바다 바람 가중 |
| 이종석 외 2004 ([KCI 학술대회](https://www.koreascience.or.kr/article/CFKO200411722756623.page)) · 2006 ([PDF](https://koreascience.kr/article/CFKO200625522717202.pdf)) | 국내 6개 지점 거리 감쇠 b, 27개 해안 100 m 비래염분 (mdd) | `KR_DECAY` · `KR_SALT_100M` |
| [Li 2011 영종대교](https://koreascience.kr/article/JAKO201121161903206.pdf) | 상자 거더 안 · 밖 염화물 · SO₂ · 젖음 → 밀폐면 1/75 | 밀폐면 배수 |
| [Yan et al. 2020 (NIMS)](https://www.ebi.ac.uk/europepmc/webservices/rest/PMC7435615/fullTextXML) | 초시 · 미야코지마 노출 · 차폐 첫해 부식 | 차폐 배수 보정 · 벤치마크 |
| [Chico et al. 2017 (ISOCORRAG · ICP · MICAT)](https://www.ebi.ac.uk/europepmc/webservices/rest/PMC5553418/fullTextXML) · [Chen et al. 2026 (동중국해)](https://www.ebi.ac.uk/europepmc/webservices/rest/PMC13027690/fullTextXML) · Vera 2023 (라파누이) · Paterlini 2024 (보나솔라) | 실측 첫해 부식 속도 · 환경 | 부식 벤치마크 12곳 |
| [PennDOT 교량 세척 (FHWA-PA-2013-007)](https://www.pa.gov/content/dam/copapwp-pagov/en/penndot/documents/research-planning-innovation/research-projects/evaluation%20of%20bridge%20cleaning%20methods%20on%20steel%20structures%20-%20final%20report.pdf) | 담수 세척 염 제거 40–93 % | 세척 효율 0.75 |
| ISO 12944 ([Hempel 안내](https://www.hempel.com/fi-fi/-/media/Files/Local/EU-condensed/Brochures/FI/ISO-Booklet-FI.pdf) · [Cloverdale 계통표](https://www.cloverdalepaint.com/docs/default-source/industrial/cloverdale-paint-iso12944-2017-systems-guide-forc4-c5-and-cx-extreme-2021.pdf)) | 내구성 L · M · H · VH, C4 · C5 · CX 계통 | 도장 수명 · 방지 프리셋 |
| [GAA 첫 보수까지 수명표](https://gaa.com.au/wp-content/uploads/2026/08/Life-to-First-Maintenance-Chart-Australia-v2.pdf) · [AGA 이중 계통](https://www.galvanizeit.org/duplex-systems-gi-seminar/why-duplex/synergistic-effect/) | 85 µm C5 10–20년, 이중 계통 배수 1.5–2.3 | 통로 아연 · 이중 계통 · 벤치마크 |
| ISA-71.04-2013 ([LBNL 요약](https://datacenters.lbl.gov/sites/default/files/Combination%20Corrosion%20Coupon%20Testing%20Needed%20for%20Today's%20Control%20Equipment.pdf)) · IEEE 43 ([요약](https://wiki.testguy.net/t/field-testing-of-motors/249)) | G1–GX 반응 두께, 습도 규칙 · 절연저항 최소 5 MΩ | 전장 · 모터 |
| Cantley 1977 (ASLE Trans. 20) — R2 6a | 베어링 수명 (100/ppm)^0.6, 25–400 ppm | 감속기 수분 |

### 화물

| 자료 | 담긴 값 | 쓴 곳 |
| --- | --- | --- |
| [Interpave Heavy Duty Pavements](https://paving.org.uk/download/heavy-duty-pavements) · Hapag-Lloyd 컨테이너 사양 (R3 1c) | 적재 무게 분포, 자체 무게 2.3 · 3.9 t | 컨테이너 무게 · 빈 컨테이너 |
| [VICT Webb Dock VGM (FTA)](https://ftalliance.com.au/newsdetails/11919) | 수출 530개 중 ±1 t 넘게 다름 8.5 % | 신고 오차 · 재확인 벤치마크 |
| [KOSHA A-G-18-2026](https://portal.kosha.or.kr/openapi/v1/file/down/CTC2026012909544827454054/1) | 시험 인양 20 cm, 특수 20–30 cm, 위험물 직반출, 20 m/s 중지 | 시험 인양 · 위험물 |
| [Liebherr 강풍 지침](https://www.liebherr.com/en-at/mobile-and-crawler-cranes/customer-magazine/my-tip/safe-to-use-in-high-winds-5924163) (EN 13000 비례식) | 허용 풍속 ∝ √(1.2 / (A·c_D/m)) | 화물별 허용 풍속 |

### 통계 · 설계

| 자료 | 담긴 값 | 쓴 곳 |
| --- | --- | --- |
| KDS 41 12 00:2019 ([국가건설기준센터](https://www.kcsc.re.kr)) — R4 | 지역 기본풍속 (부산 38 · 포항 36 · 울산 · 여수 · 서산 34 · 인천 28 m/s) | 폭풍 설계 확인 |
| [Jo & Kim 2020 (JMSE 8, 6)](https://mdpi-res.com/d_attachment/jmse/jmse-08-00006/article_deploy/jmse-08-00006.pdf) | 부산 HKT 안벽 크레인 14기 가용률 95.0 %, 46 고장/년 · MTTR 0.40 h | 가용률 확인 |

### 시계열 열화 · 점수

| 자료 | 담긴 값 | 쓴 곳 |
| --- | --- | --- |
| [van Noortwijk 2009](https://repub.eur.nl/pub/7974) | 감마 과정 열화 모형 | 참 상태 |
| [Gneiting & Raftery 2007](https://sites.stat.washington.edu/raftery/Research/PDF/Gneiting2007jasa.pdf) | CRPS · 구간 점수 · 적정 점수 규칙 | `scoring` |
| [Saxena et al. 2008 (NASA)](https://ntrs.nasa.gov/api/citations/20100039169/downloads/20100039169.pdf) | α–λ 정확도 · 예측 지평 | `scoring` |
| Kuleshov, Fenner & Ermon 2018 (ICML, PMLR 80) | PIT 재보정 F' = G∘F | v4 |
| Brier 1950 · Murphy 1973 (R5) | Brier 점수 · 분해 | 30일 고장 확률 |
| ISO 4309 · ISO 10816/20816 · ISO 4628-3 (R2 · R5) | 로프 폐기 단선 수 · 진동 구역 · 녹 면적 Ri | 관측 지표 |

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
