# R3 — Cargo scenario presets: sourced numbers

Status: COMPLETE (2026-10-20; v1 + continuation filling Q1 empties/reefer/OOG, Q2 TEU/T-S and productivity/downtime comparators, Q4 precast weights, Q6 corroboration, Q9 coil/project/tandem times, Unverified list). Each value carries source URL + short quote/table ref + confidence.
Confidence: high = primary source/standard/statute opened; med = secondary but opened; low = indirect/derived.

---------------------------------------------------------------------------------------------------

## Q1 Container weights, VGM, reefer/OOG share

### 1a. Gross-weight histogram (loaded mix) — Interpave Table 16 (UK ports, 1970s data)
Source: J. Knapton, *The Structural Design of Heavy Duty Pavements for Ports and Other Industries*, Ed. 4, Interpave 2007,
§8.4 Table 16, p.36. URL (PDF): https://paving.org.uk/download/heavy-duty-pavements
Quote (§8.4): "Table 16 shows the distribution of container weights normally encountered in UK ports for different proportions of 20ft and 40ft containers."
Caveat quote (Table 16 note): "these figures were derived during the 1970s. There is no evidence to suggest that they are inaccurate".
Confidence: high (table opened) for the table itself; med as a proxy for a modern Busan mix.

% of containers per 1,000 kg bin (bin label = container gross weight, kg):

| Gross kg | 100/0 (all 40ft) | 60/40 | 50/50 | 40/60 | 0/100 (all 20ft) |
|---:|---:|---:|---:|---:|---:|
| 2,000 | 0.00 | 0.18 | 0.23 | 0.28 | 0.46 |
| 3,000 | 0.00 | 0.60 | 0.74 | 0.89 | 1.49 |
| 4,000 | 0.18 | 1.29 | 1.57 | 1.84 | 2.95 |
| 5,000 | 0.53 | 1.90 | 2.25 | 2.59 | 3.96 |
| 6,000 | 0.98 | 2.17 | 2.46 | 2.76 | 3.94 |
| 7,000 | 1.37 | 2.41 | 2.67 | 2.93 | 3.97 |
| 8,000 | 2.60 | 3.05 | 3.16 | 3.27 | 3.72 |
| 9,000 | 2.82 | 3.05 | 3.11 | 3.17 | 3.41 |
| 10,000 | 3.30 | 3.44 | 3.48 | 3.52 | 3.66 |
| 11,000 | 4.43 | 4.28 | 4.24 | 4.20 | 4.04 |
| 12,000 | 5.73 | 5.24 | 5.12 | 4.99 | 4.50 |
| 13,000 | 5.12 | 4.83 | 4.76 | 4.69 | 4.41 |
| 14,000 | 5.85 | 5.38 | 5.26 | 5.14 | 4.67 |
| 15,000 | 4.78 | 5.12 | 5.21 | 5.29 | 5.63 |
| 16,000 | 5.22 | 5.58 | 5.67 | 5.76 | 6.13 |
| 17,000 | 5.45 | 5.75 | 5.83 | 5.91 | 6.21 |
| 18,000 | 5.55 | 5.91 | 6.00 | 6.10 | 6.46 |
| 19,000 | 6.08 | 6.68 | 6.83 | 6.98 | 7.58 |
| 20,000 | 7.67 | 8.28 | 8.43 | 8.58 | 9.19 |
| 21,000 | 10.40 | 8.93 | 8.56 | 8.18 | 6.72 |
| 22,000 | 9.95 | 7.60 | 7.02 | 6.43 | 4.08 |
| 23,000 | 5.53 | 4.31 | 4.00 | 3.69 | 2.47 |
| 24,000 | 2.75 | 1.75 | 1.50 | 1.25 | 0.24 |
| 25,000 | 0.95 | 0.63 | 0.55 | 0.47 | 0.15 |
| 26,000 | 0.67 | 0.40 | 0.33 | 0.27 | 0.00 |
| 27,000 | 0.72 | 0.43 | 0.36 | 0.29 | 0.00 |
| 28,000 | 0.53 | 0.32 | 0.27 | 0.21 | 0.00 |
| 29,000 | 0.43 | 0.26 | 0.22 | 0.17 | 0.00 |
| 30,000 | 0.28 | 0.17 | 0.14 | 0.11 | 0.00 |
| 31,000 | 0.03 | 0.02 | 0.02 | 0.01 | 0.00 |
| 32,000 | 0.03 | 0.02 | 0.02 | 0.01 | 0.00 |
| 33,000 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| 34,000 | 0.05 | 0.03 | 0.02 | 0.02 | 0.00 |
(0 and 1,000 kg rows are 0.00 in all columns.)

Derived statistics (my computation from Table 16, bin value = label; confidence low-med, derived):

| Column | mean kg | SD kg | p10 | p50 | p90 | p99 |
|---|---:|---:|---:|---:|---:|---:|
| 40ft (100/0) | 17,114 | 5,110 | 10,000 | 18,000 | 23,000 | 28,000 |
| 20ft (0/100) | 14,354 | 5,634 | 6,000 | 15,000 | 21,000 | 23,000 |
| 50/50 mix | 15,735 | 5,555 | 8,000 | 17,000 | 22,000 | 27,000 |

Other Interpave §8.5 facts (high):
- "40ft containers weigh approximately 3,700kg when empty and 30,250kg when loaded to their legal maximum."
- Critical (most pavement-damaging) container weight: "100% of 40ft containers, the critical load is commonly 22,000kg and when 20ft containers are being handled, the critical load is 20,000kg ... mixes ... 21,000kg."
- Empty-stack note (§8.5/Fig 7B): special plant handles "empty containers up to eight high" and stacks must remain "stable under wind loading".

### 1b. VGM / weight misdeclaration statistics

| Dataset | Sample | Threshold | Share misdeclared | Source / quote | Conf. |
|---|---|---|---|---|---|
| VICT Webb Dock (Melbourne), April 2017, one vessel, EXPORT | 530 boxes | ±500 kg / ±1,000 kg / ±2,000 kg | 20.6% (109) / 8.5% (45) / 3.6% (19) | https://ftalliance.com.au/newsdetails/11919 — "an average of 20% of export boxes mis-declared by a variance of 500kgs"; "more than 8% of boxes were mis-declared by over a tonne." (post-SOLAS-VGM, 1 May 2017) | high (single-vessel sample) |
| same, IMPORT | 453 boxes | ±500 / ±1,000 / ±2,000 kg | 40.4% (183) / printed 11.5% (93) / printed 3.8% (29) | same URL — "over 40% mis-declared by a variance of 500kg". NOTE: printed import % at 1 t and 2 t are internally inconsistent with counts (93/453 = 20.5%, 29/453 = 6.4%). | med |
| Philippine Ports Authority, export containers | 2021: 212,018; 2022: 386,655; 2023: 286,994 | > ±1,500 kg | 2021 9.01% (19,104); 2022 8.08% (31,250); 2023 6.30% (18,078) | https://portcalls.com/misdeclaration-of-container-weight-down-since-issuance-of-vgm-rule — "allowable threshold weight discrepancy of above or below 1,500 kilograms"; "6.30% or 18,078 out of the 286,994 containers" (article year not printed; PPA AO 02-2021) | high (port-authority figures relayed by trade press) |
| WSC member feedback, Sept 2016 | — | — | 95% of shipments had a VGM value (presence, not accuracy) | https://www.ttclub.com/news-and-resources/news/article/tt-talk-vgm-nine-months-on/ — "95% of shipments had a VGM value." Also: reweighing finds values "discrepant by more than 1 tonne" (no %). | high (but not an accuracy stat) |

### 1c. Tare / max-gross by box type — Hapag-Lloyd "Container Specification" (Group Communications 03/2016)
PDF: https://www.hapag-lloyd.com/content/dam/website/downloads/press_and_media/publications/15211_Container_Specification_engl_Gesamt_web.pdf  (Confidence: high — carrier spec; values vary by series)

| Type (ISO code) | Tare kg (series range) | Max gross kg | Spec page |
|---|---|---|---|
| 20' GP 8'6" wooden floor (22G1) | 2,300 – 2,350 | 32,500 (some series 30,480) | p.9 |
| 20' GP steel floor | 2,250 – 2,570 | 32,500 | p.9 |
| 40' GP 8'6" (42G0/42G1) | 3,700 – 3,780 | 32,500 / 30,480 | p.11 |
| 40' HC 9'6" (45G0/45G1) wooden floor | 3,850 – 3,900 | 32,500 | p.13 |
| 40' HC steel floor | 3,830 – 4,300 | 32,500 | p.13 |
| 45' HC (L5G0/L5G1) | 4,700 – 5,050 | 32,500 | p.15 |
| 20' flatrack (flush-folding ends) | 2,740 – 2,940 | 34,000 / 40,000 / 45,000 | p.34 |
| 40' HC flatrack | 5,800 – 5,950 | 50,000 – 60,000 | p.36 |
| 20' reefer | 2,770 – 3,030 | 30,480 – 32,000 | p.40 |
| 40' HC reefer (incl. CA "ExtraFresh/EverFresh") | 4,300 – 4,570 | 34,000 | p.42 |
General page quote: "20' containers: Up to 32,500 kg ... for general purpose containers and up to 45,000 kg ... for Flatracks." "40' and 45' containers: Up to 32,500 kg ... up to 60,000 kg ... for Flatracks and 35,000 kg ... for reefer containers."
Spread-load limits: "for 20' containers: 4.8 tons ... per running meter in length" and "for 40' containers: 3 tons per running meter" (relevant to coil-in-container loading).
Cross-check: Interpave gives 40' empty ≈ 3,700 kg (consistent).

### 1d. Share of empties / reefers / OOG
Busan-specific full/empty, reefer and OOG shares: NOT FOUND (no BPA breakdown opened; a BPA open-data set "부산항만공사_부산항 컨테이너 수송통계", data.go.kr 15055478, was seen only as a search hit, not opened). KOSHA A-G-18-2026 confirms reefers need a plug check before lift and empties must be stacked low/stepped, lashed ≥3 high — handling rules only. Comparators found (2026-10-20 continuation):

| Item | Value | Source + quote | Conf. |
|---|---|---|---|
| World port throughput, full vs empty | 80 % full / 20 % empty (no year stated) | Notteboom, Pallis & Rodrigue, *Port Economics, Management and Policy*, "World Container Throughput, 1980-2024" (data credited to Drewry/UNCTAD): "both full and empty containers, which account for 80% and 20% of maritime traffic, respectively." https://porteconomicsmanagement.org/pemp/contents/part1/maritime-shipping-and-international-trade/world-container-throughput/ | med (textbook relaying Drewry) |
| World throughput level | 849 M TEU (2021) | same page: "In 2021, container traffic resumed its growth, reaching 849 million TEUs." | med |
| Gateway comparator — Port of Los Angeles, CY2024 | empties 3,446,471 of 10,297,352 TEU = **33.5 %** (my calc); empty exports = 69.7 % of exports; empty imports ≈ 0.14 % of imports | POLA "Historical TEU Statistics 2024", row "Total Calendar Year 2024": "Empty Imports" 7,529.30; "Empty Exports" 3,438,941.50; "Loaded Exports" 1,494,201.30; "Total TEUs" 10,297,352.25. https://www.portoflosangeles.org/business/statistics/container-statistics/historical-teu-statistics-2024 | high (port-authority table; trade-imbalance gateway, NOT a T/S hub like Busan) |
| Reefer share — Antwerp 2021 | **8.2 %** of TEU; >1 M reefer TEU | IIR/IIF news 2022-04-25 citing Port of Antwerp: "The share of reefer container volumes (in TEUs) is 8.2% of the total container traffic in the port." / "The 1 million TEU mark of operational reefer containers has been passed for the second year in a row." https://iifiir.org/en/news/reefer-container-throughput-in-main-european-ports | med |
| Reefer — Rotterdam | ~15 % "of container types" (metric not stated as TEU throughput; year not stated) | same page: "Reefer containers represent about 15% of container types." | low-med |
| OOG / flat-rack share | see 1e | — | — |

Sim note (DERIVED, low): a Busan preset could use empty share 20 % (world) as default, with a sensitivity range 20–35 % (world → gateway-with-imbalance); reefer 8 % (Antwerp-like). Both are comparators, not Busan data.

### 1e. OOG / flat-rack share
NOT FOUND as a share of port throughput (one English search; results were forwarder/glossary pages without statistics) → Unverified. What IS sourced: flat-rack tare and max gross (1c: 20' FR 2,740–2,940 kg tare, up to 45,000 kg gross; 40' HC FR 5,800–5,950 kg tare, 50,000–60,000 kg gross) and the KOSHA OOG handling rule (Q8b: sling worker fore and aft, 20–30 cm test lift). For a preset, OOG share must be an explicit ASSUMPTION parameter.

---------------------------------------------------------------------------------------------------

## Q2 Busan port: TEU, transshipment, crane productivity, wind downtime, wind limits

### 2a. Throughput
| Year | Total | Transshipment | T/S share | Source | Conf. |
|---|---|---|---|---|---|
| 2025 ("지난해" in a 2026-06-17 article) | 24,882 thousand (6 m container = TEU) | 14,097 thousand | 56.7% (my calc 14,097/24,882) | Kukinews 2026-06-17 citing 동남지방데이터청 "데이터로 본 부산과 세계 주요 항구 도시": "지난해 부산항 컨테이너 물동량은 2,488만2천개(약 6m짜리 컨테이너 기준)다." / "환적 물동량은 1,409만7천개로 세계 2위" https://www.kukinews.com/article/print/kuk202606170089 | med (statistics office via press; not BPA release) |
| 2025 (BPA release, 2026-02-02) | 24.88 M TEU (+2.0 %) | 14.10 M TEU (+4.4 %) | **57 %** (printed) | TWIG24 2026-02-02 relaying 부산항만공사: "지난해 부산항 컨테이너 물동량이 2488만 TEU로 전년 2440만 TEU보다 2.0% 증가해 역대 최고치" ; "환적 물동량은 전년 1410만 TEU로 전년보다 4.4% 증가" ; "이는 전체 물동량의 57% 비중이다." ; "환적 화물의 80%는 외국 선사가" https://www.twig24.com/news/local_bs/2026/02/02/20260202500225 (the "전년" before 1410만 is a wording slip: 1,410만 is the 2025 value, matching Kukinews 1,409.7만 and 57 % = 1,410/2,488) | high (BPA figures via press; consistent with the Kukinews row above) |
| 2024 (same BPA release, prior-year figure) | **24.40 M TEU** | ≈13.51 M TEU (DERIVED: 14.10 / 1.044) | ≈55.4 % (DERIVED) | same TWIG24 quote "전년 2440만 TEU" ; T/S only via the printed +4.4 % growth | total high; T/S share low-med (derived) |
| 2018, Busan NEW PORT only | 14,559 thousand TEU | — | 58.89 %; inter-terminal T/S = 26.3 % of T/S (15.51 % of New Port volume) | 김환성 외, "부산신항 ITT 도입 평가요소에 관한 연구" (KMOU + BPA authors), KINPR conference, koreascience CFKO201922850135977: "부산항 신항은 2018년도 기준으로 컨테이너 14,559천TEU를 처리하였으며, 환적은 58.89%을 점유" https://koreascience.kr/article/CFKO201922850135977.pdf | high (abstract) |

### 2b. Korean wind stop limits used for container handling (KOSHA guide)
Source: KOSHA GUIDE A-G-18-2026 "항만하역작업 안전에 관한 기술지원규정" (공표 2026-01-30), PDF:
https://portal.kosha.or.kr/openapi/v1/file/down/CTC2026012909544827454054/1  (Confidence: high — official KOSHA technical guide; it is guidance, not statute)

| Equipment / task | Limit | Verbatim (Korean) | Section |
|---|---|---|---|
| Quay (container) crane — appears under heading "7.10.1 야드 트랙터 운전자" but content is ship-to-shore spreader work | stop at gust ≥ 20 m/s | "(러) 순간풍속이 초속 20 m 이상의 강풍이나 돌풍이 발생한 경우 작업 중지 후 지휘계통에 따라 보고하여야 한다." | 7.10.1(2) p.32 |
| Transfer crane (yard RTG/RMG) | stop at ≥ 20 m/s | "(타) 초속 20 m 이상 강풍(돌풍) 발생 시에는 작업 중지 후 지휘계통에 따라 보고 하여야 한다." | 7.10.2(2) p.34 |
| Lashing work on container tops | stop at max gust ≥ 20 m/s | "(라) 순간 최대풍속이 20 m/s 이상일 때는 작업을 중지한다." | p.40 |
| Empty stacks | low/stepped, lash ≥3 high | "(5) 빈 컨테이너는 태풍, 돌풍 등에 대비하여 가급적 저단 또는 계단식으로 적재하고, 3단 이상은 라싱하여야 한다." | 7.10.x p.34-35 |
| Vehicles on quay | 30 km/h; 10 km/h with heavy load | "(버) 부두 내에서는 최고시속 30 km 이내로 운행하고, 중량물 작업 시에는 시속 10 km 이내로 서행" | 7.10.1 p.32 |

### 2c. Korean statutory wind thresholds (산업안전보건기준에 관한 규칙, 현행 2026-03-02 per nepla.ai mirror)
Source text mirror: https://www.nepla.ai/law/산업안전보건기준에-관한-규칙 (statute text; official home law.go.kr). Confidence: high.
- 제37조② "순간풍속이 초당 10미터를 초과하는 경우 타워크레인의 설치ㆍ수리ㆍ점검 또는 해체 작업을 중지 ... 순간풍속이 초당 15미터를 초과하는 경우에는 타워크레인의 운전작업을 중지" (개정 2017.3.3; applies to TOWER cranes only).
- 제37조① general: stop work when "비ㆍ눈ㆍ바람 또는 그 밖의 기상상태의 불안정으로 인하여 근로자가 위험해질 우려가 있는 경우".
- 제140조 "순간풍속이 초당 30미터를 초과하는 바람이 불어올 우려가 있는 경우 옥외에 설치되어 있는 주행 크레인에 대하여 이탈방지장치를 작동" (storm tie-down/anchoring of outdoor travelling cranes at >30 m/s).
- 제143조 after gust >30 m/s or earthquake ≥ 중진: inspect "기계 각 부위에 이상이 있는지를 점검" before using outdoor lifting appliances.

### 2d. Not found (in budget)
- Busan New Port ARMG/RTG moves/h and QC GMPH: two 2023 KINPR papers on Busan berth productivity (김정훈·류원형·박신우·남형식, koreascience CFKO202332743207881 / CFKO202321357600629) were opened but the online versions are extended abstracts with the numeric sections omitted ("(중략)"). → Unverified.
- Hours/days per year of wind-related cargo-handling suspension at Busan: no BPA/terminal statistic found. Papers found are on design wind (Typhoon Maemi "60m/s", design "75m/s" — 권순규 외 2006, koreascience CFKO200617054768283) and berthing limits, not handling downtime.
- Generic quay-crane productivity reference (Liftech, 2002, see Q9): conventional post-Panamax "computed average period ... 72 seconds"; with yard and normal delays "about 140 percent of this, or 100 seconds (36 lifts per hour)"; real terminals "as low as 65 percent and as high as 80 percent of the computed number." (high, but US/2002 context)

### 2e. Continuation (2026-10-20): productivity, berthing wind, weather-downtime comparators

| Item | Value | Source + quote | Conf. |
|---|---|---|---|
| Busan New Port QC productivity vs ship size | QC productivity falls as ship size rises; robust when yard congestion is controlled; monthly panel, 5 terminal operators, 2013–2023 (abstract gives NO moves/h figures) | Jeong (HMM PSA New-Port Terminal) & Kim (KMOU), "Containership size and quay crane productivity: empirical evidence from Busan New Port", *Maritime Economics & Logistics* 27(2) 293-303, 2025: "quay crane productivity is negatively affected by containership size" https://ideas.repec.org/a/pal/marecl/v27y2025i2d10.1057_s41278-024-00286-0.html | high (direction only) |
| Busan STS net cycle (Hutchison Korea Terminal, 14 STS, 2013–18) | 1.44–1.56 min per move (avg 1.50) → ≈40 net moves/h per crane (DERIVED 60/1.5) | cross-reference: R4_stats_gaps.md Q4a (Jo & Kim, JMSE 2020 8(1) 6, Table 2 "Net cycle (min)") | high (as recorded in R4) |
| Berthing wind limit, large container ship at Busan (vessel side, not crane) | mean ≤ 13.5 m/s workable with 2 tugs; ≥ 16.9 m/s (incl. 25 % gust) "difficult", 3 tugs | Yoo, Yun & Lee (Korea Maritime Univ.), KOSOMES 2006 fall conf. pp.303-308: "difficult to conduct and get her alongside to the piers under rough weather conditions of wind force 16.9m/sec or more"; "정상풍속 13.5m/sec 이하의 바람에서는" https://koreascience.or.kr/article/CFKO200601436761083.page | med (abstract only) |
| Wind-downtime comparator — Cape Town Container Terminal, Feb 2025 | > 200 operational hours lost in two weeks; 80 km/h (22.2 m/s) "threshold for safe cargo handling"; winds 100–120 km/h | FreshPlaza 2025-02-19: "lost over 200 operational hours"; "the 80 km/h threshold for safe cargo handling." https://www.freshplaza.com/north-america/article/9706519/additional-resources-deployed-to-counter-windy-conditions-at-cape-town-container-terminal/ | med (trade press; notoriously windy port — not transferable to Busan) |

Searches for a Busan per-year wind-downtime statistic and for Busan New Port ARMG/RTG moves/h (Korean "부산항 강풍 하역 중단 시간", "부산신항 장치장 크레인 시간당 처리" and English equivalents) returned only typhoon whole-port closure news (seen as snippets, not opened) and no numbers → Unverified. Busan fog: 14.4 fog days/yr (KMA 1991–2020) is in R4_stats_gaps.md Q5; fog-closure hours not found there either.

---------------------------------------------------------------------------------------------------

## Q3 Steel coils / slabs / coil tongs

| Item | Value | Source + quote | Conf. |
|---|---|---|---|
| Cold-rolled coil weights (sea transport) | 2 – 28 t | Standard Club (with Lloyd's Register), *A Master's Guide to: The Carriage of Steel Cargo*, 3rd ed., Oct 2020, §03 table "Finished steel products" — CR coils: "Finished sheet steel in a transportation coil. 2 to 28 tonne weights". PDF: https://iims.org.uk/download/241 | high (P&I club guide) |
| Hot-rolled coil weights | 2 – 30+ t | same table — HR coils: "Sheet steel being transported to a rolling mill in 2 to 30+ tonne weights" (table row layout extracted by pdftotext; CR/HR assignment read from row order) | high/med |
| Single HR coil example | 20 – 24 t | TIS (GDV transport information service), "Photo of the month – April 2008": "This coil would weigh somewhere between 20 and 24 tonnes and is not secured at all." https://www.tis-gdv.de/?p=29638 | med |
| Forklift for coils | much heavier than coil | Standard Club §07: "A forklift capable of lifting a 20-tonne coil will itself weigh much more than 20 tonnes." | high |
| Coil stowage rule of thumb | key coil gap ≤ 60 % of key coil diameter; 50 % of upper-tier coil weight passes to each coil below | Standard Club §07 (Fig. 27: "50% of the weight of a coil in an upper tier is assumed to pass to the coils below") | high |
| Slab weights | NOT FOUND (Standard Club only says slabs are "loaded eight high, with the top three slabs lashed together") → Unverified U12 | — | — |
| C-hook / coil-tong self-weight | NOT FOUND → Unverified U13 | — | — |
| Coil-yard crane pick/drop time and kinematics | 35 s pick + drop per loaded move; portal 1.667 / trolley 0.833 / hook 0.2 m/s | see Q9d (Weckenborg et al. 2025) | med |

## Q4 Precast concrete

Main source: WorkSafe Queensland (Office of Industrial Relations), *Tilt-up and precast construction Code of Practice 2026* (file name "...-2026-draft.pdf"):
https://www.worksafe.qld.gov.au/__data/assets/pdf_file/0018/163035/tilt-up-and-precast-code-of-practice-2026-draft.pdf  (Confidence: high as a regulator's code; DRAFT status per file name)
Change summary: https://www.worksafe.qld.gov.au/__data/assets/pdf_file/0019/163036/summary-of-key-changes-to-tilt-up-and-precast-cop-2003.pdf

| Item | Value | Verbatim | Section |
|---|---|---|---|
| Mobile-crane wind limit (small frontal area loads) | 10 m/s (36 km/h) | "This maximum wind speed is commonly 10 metres/second (36 km/h) and is based on loads with a small frontal area being lifted." | §12.1 |
| Large panels | must be reduced below 36 km/h by engineer | "For concrete wall panels of a large surface area, it is likely that maximum safe wind speed for the mobile crane will need to be lower than 36 km/h." | §12.1 |
| Tower crane limit (Qld) | 15 m/s (54 km/h); design up to 72 km/h "largely impractical" | "specifies a maximum wind speed of 54 km/h (15 metres/second) for tower crane operations" | §12.2 |
| Old 2003 cap (now replaced by risk assessment) | 45 km/h (12.5 m/s) | 2003 Code: "lifting of concrete elements should not continue under any circumstances when the wind speed exceeds 45 km/h (12.5m/s)" | summary §12.2 |
| Gust direction change | up to ~30° | "the direction of the wind can change by up to approximately 30 degrees during a wind gust" | §12 |
| Wind mitigation | lift less wind-sensitive elements, tag lines, lay-down until wind subsides | "lifting precast elements that are less susceptible to wind effects including narrow columns, floor planks, stair flights and beams" | §12.2 |
| Crane sizing rule | rated capacity ≥ 3× heaviest panel | "usually have a maximum rated capacity of at least three times the mass of the heaviest panel ... with a tilt-up panel of 20 tonnes mass, a mobile crawler crane with a maximum rated capacity of 60 tonnes" | §11.2 |
| Radius allowance | +1.5 m | "at least 1.5 metres should be added to the working radius of the crane" | §11.2 |
| Heavy-lift trigger (high-risk SWMS list) | bridge beams ≥ 10 t; any load ≥ 50 t | "heavy lifts (such as bridge beams (10 tonnes or more) or other lifts where the load is 50 tonnes or more" | §1.4 |
| Bridge beams | often dual mobile-crane lifts; intermediate rigger minimum | "they are often lifted into position using two mobile cranes"; "an intermediate rigger (as a minimum) will be required" | §12.3 |
| Exclusion zone | ≥ 1.25 × element height from centre line, and ≥ 1.25 × longest dimension horizontally | "at least 1.25 times the height of the element measured from the centre line of the element in all directions ... minimum horizontal distance of 1.25 times the longest dimension" | §13.1 |
| Panel release from bed | stop if not released at panel+rigging weight | "If the panel is not released when the crane indicator registers the combined weight of the panel, rigging and other suspended dead load ... lifting should be stopped." | §14.1.1 |
| Rotation (tilting) rule | aux winch / lesser crane ≥ 75 % of element mass; >50 % load on one rope | "the auxiliary winch should have a minimum capacity of 75 per cent of the element mass"; "more than 50 per cent of the element weight will be applied to one of the hoist ropes" | §14.2 |
| Rotation geometry limits | aux pick ≤ 1/4 length from end; rope included angle ≤ 45°; aux block+gear ≤ 200 kg | verbatim conditions in §14.2 | §14.2 |
| Hollow-core plank geometry | 1,200 mm wide; 150–400 mm thick; beds 120–170 m | "Planks are normally 1200 mm wide; thicknesses range from 150 mm to 400 mm" | §12.4 |

### 4b. Element weights (2026-10-20 continuation)

Hollow-core planks (width 1,200 mm per Qld code §12.4 above):
| Product | Depth | Weight | Source + quote | Conf. |
|---|---|---|---|---|
| Mannok Precast Hollowcore 200 (UK/IE) | 200 mm | 276 kg/m² (page also prints "Density" 2.89 kN/m²) | NBS Source product page: "276 kg/m²" https://source.thenbs.com/en/product/mannok-precast-hollowcore-200/cA2VsQoyxaFUgRyYk6yfgn/t77f18LG8ZRKHrvUbptSK1 | med (manufacturer listing) |
| Mannok Hollowcore 300 | 300 mm | 380 kg/m² ("Density" 3.95 kN/m²) | https://source.thenbs.com/en/product/mannok-precast-hollowcore-300/sYZ7ozwhn1SdKGZFXuyRPG/tpFNyJXdXXdRyk9i4y1mCv — "380 kg/m²" | med |
| Mannok Hollowcore 400 | 400 mm | 482 kg/m² ("Density" 5.02 kN/m²) | https://source.thenbs.com/en/product/mannok-precast-hollowcore-400/7KNJv7b4Y62m2LKyVFGq7s/bJHHYmXzbqfR8kpXjeXKJj — "482 kg/m²" | med |
| Milbank propped composite hollowcore (UK), self-weight INCLUDING structural topping | unit 150 / 200 / 250 mm (overall 200 / 250 / 300 mm) | 3.68 / 4.17 / 4.66 kN/m²; span with 1.5 kN/m² finishes + 0.75 kN/m² imposed: 8.85 / 10.20 / 11.35 (m — unit not printed in the extracted text, inferred) | Milbank "Propped Composite Hollowcore Load/Span Table" (PDF dated 2023-04): column "Self Weight inc. Topping (Kn/m2)". https://www.milbank.co.uk/app/uploads/2023/04/propped-composite-Hollowcore-load-table.pdf | high (table opened via pdftotext) |
Note: Mannok's kN/m² values are ≈6 % above kg/m² × g (276 kg/m² → 2.71 kN/m²), probably including joint fill — use kg/m² for lifted mass. DERIVED lift masses (1.2 m wide, my calc): 200 mm × 12 m ≈ 4.0 t; 300 mm × 12 m ≈ 5.5 t; 400 mm × 16 m ≈ 9.3 t.

Double-tees (PCI Design Handbook 8th ed. (2017) pretopped load tables, normalweight f'c 5,000 psi; "typically used in parking structures"):
| Section | Size | Weight | Spans in table | Source + quote | Conf. |
|---|---|---|---|---|---|
| 12DT30 | 12 ft 0 in × 30 in | wt = 939 lb/ft (1,397 kg/m) NW; 720 lb/ft LW; DL 78 psf (3.73 kN/m²) NW / 60 psf LW | 40 – 82 ft | PCI table §3.5 "Pretopped Double-Tee Load Tables": "wt = 939 lb/ft"; "DL = 78 lb/ft2" https://ems-www.pci.org/PCI_Docs/Design_Resources/Design_Tables/PCI-DH-2017/PretoppedDoubleTees_12DT30and12LDT30.pdf | high |
| 15DT34 | 15 ft 0 in × 34 in | wt = 1,234 lb/ft (1,836 kg/m) NW; 946 lb/ft LW; DL 82 psf (3.93 kN/m²) NW / 63 psf LW | 32 – 88 ft | same series: "wt = 1234 lb/ft  946 lb/ft"; "DL = 82 lb/ft2  63 lb/ft2" https://ems-www.pci.org/PCI_Docs/Design_Resources/Design_Tables/PCI-DH-2017/PretoppedDoubleTees_15DT34and15LDT34.pdf | high |
DERIVED piece masses (my calc, NW): 12DT30 × 60 ft (18.3 m) ≈ 25.6 t; 15DT34 × 60 ft ≈ 33.6 t; 15DT34 × 88 ft (26.8 m) ≈ 49.3 t.

Bridge girders:
| Item | Value | Source + quote | Conf. |
|---|---|---|---|
| California standard I-girders CA I36 … CA I66 | 450 … 669 lb/ft (670 … 996 kg/m) | Caltrans Bridge Design Memo 5.3 (Sept 2023), Table 5.3.4.1-1, column "Weight (lbs/ft)" https://dot.ca.gov/-/media/dot-media/programs/engineering/documents/bridgedesignmemos/09/092023bdm-53--precast-prestressed-girders-a11y.pdf | high |
| California bulb-tee CA BT49 … CA BT85 | 892 … 1,205 lb/ft (1,327 … 1,793 kg/m) | same memo, Table 5.3.4.1-2: "CA BT49 ... 892" … "1,205" | high |
| Span ranges (possible / preferred) | I-girder 50–125 / 50–95 ft; bulb-tee 80–150 / 95–150 ft; wide-flange 80–200 / 80–180 ft; precast double-T girder 30–100 / 30–60 ft; segmental precast 200–450 ft | same memo, Table 5.3.4.3.1/2; "The preferred span lengths take into account the girder weight for transportation and erection." | high |
| WSDOT wide-flange (thin-)deck girders: single-piece shipping weight | normal-weight tables: WF36TDG 98–102 kips (44–46 t) at 100–120 ft … 261–270 kips (118–122 t) for the deepest sections at 195–225 ft; lightweight-concrete WF36TDG as low as 65 kips (29.5 t); spans up to 225 ft (TDG) / 195 ft (DG) | WSDOT Design Memorandum (B. Khaleghi), 2014-12-18, shipping-weight tables; "Shipping weight < 270 kip"; "The shipping weight limits shown above are for a single girder segment and do not include the weight of the hauler." ; "Girders of 270+ kips could be shipped to some specific projects." https://wsdot.wa.gov/publications/fulltext/Bridge/designmemos/12-2014.PDF | high |
DERIVED (my calc): CA I48 × 95 ft ≈ 23 t; CA BT73 × 150 ft ≈ 75 t; CA BT85 × 150 ft ≈ 82 t. With the Qld §1.4 trigger (bridge beams ≥ 10 t = high-risk) essentially every bridge girder is a planned heavy lift, often by two cranes (§12.3).

Precast segmental box-girder SEGMENT weights: NOT FOUND (no opened source) → Unverified.

## Q5 Shipyard blocks, goliath cranes

| Item | Value | Source + quote | Conf. |
|---|---|---|---|
| DSME Okpo "900톤 골리앗" | lifts 900 t to 78 m; span 180 m; height 90 m; crane self-weight ~5,500 t; installed in one lift by two 3,600 t floating cranes (2009) | kidd.co.kr 2009-03-01: "900톤의 중량물을 최고 78m까지 들어올려"; "폭 180m, 높이 90m"; "무게만도 5,500톤에 달하는 '900톤 골리앗 크레인'" https://kidd.co.kr/news/122399 | med (trade press) |
| HHI Gunsan goliath | 1,650 t ("현재 세계 최대 갠트리 크레인" in 2018) | 류기수, "세계 최대 갠트리 크레인", 대한조선학회지(BSNAK) Vol.55 No.2, June 2018, p.29-33: "현재 세계 최대 갠트리 크레인은 ... 현대중공업의 1,650톤 크레인이다." https://koreascience.or.kr/article/JAKO201819757623508.pdf | high |
| HD현대삼호 goliath use | blocks & engines up to 1,600 t | 문화일보 2024-11-20: "골리앗 크레인은 최대 1600t의 대형 블록과 엔진 등을 탑재하는 데 사용한다." https://www.munhwa.com/article/11467069 | med |
| Tera-block (floating-crane, not goliath) | ~100 m long, ~12,000 t | BSNAK 2018 (above): "테라 블록의 경우 최대 길이는 100여 미터(m), 중량은 12,000여 톤(ton) 정도" | high |
| World's largest gantry (Sembcorp Marine, 2 units, KOCKS-type) | 15,000 t; 127 m high, 181 m wide | 류기수·정사교·이상호, "Post Goliath Crane", 대한조선학회지(BSNAK) Vol.57 No.1, March 2020, p.47-50 (quoting press): "높이 127m, 너비 181m 크기의 세계 최대 규모인 15,000톤 골리앗 크레인" https://www.koreascience.or.kr/article/JAKO202010163510048.pdf ; Table 1 lists max lift per crane: KOCKS 15,000 t, ableENC (Korea) 1,650 t | high |
| Goliath design note | hook capacity and block turnover must be fixed at planning | BSNAK 2018: "후크(Hook)의 용량(Capacity)과 권상물의 반전(Turn Over) 여부도 사전에 감안" ; KOCKS type = double girder, trolley spanning both girders + trolley between girders | high |
| 700 t class goliath (Sungdong Masan) | self-weight 3,200 t, height 105 m | 아주경제 2016-07-10: "700t 골리앗 크레인은 크레인 자체 무게만 3200t, 높이는 105ｍ" https://www.ajunews.com/view/20160710103139017 | med |
| Wind event (not an operating limit) | 900 t-class goliath being ERECTED at SHI Geoje collapsed at 21 m/s (2009-03-21) | 아주경제 2009-03-22: "초속 21ｍ의 강풍에 무너졌다" https://www.ajunews.com/view/20090322000107 | med |
| Block-shop output | 100,000 t of blocks/yr (SHI block plant, 280,000 m²) | same article: "연간 10만t의 블록을 제조할 수 있는" | med |
| Typical block weight distribution; goliath in-service wind limit | NOT FOUND (see Unverified) | — | — |

---------------------------------------------------------------------------------------------------

## Q6 Wind limit with load sail area (EN 13000 / Liebherr / EN 13001-2)

Source: Liebherr-Werk Ehingen, "Influence of wind on crane operation" (mobile & crawler cranes training brochure, LiWINDiag screenshot dated 20.11.2023), PDF:
https://assets-cdn.liebherr.com/versions/61d28acb-d5bc-4fad-8941-6b243f024396/original  (Confidence: high)

Formula (Method 2, §5.2; the formula itself is an image — form confirmed by back-calculating Liebherr's worked example):

    v_max = v_max_TAB · sqrt( 1.2 [m²/t] · m_H / A_W ),   A_W = A_P · c_D   (older Liebherr docs write c_W)

- m_H = hoist load "incl. lifting accessories and hook block and any relevant portion of hoisting cable" (§5.2).
- A_W definition (§4.2.1): "It is composed of the projected surface area AP and the cD-value." "AW = AP × cD".
- Meaning of 1.2 m²/t (§4.2.2): load-chart wind speeds use "Standard values from EN 13000 ... (load reference value of 1.2 m² per ton)."
  Box (§5.2): "The value 1.2 m²/t below the root corresponds to the constant according to EN 13000 and not the cD-value! This value may not be changed!"
  §5.1: "If the surface area exposed to wind of the load is greater than 1.2 m² per t load then the maximum permissible wind speeds per the load table are no longer valid."
  => 1.2 m²/t is the reference WIND-EXPOSED area (A_P·c already included) per tonne of hoist load assumed in EN 13000 load charts.
- Reference wind: "vmax ... and ... vmax_TAB relate at all times to the 3-second gust speed prevalent at highest point on the crane." EN 13000 gust = "average value of the wind speed measured for a duration of 3 seconds".
- Worked example (§5.1.1/5.2.1): m_H 65 t, c_D 1.4, A_P 200 m² → A_W 280 m² (4.31 m²/t); v_max_TAB 11.2 m/s → "The wind speed from the load chart drops from 11.2 m/s to 5.9 m/s." Check: 11.2·sqrt(1.2·65/280) = 5.91 m/s ✔.
- Example 2: 85 t, c_D 1.2, A_P 50 m² → A_W 60 m² (0.71 m²/t) → result > v_max_TAB, so "the load can be lifted up to the maximum wind speed specified in the load chart, here 9 m/s." (formula output is capped at v_max_TAB).
- Dynamic pressure / force (§4.2.3-4): "p = 0,5 × ρ × v²"; "FW = AW × p".

c_D table (§4.1.3 "Some examples of common bodies"):
| Body | c_D |
|---|---|
| Plate / cube | 1.1 – 2.0 |
| Cylinder | 0.6 – 1.0 |
| Ball | 0.3 – 0.4 |
| Hemisphere (front) | 0.2 – 0.3 |
| Hemisphere (rear) | 0.8 – 1.2 |
| Wind power plant rotor | ca. 1.6 |
Liebherr exercise uses a window pane A_P 2.6 m², c_D 1.2 → A_W 3.12 m².

Height factor table (§2.2.1, p.14; extracted columns realigned so that the 100 m row matches the worked example "6,2 m/s x 1,272 = 7,89 m/s"):
| Height m | factor on 10-min mean @10 m | factor on 3-s gust @10 m |
|---:|---:|---:|
| 10 | 1.400 | 1.000 |
| 20 | 1.502 | 1.073 |
| 30 | 1.566 | 1.119 |
| 40 | 1.614 | 1.153 |
| 50 | 1.653 | 1.181 |
| 60 | 1.685 | 1.204 |
| 70 | 1.713 | 1.224 |
| 80 | 1.738 | 1.241 |
| 90 | 1.760 | 1.257 |
| 100 | 1.780 | 1.272 |
| 150 | 1.874 | 1.339 |
| 200 | 1.921 | 1.372 |
(confidence med for the row alignment — pdftotext shifted the column; 100 m row verified by Liebherr's own example.)

Corroboration (2026-10-20 continuation) that 1.2 m²/t is the wind-exposed area INCLUDING the shape coefficient:
| Source | Quote | Conf. |
|---|---|---|
| Liebherr UpLoad magazine 01/2020, expert tip "Safe to use even in high winds" (J. Henkel pictured) https://www.liebherr.com/en-at/mobile-and-crawler-cranes/customer-magazine/my-tip/safe-to-use-in-high-winds-5924163 | load charts assume "an area exposed to the wind (including cw value) on the load of 1.2 m² per tonne" | high (manufacturer) |
| Cranes Today, "High wind load, high workload", 2011-01-25 https://www.cranestodaymagazine.com/?p=5059 | EN 13000 guidance: sail area per tonne "in the order of, or less than, 1.2m2 of sail area per tonne"; sail area is multiplied by the load's "Cw factor" | med (trade press) |
Neither page prints the formula as text; the formula form stands on the Liebherr brochure's worked example (back-calculated above). Typical c_W for box-shaped cargo (containers, panels, blocks): only the generic Liebherr "Plate / cube 1.1 – 2.0" band above — no container- or panel-specific c_W found.

DERIVED illustration (my calc, low — inputs are assumptions except the formula): a 40' HC side-on (broadside) area ≈ 12.2 m × 2.9 m ≈ 35 m² (nominal external dims; end-on only ≈ 2.44 × 2.9 ≈ 7 m²), m_H 30 t incl. spreader share, c_W 1.2 → A_W ≈ 42 m² = 1.4 m²/t > 1.2 → v_max ≈ v_max_TAB·sqrt(1.2·30/42.4) ≈ 0.92·v_max_TAB. A 10 t, 3 m × 10 m precast panel lifted flat-face to wind (c_W 1.2): A_W 36 m² → 3.6 m²/t → v_max ≈ 0.58·v_max_TAB. An EMPTY 40' HC side-on (≈3.9 t box; ≈3–10 m²/t depending on how much spreader mass is counted) is far above 1.2 m²/t — consistent with the KOSHA empty-stack wind rule.
Caveat: EN 13000 is the MOBILE-crane standard; STS/RTG/RMG and goliaths are designed to EN 13001 / FEM 1.001 in-service wind, so for yard/quay cranes this formula is only a scaling heuristic for sail-area-heavy loads, not a code rule.
(Still open → Unverified U8: EN 13001-2 default hoist-load wind area/shape coefficient when the load is not specified.)

---------------------------------------------------------------------------------------------------

## Q7 Hazardous cargo (IMDG) handling restrictions, Korea
KOSHA GUIDE A-G-18-2026 §7.9.1 위험물 하역작업 (URL above; high, guidance):
- "(6) 위험물 중 국제해상위험물규칙(IDMG Code)의 Class1(화약류), Class2(가스류), Class6.2(전염성물질), Class7(방사능물질)은 직 선적 및 직 반출시켜야 하며" — i.e., Class 1/2/6.2/7 = direct ship↔truck, no yard dwell; if the trailer is absent the plan is changed and other boxes are worked first.
- "(7) ... 위험화물임을 장비 운전자(크레인, 트레일러)에게 알려 주어야 하고 ... 작업주변의 차량 및 관련자 이외 작업자들의 접근을 통제"
- "(10) 적․양하 시 작업현장 주변에서 흡연, 용접, 절단작업 등 화재의 위험이 있는 작업과 행동을 하여서는 아니 된다."
- p.13-14 (육상 작업지휘자 duties): "(사) 위험물 적․양하시 소화기 및 위험표지판을 설치하고 차량 및 작업자의 접근을 통제하여야 한다." DG safety officer: "(라) 위험물 하역 시에는 입회하여 위험지역 설정 후 작업자 및 차량을 통제하고 “위험물 작업 중” 표시와 적절한 소화설비를 비치" — exclusion zone required, but NO numeric distance given.

Statute — 선박의 입항 및 출항 등에 관한 법률 (현행 2026.05.12 per nepla.ai mirror https://www.nepla.ai/law/선박의-입항-및-출항-등에-관한-법률 ; high):
- 제34조① DG handling inside a trade-port area needs a 자체안전관리계획 approved by 관리청.
- 제34조③ "관리청은 기상 악화 등 불가피한 사유로 무역항의 수상구역등에서 위험물을 하역하는 것이 부적당하다고 인정하는 경우에는 ... 그 하역을 금지 또는 중지하게 하거나 무역항의 수상구역등 외의 장소를 지정하여 하역하게 할 수 있다." → weather-based DG stop is a discretionary authority order; NO numeric wind/lightning criterion in the Act.
- 시행규칙 (현행 2024.11.13) 제17조①: such prohibition/suspension is notified "서면으로 ... 다만, 긴급한 경우에는 구두(口頭)로 통보할 수 있다."
- 제35조① required DG-handling measures: "1. ... 위험물 안전관리자 ... 확보 및 배치", "3. 위험물의 특성에 맞는 소화장비의 비치", "4. 위험표지 및 출입통제시설의 설치", "5. 선박과 육상 간의 통신수단 확보", "6. 작업자에 대한 안전교육".
- Negative result: full-text search of 위험물 선박운송 및 저장규칙 (nepla.ai mirror) found NO "낙뢰/번개/천둥/풍속" clause. It does limit explosives storage: "순화약질량 10톤을 초과하는 수량의 화약류를 동일 저장선 안에 저장하지 말 것".

International comparator (med): Western Australia DMP (2015-02-18, via AMSJ) "recommends operators suspend loading and unloading of flammable dangerous goods (for example petrol)" when "severe lightning storms are in the vicinity" — no distance/time criterion. https://www.amsj.com.au/warning-issued-lightning-dangerous-goods-explosives/
→ For the sim, a lightning-distance rule (e.g. "storm within N km") would be an ASSUMPTION; no sourced number found.

---------------------------------------------------------------------------------------------------

## Q8 Critical / heavy lift definitions and procedures

### 8a. Korea — 산업안전보건기준에 관한 규칙 (statute; high)
- 제38조① "다음 각 호의 작업을 하는 경우 ... 별표 4에 따라 ... 사전조사를 하고 ... 작업계획서를 작성하고 그 계획에 따라 작업을 하도록 해야 한다." Item "11. 중량물의 취급작업" (also 2. 차량계 하역운반기계등을 사용하는 작업).
- 별표 4 <개정 2025. 9. 1.> item 11 중량물의 취급 작업 — 사전조사 내용: "-" (none); 작업계획서 내용: "가. 추락위험을 예방할 수 있는 안전대책 / 나. 낙하위험 ... / 다. 전도위험 ... / 라. 협착위험 ... / 마. 붕괴위험을 예방할 수 있는 안전대책". (Annex PDF from nepla.ai mirror, flSeq=158646333.)
- 제39조① "제38조제1항제2호ㆍ제6호ㆍ제8호ㆍ제10호 및 제11호의 작업계획서를 작성한 경우 작업지휘자를 지정하여 작업계획서에 따라 작업을 지휘하도록 해야 한다."
- 제40조① 신호 required for "1. 양중기(揚重機)를 사용하는 작업" and "5. 중량물을 2명 이상의 근로자가 취급하거나 운반하는 작업".
- Note: the statute defines no % of rated capacity threshold for "중량물" (none found in 제38조/별표4).

### 8b. Test lift ("stopping") practice — KOSHA GUIDE A-G-18-2026 (high, guidance)
- Quay crane: "(다) 컨테이너를 취급하는 경우 약 20 cm 높이에서 일단 정지 후 화물상태 및 위치를 확인하여 작업하는 스토핑(Stopping) 작업을 실시" and "(파) 화물을 20 cm 정도 권상하여 화물의 편하중을 확인한 후, 스프레더의 중심을 맞추어 권상".
- Transfer crane: "(사) 컨테이너 권상 시 20 cm 정도 들었다가 정지하여 화물상태를 확인한 후 다시 권상"; before restart "(카) ... 호이스트와 트롤리를 2～3회 앞뒤로 서서히 작동"; pre-shift "(다) ... 권상, 권하 및 주행을 2～3회 반복하는 시험운전".
- Special/OOG (flat rack etc.): one sling worker fore and aft, "20 cm ～ 30 cm 정도 권상한 후 이상이 없으면 신호에 따라 서서히 권상".
- Rated-capacity: "(바) 크레인의 정격하중을 초과하는 컨테이너는 작업지휘자가 사전 작업을 중지하고 선사에 계통보고".
- Reefer: "(6) 냉동 컨테이너의 경우 전기플러그의 연결 여부를 확인한 후 권상".
- Yard rule: "(차) 40 ft(12 M) 컨테이너 위에 20 ft(6 M) 컨테이너를 적재하여서는 아니 된다."

### 8c. "Critical lift" definitions with numbers (regulations; high)
WorkSafeBC OHS Regulation Part 14, s.14.1 (https://www.worksafebc.com/en/law-policy/occupational-health-safety/searchable-ohs-regulation/ohs-regulation/part-14-cranes-and-hoists):
| Trigger | Threshold |
|---|---|
| Mobile crane / boom truck | "exceeds 90% of its rated capacity" while at a radius "of more than 50% of its maximum permitted load radius" |
| Tandem lift | "if the load on any one crane, hoist or other piece of powered lifting equipment exceeds 75%" of its rated capacity |
| Multi-crane | "simultaneous use of more than two cranes" |
| Floating base | "exceeds 90% of rated capacity for the lifting system" |
| Other | personnel in suspended platform; CoG changes during lift; sling-leg length changes during lift; over/between energized HV conductors; submerged load |
Plan (s.14.42.1): "A written lift plan must be prepared for every critical lift" incl. rigging details, "wind speed limitations", max hoist-line speed, max travel speed, load distribution, signallers; documented pre-job meeting.
WorkSafeNB (New Brunswick) Reg. 91-191 s.212.1 — same structure: mobile crane >90 % at >50 % radius; tandem "if the load of one of them exceeds 75% of its rated capacity"; >2 hoisting apparatuses. https://www.worksafenb.ca/safety-topics/critical-lifts/

ASME P30.1 (med/low — standard text not opened): Maxim Crane blog (updated 2026-07-09) "Organizations commonly define a critical lift as one that exceeds 75% of a crane's rated capacity" (citing USACE EM 385-1-1 §16, which returned HTTP 403) and "there is no single industry-wide percentage"; multi-crane lifts need "a written lift plan prepared by a qualified person under consensus standards such as ASME P30.1." https://www.maximcrane.com/blog/what-is-a-tandem-lift/

### 8d. Test lift / brake check (US federal rule; high)
OSHA 29 CFR 1910.179(n)(3)(vii) (overhead & gantry cranes): "The operator shall test the brakes each time a load approaching the rated load is handled. The brakes shall be tested by raising the load a few inches and applying the brakes." https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.179
1910.179(k)(2) rated-load test: "Test loads shall not be more than 125 percent of the rated load unless otherwise recommended by the manufacturer."
1910.179(b)(4): "Outdoor storage bridges shall be provided with automatic rail clamps." + wind-indicating alarm at a predetermined velocity.
→ Sim test-lift parameters supported by sources: lift 0.2 m and stop (KOSHA, containers), 0.2–0.3 m (KOSHA, OOG/special), "a few inches" (OSHA, near-rated loads).

---------------------------------------------------------------------------------------------------

## Q9 Handling-time breakdowns per cargo type

### 9a. STS + RTG element times, BCT Gdynia (PERT triangular to / td / tp, seconds)
Source: A. Bartosiewicz & A. Kucharski, "The Determination of Times of Transhipment Processes at Maritime Container Terminals", TransNav 16(3), Sept 2022, DOI 10.12716/1001.16.03.13, Table 2 & 3.
PDF: https://www.transnav.eu/files/The_Determination_of_Times_of_Transhipment_Processes_at_Maritime_Container_Terminals,1243.pdf  (Confidence: high for this terminal; partly "adopted" assumed values, e.g. "tHL ... load hooking (3 s)", unhooking "(10 s)")

| Equip. | Element | UNLOADING to/td/tp | LOADING to/td/tp |
|---|---|---|---|
| STS | A. Load hooking (spreader lock) | 3 / 3 / 3 | 3 / 3 / 3 |
| STS | B. Load lifting | 4 / 4 / 4 (deck boxes, lift at 3 m) | 17 / 25 / 32 |
| STS | C. Trolley travel + load lowering | 34 / 48 / 60 | 15 / 20 / 25 |
| STS | D. Load unhooking (incl. setting on vehicle when unloading) | 10 / 10 / 10 | 5 / 5 / 5 |
| STS | E. Spreader lifting + trolley return | 16 / 25 / 33 | 12 / 18 / 23 |
| STS | F. Spreader lowering | 12 / 12 / 12 | 12 / 15 / 18 |
| IMV | Transfer to/from yard | 37 / 74 / 131 | 37 / 74 / 131 |
| RTG | A. Load hooking | 3 / 3 / 3 | 3 / 3 / 3 |
| RTG | B. Load lifting (to 15 m) | 26 / 26 / 26 | 8 / 19 / 29 |
| RTG | C. Gantry (bridge) travel | 0 / 36 / 66 | 0 / 36 / 66 |
| RTG | D. Trolley travel | 5 / 14 / 20 | 5 / 14 / 20 |
| RTG | E. Load lowering + unhooking | 28 / 38 / 49 | 46 / 46 / 46 |
| RTG | F. Spreader lifting | 7 / 12 / 17 | 15 / 15 / 15 |
| RTG | G. Gantry + trolley travel (return) | 5 / 36 / 66 | 5 / 36 / 66 |
| RTG | H. Spreader lowering | 20 / 20 / 20 | 12 / 17 / 22 |
Table 3 (PERT mean ± SD per container): STS unloading 101.5 ± 5.18 s; STS loading 85.7 ± 3.66 s; IMV 77.3 ± 12.60 s; RTG 183.5 ± 15.67 s (unload) / 184.2 ± 15.67 s (load). Text: "The average working time of the STS unloading a container from a ship is approx. 102 seconds".

### 9b. Quay-crane dwell overhead (Liftech)
M. A. Jordan (Liftech Consultants), "Quay Crane Productivity", TOC Americas, Miami, Nov 2002. https://www.liftech.net/wp-content/uploads/2002/11/Quay-Crane-Productivity-Paper.pdf (high)
- "dwell times, starting and stopping motions, finding spots on the vessel and quay, and checking clearances ... takes about 30 seconds" per cycle.
- Table 2: 30/45/60/75/90 lifts/h ↔ 120/80/60/48/40 s per lift.
- Conventional post-Panamax: computed 72 s/period; actual ≈ 100 s (36 lifts/h).

### 9c. Test-lift / stop elements (adds to cycle; see Q8b)
- KOSHA A-G-18-2026: stop at ~20 cm on every container pick ("스토핑(Stopping) 작업"), 20–30 cm for OOG/special containers; shift start "2～3회 반복하는 시험운전".
- OSHA 1910.179(n)(3)(vii): near-rated loads → raise "a few inches" and apply brakes.

### 9d. Steel coils (2026-10-20 continuation)
| Item | Value | Source + quote | Conf. |
|---|---|---|---|
| Coil-storage gantry crane: hook closing at pick-up + opening at drop | **35 s per loaded move** (pick + drop combined) | Weckenborg, Oetjegerdes & Spengler, "Energy-oriented crane scheduling in a steel coil storage", *Flexible Services and Manufacturing Journal* 37:97-124 (2025, open access), §6.1: "a loaded move requires an additional closing and opening of the hook for pickup and dropping of the coil"; "we assume the time for pickup and dropping of coils as [δ] = 35 s" (symbol lost in text extraction). https://epub.uni-regensburg.de/78566/1/s10696-024-09534-0.pdf | med (study assumption; settings "derived from applications observed in practice") |
| Same study: crane kinematics & storage geometry | portal 1.667 m/s, trolley 0.833 m/s, hook 0.2 m/s; lifting height 5 m; row pitch 2.2 m, position pitch 0.8 m, layer 1.0 m; portal+trolley simultaneous, no travel while hook moves; coils in 2-layer triangular (saddle) stacks | same §6.1: "velocities are v1 = 1.667 m/s, v2 = 0.833 m/s, and v3 = 0.2 m/s."; "neither portal nor trolley movement may co-occur during hook movements" | med |
| RTG with rotating hook + C-hook, hot-band coils to railcar | 4 coils per railcar in < 30 min → ≈ 7.5 min per coil incl. car spotting/positioning (DERIVED) | Mi-Jack, "Product Spotlight: Specialized Coil Handling Solutions", 2024-11-11 (70-ton Travelift RTG, Ghent KY): "load four steel coils per railcar in under 30 minutes" https://mi-jack.com/specialized-coil-handling-solutions-with-mi-jack-products/ | low-med (vendor) |
DERIVED example (my calc from the study's parameters): lift 4 m at 0.2 m/s = 20 s, lower 4 m = 20 s, travel 10 rows × 2.2 m at 1.667 m/s ≈ 13 s, + 35 s pick/drop → ≈ 90 s per coil move (excluding the empty return).

### 9e. Project / heavy-lift cargo and tandem lifts
| Item | Value | Source + quote | Conf. |
|---|---|---|---|
| 680 t component, ship's-gear tandem lift (2 × 700 t SWL), Port Canaveral | whole heavy-lift operation ≈ 5 days (includes preparation; NOT hook time) | AJOT 2023-10-23: "this component was tandem lifted off the vessel utilising the ship's gear"; "The heavy lift operation took approximately 5 days." https://www.ajot.com/news/gt-usas-canaveral-cargo-terminal-records-heaviest-lift-crane-operations-boosting-capacity-at-its-facility | med (press release) |
| Tandem vs single lift time multipliers | planning ×2–3; lift ×1.5–2 | S.K. Kunatham Group blog, 2026-03-02: "Planning typically requires 2–3 times more preparation."; "The lift itself may take 1.5–2x longer due to the controlled pace required for synchronisation." https://www.skkunathamgroup.com/en/blog/tandem-lift-explained-when-one-crane-isnt-enough | low (vendor blog, no method) |
| Tandem = critical lift when any crane > 75 % of its rating | see Q8c (WorkSafeBC / WorkSafeNB) | — | high |
| Test lift for OOG/special boxes | lift 20–30 cm, hold, then raise slowly on signal; one sling worker fore and aft | see Q8b (KOSHA A-G-18-2026) — duration of the hold not specified | high |
Per-lift rigging/slinging minutes for project cargo, precast or ship blocks: NOT FOUND → Unverified.

### 9f. What the presets can draw on (summary of sourced elements; blank = no source)
| Cargo | Attach | Detach | Test-lift stop | Other per-cycle overhead | Rows |
|---|---|---|---|---|---|
| Container (STS / RTG spreader) | 3 s lock | 5–10 s (incl. set-down) | at ≈0.2 m on every pick (time not given) | ≈30 s dwell/positioning per STS cycle; STS ≈86–102 s/box, RTG ≈184 s/box | 9a, 9b, 8b |
| Steel coil (C-hook / tong, storage gantry) | 35 s pick + drop combined (study assumption) | (incl.) | — | hook 0.2 m/s; coil→railcar ≈7.5 min/coil (vendor) | 9d |
| Precast element | — | — | stop if not released at panel + rigging weight (Qld §14.1.1) | rotation needs aux winch ≥ 75 % of mass (Qld §14.2) | Q4 |
| Project / OOG | — | — | 20–30 cm (KOSHA) | whole 680 t tandem job ≈ 5 days | 9e, 8b |
| Tandem (any cargo) | — | — | — | lift ×1.5–2, planning ×2–3 (low) | 9e |

---------------------------------------------------------------------------------------------------

## Unverified (remembered, not confirmed from an opened source)
Do NOT use these as sourced values in a preset; each needs an opened source first. "Recollection" = model memory, not a citation.

| # | Item | Recollection / lead | Why unverified | Conf. |
|---|---|---|---|---|
| U1 | Busan full/empty split, reefer share, OOG share | no Busan value recalled. Lead: BPA open data "부산항만공사_부산항 컨테이너 수송통계" (data.go.kr 15055478) | seen only as a search hit, not opened | — |
| U2 | Busan 2024 transshipment volume/share | ≈13.5 M TEU / ≈55 % (DERIVED in 2a from the 2025 growth rate); a community post (damoang.net, Perplexity-generated) gave "1,350만 TEU" | BPA 2024 year-end release not opened; post seen only in a search summary | low |
| U3 | Busan New Port STS productivity (moves/h, GMPH) | recollection: ≈25–30 gross moves/h per STS is the range usually quoted for Asian hub terminals | 2023 KINPR papers omit the numbers ("(중략)"); Jeong & Kim 2025 abstract has none; R4 Q4a HKT net cycle (≈40 net moves/h) is the only Busan figure | low |
| U4 | Busan New Port ARMG/ATC and RTG moves/h | recollection: ≈20–25 moves/h per automated yard crane, ≈15–20 per manned RTG | no opened source; only BCT Gdynia RTG ≈184 s/box (≈20 boxes/h) in 9a | low |
| U5 | Busan wind/weather cargo-handling downtime (h or days per year) | no value recalled. Typhoon whole-port closures (태풍 다나스 2013 and 2019: "하역작업도 전면 금지") seen only as search snippets | not opened; no annual statistic found anywhere | — |
| U6 | Terminal-operator internal wind limits (PNC, HJNC, HPNT, BNCT, BCT, DGT …) | recollection: stop at ≈20 m/s gust, i.e., same as KOSHA 2b | no terminal SOP opened | low |
| U7 | Hutchison Busan (HBCT) vessel-rate record | "260 boxes per hour" with four twin-lift QCs (2005) | seen only as a search result title (hutchisonports.com), not opened | low |
| U8 | EN 13001-2 / FEM 1.001 default hoist-load wind area when the load is not specified | recollection: 0.5 m² per t of hoist load with aerodynamic coefficient 2.4 (note 0.5 × 2.4 = 1.2 m²/t, the EN 13000 constant) | standard text not opened; targeted search returned only the EN 13000 1.2 m²/t statements (Q6) | low |
| U9 | Container- or panel-specific c_W | recollection: ≈1.1–1.3 for a box broadside to the wind, i.e., inside Liebherr's generic "Plate / cube 1.1 – 2.0" | no container-specific source opened | low |
| U10 | Precast segmental box-girder SEGMENT weights | recollection: tens of tonnes per segment (≈30–80 t), pier segments heavier | no opened source (girder weights ARE sourced in 4b) | low |
| U11 | Korean PC (프리캐스트) element weights; Korean tilt-up/precast wind rule | not searched (budget) | — | — |
| U12 | Steel slab weights | recollection: cast slabs ≈200–300 mm × 1–2.5 m × 5–12 m → ≈8–70 t (my arithmetic at 7.85 t/m³ on recalled dimensions) | Standard Club gives only the stacking rule (Q3) | low |
| U13 | C-hook / coil-tong self-weight | no value recalled | not found (9d gives only pick + drop time) | — |
| U14 | Goliath (shipyard) in-service wind limit; ship-block weight distribution | no reliable value recalled | not found (Q5) | — |
| U15 | Lightning rule for DG handling | recollection: some terminals suspend DG/fuel work with lightning within ≈10 km, or use the "30-30" rule | no port SOP or statute opened (Q7 found none in Korean law) | low |
| U16 | Test-lift HOLD duration | recollection: lift-plan templates often hold a few minutes at 100–300 mm | KOSHA/OSHA give only the height (Q8b/8d) | low |
| U17 | Rigging/slinging minutes per lift — project cargo, precast, ship blocks | no value recalled | not found (9e) | — |
| U18 | OOG share of container throughput | recollection: well under 1–2 % of container volumes; a forwarder page (SINO Shipping, Danish) reportedly says flat racks/open tops carry 65–70 % of containerized OOG from China | neither opened; no statistic found | low |
| U19 | USACE EM 385-1-1 75 % critical-lift threshold | quoted second-hand by Maxim Crane (Q8c) | EM 385 returned HTTP 403 | med-low |
| U20 | Modern loaded gross-weight distribution for Busan / Asian hubs | only Interpave Table 16 (1970s UK, 1a) is sourced; VGM data (1b) give misdeclaration rates, not a distribution | not found | — |
