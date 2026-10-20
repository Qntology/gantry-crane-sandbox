# R4 — Statistics gaps for gantry-crane site-operation sim (SOURCED)

Status: COMPLETE (2026-10-20). 15 web searches used; the rest are direct fetches/APIs.
Rules: every number = URL actually opened + short verbatim quote (<=25 words) or table ref. Confidence: high / med / low.
"DERIVED" = arithmetic on sourced numbers (not a published figure). Remembered-but-unverified values are only in the last section.

Quick index of the most sim-relevant numbers
| Constant | Value | Conf. | Where |
|---|---|---|---|
| Korea crane deaths / yr (2019–2023) | ~31 (157 in 5 yr) | med | Q1b |
| Korea crane stock (c. 2003) | 46,926 (gantry 2,551) | med | Q1a |
| Fatality rate per crane-year (Korea) | ~6–7e-4 | low (DERIVED) | Q1c |
| STS failures per crane-year (Busan HKT) | 46 (MMBF ≈1,805 moves) | high | Q4 |
| STS MTTR | 0.40 h | high | Q4 |
| STS availability Ai (PM+EM excluded) | 95.0 % (PM 4.73 %, EM 0.25 %) | high | Q4 |
| STS operating time | ≈2,780 h/yr (utilization 33.4 %) | med (DERIVED) | Q1d/Q4 |
| Quay-crane wind operating limit (industry rec.) | 18–19 m/s at crane top | high | Q2/Q3 |
| Gantry brakes hold / rail brakes design | 28 m/s from behind / 35 m/s | high | Q2/Q3 |
| KR law: anti-drift device ON | instantaneous wind > 30 m/s expected | high | Q3d |
| KR law: tower-crane operation stop | instantaneous > 15 m/s | high | Q3d |
| Busan V0 (KDS 2019, 100-yr, 10-min, 10 m, exp. C) | 38 m/s | high | Q3b |
| Busan max gust in Maemi (KMA station) | 42.7 m/s (2003-09-12) | high | Q6 |
| Maemi crane losses, Busan | 8 collapsed + 3 derailed/damaged = 11 | high | Q6 |
| Busan fog days | 14.4 d/yr (1991–2020; fog = vis < 1 km) | high | Q5 |
| Departure-control visibility (KR) | 0.5 km (laden tankers etc.), 1 km (domestic passenger) | high | Q5 |
| Illuminance (KR law) | 750/300/150/75 lx; walkways 75 lx | high | Q7 |

---------------------------------------------------------------------------------------------------
## Q1. Korea crane population, exposure, accident statistics

### 1a. Crane population (stock)
| Item | Value | Source | Quote / ref | Conf. |
|---|---|---|---|---|
| Total cranes held (보유) | 46,926 | 기도형 (계명대), "크레인 사망 재해 실태와 안전 대책", 안전논단, THE SAFETY TECHNOLOGY, Apr 2006, p.14 — https://koreascience.kr/article/JAKO200644271492438.pdf | "보유 중인 전체 크레인 46,926대 중 천장크레인이 26,735대로 전체 57%를 점유" | med (count year not stated, ~2003; scope presumably national) |
| Overhead (천장) | 26,735 (57%) | same | same | med |
| Mobile (이동식) / Hoist | 8,398 (17.8%) / 5,133 (11.1%) | same | "이동식크레인 8,398대(17.8%), 호이스트 5,133대(11.1%)" | med |
| Tower / Gantry | 2,951 (6.3%) / 2,551 (5.4%) | same | "타워크레인 2,951대 (6.3%), 갠트리크레인 2,551대(5.4%)" | med |
| KOSHA inspections in 2006, all hazardous machines | 99,382 units; cranes 49.5% (≈49,200 cranes, DERIVED) | 서울신문 2007-10-29 — https://m.seoul.co.kr/news/2007/10/29/20071029012001 | "검사한 실적은 9만 9382대에 이른다" / "크레인이 49.5%로 가장 많았고" | med (article's shares sum to 102.8%) |

Recent (2020s) 안전검사 대상 크레인 대수 by type: NOT FOUND in any openable source. data.go.kr only has 불합격 정보 / 기관지정 datasets (https://www.data.go.kr/tcs/dss/selectDataSetList.do?keyword=안전검사 크레인).

### 1b. Korean crane accident statistics
| Item | Value | Source | Quote / ref | Conf. |
|---|---|---|---|---|
| Crane deaths 2019–2023 (among 12 hazardous machine types) | 157 of 513 (≈31/yr; largest share) | 전북일보 2025-09-21 — https://jjan.kr/article/20250921580006 | "위험기계 12종 사망사고(513명) 가운데 크레인 사고가 157명으로 가장 큰 비중" | med (agency not named in article) |
| Crane deaths 2012–2017 (6 yrs) | 200 (≈33/yr) | 아시아경제 2017-12-12 citing MP 문진국 — https://www.ajunews.com/view/20171211151248007 | "2012년부터 최근까지 6년간 크레인 사고로 숨진 노동자는 200명이나 된다" | med |
| 2017 YTD | 16 dead, 37 injured | same | "올해에만 관련 사고로 16명이 죽고 37명이 다쳤다" | low (no agency) |
| Tower-crane deaths by year (MOEL) | 2015:1, 2016:5, 2017:10, 2018:0, 2019:1, 2020:3, 2021 (to mid-Oct):5 | 데일리안 2021-10-17 — https://www.dailian.co.kr/news/view/1042666/ | "2015년 1명, 2016년 5명, 2017년 10명, 2018년 0명, 2019년 1명, 작년 3명" | high |
| Crane deaths late 1990s–2002 | < 30 / yr | 기도형 2006 (1a) | "크레인 재해 사망자는 1990년대 말부터 30명을 넘지 않고 있다" | med |
| Crane share of all occupational deaths | 4.1% (1994) → ~0.8% (2002) | same | "1994년에 4.1%를 차지하여 가장 높았고, 2002년에는 약 0.8%" | med |
| Crane as 기인물, 2002 death analysis | 21 of 659 (5.42%), 2nd after scaffolds | same | "크레인 종류가 21건(5.42%)으로 가설 건축 구조물의 비계(22건, 5.67%) 다음으로 높게 나타났다" | med |
| Regional sample: deaths 1998-01 → 2003-07 (Busan/Ulsan/Yangsan/Gyeongnam) | 60 | same | "1998년 1월 1일부터 2003년 7월31일까지 부산, 울산, 양산, 경남 지역에서 발생한 60건의 재해" | high (as sample) |
| ...by crane type | mobile 20 (33%), overhead 14 (23%), gantry 10 (17%), jib 4 (7%), tower 4 (7%), hoist 2 (3%) | same | "이동식크레인이 20명(33%) ... 천장크레인 14명 (23%), 갠트리크레인 10명(17%)" | high (sample) |
| ...fatality share ÷ population share | gantry 3.14 (highest), hoist 0.27 (lowest) | same | "갠트리크레인이 3.14로 가장 높고 ... 호이스트가 0.27로 가장 낮게" | med |
| ...by accident form | falling/flying object 22 (37%), caught-in 18 (30%), fall from height 16 (27%) | same | "낙하∙비래 22명(37%), 협착 18명(30%), 추락 16명(27%)" | high (sample) |
| ...by task | load transport 32 (54%), work at height (repair/inspection/shift change) 26 (43%), other incl. overturn/collapse 2 (3%) | same | "운반작업 32명(54%), 고소작업 26명(43%), 기타 2명(3%)" | high (sample) |
| ...by cause | slinging method 22 (37%), slinging gear 12 (20%), signalman 8 (13%), rope breakage 11%, hook latch 10%, no harness 8% | same | "줄걸이 작업방법 불량 22명(37%), 줄걸이 보조기구 불량 12명(20%)" | high (sample) |
| Construction: mobile crane as 기인물, deaths 2016–2021 (MOEL/KOSHA SIF set) | 61 deaths (of 2,574 construction deaths; 4,432 all-industry) | 최종국·이종우, J. Soc. Disaster Inf. 19(3):719 (2023) — https://www.koreascience.or.kr/article/JAKO202328943546968.pdf | "이동식크레인이 기인물이 되어 발생한 사망재해가 총 61건 발생하였다" | high |
| ...by form | struck-by 24 (39%), fall 13 (21%), crushed-under 11 (18%) | same | "부딪힘이 24건(39%)로 가장 많았으며, 다음으로 떨어짐 13건(21%), 깔림 11건(18%)" | high |
| Mobile crane accidents 2011–2020 (citing 김진환 2021) | 1,009 accidents, 1,028 victims | same | "2011~2020년에 1009건(1028명의 재해자)의 재해가 발생" | med (secondary) |

### 1c. Exposure-based rate (DERIVED — not a published number) [LOW]
- ≈31 crane deaths/yr (2019–2023) ÷ 46,926 cranes (c. 2003 stock) ≈ 6.7e-4 deaths per crane-year; ÷ ≈49,200 KOSHA-inspected cranes (2006) ≈ 6.4e-4. Mixed eras; today's stock is probably larger, so the true rate is likely lower. Also note many mobile/tower cranes are 건설기계 (MOLIT registry), so the 안전검사 population ≠ all cranes.
- Gantry cranes are over-represented in deaths (share ratio 3.14, 1998–2003 sample) → a gantry-specific rate ≈ 3× the all-crane average is the best-supported scaling.
- Severity ratio: no openable official Korean injured:dead split found. Best available: 2017 YTD 16 dead : 37 injured (low). See Unverified for 185:3,556 (2013–2017).

### 1d. Typical annual operating hours
- Only port STS data found (Q4, HKT Busan): utilization 33.38% of available time with Ai 95.02% → ≈ 8,760 × 0.9502 × 0.3338 ≈ 2,780 h/yr mechanism-running time; net hoist/trolley time ≈ 82,272 moves × 1.50 min ≈ 2,060 h/yr (DERIVED, med).
- General industrial (overhead/gantry) crane hours: NOT FOUND.

---------------------------------------------------------------------------------------------------
## Q2. International comparisons

### 2a. US (BLS CFOI)
Source: BLS fact sheet "Fatal occupational injuries involving cranes, 2011–17" — https://www.bls.gov/iif/factsheets/fatal-occupational-injuries-cranes-2011-17.htm
| Item | Value | Quote | Conf. |
|---|---|---|---|
| Crane-related deaths 2011–2017 | 297; avg 42/yr | "297 total crane-related deaths, an average of 42 per year over this 7-year period" | high |
| Struck by object/equipment | 154 (52%) | "Just over half of all fatal crane injuries involved the worker being struck by an object or equipment" | high |
| ...struck by falling object | 91 of 154 | "About three-fifths of these cases (91 of 154) involved the worker being struck by a falling object or equipment" | high |
| ...object falling from / put in motion by crane | 79 | "in 79 of these cases, the worker was struck by an object falling from or put in motion by a crane" | high |
| Industry | construction 43%, manufacturing 24% | "43 percent of fatal work injuries involving cranes took place in the private construction industry" | high |
| Occupation | 1/3 transport & material moving; >half of those crane operators | "One-third of all worker deaths involving cranes in 2011–17 were to workers in transportation and material moving occupations" | high |
No US per-crane or per-hour denominator found.

### 2b. UK (HSE RIDDOR dangerous occurrences)
Source: HSE table RIDDO, published 20 Nov 2025 — https://www.hse.gov.uk/statistics/assets/docs/riddo.xlsx (sheet "Table", DO Paragraph 1)
| GB year | "Collapse, overturning or failure of lifting equipment" (all lifting equipment, not only cranes) |
|---|---|
| 2020/21 | 386 (COVID-affected, Note 4) |
| 2021/22 | 467 (COVID-affected) |
| 2022/23 | 434 |
| 2023/24r | 403 |
| 2024/25p | 409 (provisional) |
Quote: "Collapse, overturning or failure of lifting equipment | 409" (2024/25p row). Conf. high (counts); ≈420/yr mean (DERIVED). No GB crane population denominator found.

### 2c. TT Club / ICHCA / PEMA (port claims)
| Item | Value | Source | Quote | Conf. |
|---|---|---|---|---|
| Quay cranes share of port & terminal claim cost, 2017–2021 (9,323 claims > US$6,000) | 18% (single largest) | TT Club article (webinar May 2022) — https://www.ttclub.com/news-and-resources/events/article/port-and-terminal-risks-quay-container-cranes-1/ ; slides https://www.ttclub.com/media/files/tt-club/webinar-resources/port-and-terminal-risks-quay-container-cranes-slides.pdf | "wharf or quay crane claims are the single largest insurance cost for ports and terminals, accounting for 18% of claim costs" | high |
| Quay-crane claim cost by cause, 2017–2021 | other collisions 30%, stack collisions 19%, weight/twistlock/cell-guide 18%, boom-to-ship 13%, twin-20 11%, other spreader/hoist 7%, wind 0.4% (article text says 4.4%), collapses 0.3% (slide also shows "Other" 2%) | same | "Other collisions (30%) Stack collisions (19%) Weight, twist lock, cell guide issues (18%) Quay crane boom to ship collisions (13%)" | high for top 6; wind value med (slide 0.4% vs article 4.4%; 0.4% makes the pie sum ≈100%) |
| Preventability | 80% | same article | "80% of quay crane accidents could be prevented with better procedures and technology" | med |
| Boom-to-ship cost | up to US$8 m incl. business interruption; example US$2 m repair + US$6 m lost revenue, 6 months | article; and TT Club/ICHCA/PEMA "Recommended Minimum Safety Features for Quay Container Cranes", Oct 2019 — https://www.ttclub.com/media/files/tt-club/port-equipment-information-papers/recommended-minimum-safety-features-for-quay-container-cranes.pdf | "took six months and US$2m to repair and caused a further US$6m in lost revenue" | high |
| Quay crane share of claim cost, 2014–2018 | 24% (earlier 5-yr window: 34% of asset-related claims) | Min. Safety Features (2019) | "identified that 24 per cent of costs are related to quay container cranes" | high |
| 2014–2018 quay-crane causes | values 24/13/13/12/11/10/10/5/2 %; legend: boom-to-ship, other collision, other spreader/hoist, wind-cranes, crane collapses, weight/twistlock, stack collision, twin-20, other. Text confirms boom-to-vessel = largest and structural failure = 3rd largest | same | "Boom-to-vessel collisions are the single largest cost of quay crane claims" / "structural failure is the third biggest insurance claim cost for quay cranes" | med (label→value mapping not fully readable; wind ≈10–13% in this window) |
| Wind | largest preventable weather cost; mostly cranes blown along rails | same | "Wind damage, primarily involving cranes being blown along rails, is the largest preventable weather damage cost" | high |
| Recommended max operating wind | 18–19 m/s (anemometer at crane top); at 90 m wind is 3–4 m/s higher than at 10 m | same | "The recommended maximum operating wind speed should be set at 18 - 19m/s (*) (depending on the design of the crane)" | high |
| Gantry brakes / rail brakes | hold crane at 28 m/s from behind; rail brakes for 35 m/s | same | "designed to stop and hold the crane at a wind speed of 28m/s from behind" ... "rail brakes, which shall be designed for wind speeds of 35m/s" | high |
| Do not auto-shutdown on wind alarm | crane must still travel to storm pins/tie-downs | same | "Crane must not shutdown automatically, even if the alarm sounds continuously" | high |

---------------------------------------------------------------------------------------------------
## Q3. Korean design basic wind speeds + crane storm rules

### 3a. KDS 41 12 00 basic wind speed V0 — definition
| Version | Definition | Source | Quote | Conf. |
|---|---|---|---|---|
| KDS 41 12 00:2019 (rev. 2019-03-14) | 10-min mean, 10 m, exposure C, 100-yr return; Table 5.5-1 / Fig 5.5-1 | KCSC API (backend of https://www.kcsc.re.kr) — https://kcsc.re.kr/api/api/v1/tn-document-contents?doc_info_seq=3193 | "지상 10 m 높이에서 10분간 평균풍속의 재현기간 100년에 대한 값이고 ... 그림 5.5-1 또는 표 5.5-1에 의해 정한다" | high |
| KDS 41 12 00:2022 (rev. 2022-10-11; current, last_yn=Y per https://kcsc.re.kr/api/api/v1/tn_document_infos/by-kcsc_cd?kcsc_cd=KDS411200) | 10-min mean, 10 m, exposure C, **500-yr** return; map only (Fig 5.5-1), no city table; strength design wind load factor 1.0; ASD = ×0.65 | https://kcsc.re.kr/api/api/v1/tn-document-contents?doc_info_seq=5022 | "지상 10m 높이에서 10분간 평균풍속의 재현기간 500년 값으로 하고 ... 그림 5.5-1에 의해 정한다" ; "허용응력설계(재현기간 50년)의 풍하중은 강도설계로 산정한 값에 풍하중계수 0.65를 곱한다" | high |
| (caveat) 2022 §1.4 glossary still says 100-yr | internal inconsistency in the 2022 text | same | "10분간 평균풍속에 대한 재현기간 100년 기대풍속" | high (that the text says so) |
| 2022: between isotachs | use larger | same, Fig 5.5-1 note 2 | "건설지점이 등풍속선 사이에 위치할 때는 인근 등풍속선 중 큰 값을 사용한다" | high |

### 3b. V0 by port city — KDS 41 12 00:2019 Table 5.5-1 (100-yr, m/s) [HIGH]
Source: https://kcsc.re.kr/api/api/v1/tn-document-contents?doc_info_seq=3193, "표 5.5-1 지역별 기본풍속" (note: value = max over the 시/군 from Fig 5.5-1).
| City | V0 | Row quote |
|---|---|---|
| Busan 부산 | 38 | "부산 | 38" |
| Pohang 포항 (+Gyeongju, Gijang, Tongyeong, Geoje) | 36 | "포항, 경주, 기장, 통영, 거제 | 36" |
| Ulsan 울산 (+Yangsan, Gimhae, Namhae, Ulju) | 34 | "양산, 김해, 남해, 울산, 울주 | 34" |
| Changwon 창원 | 30 | "울진, 창원, 사천, 영천 | 30" |
| Yeosu 여수 | 34 | "진도, 여수, 고흥, 신안, 무안, 장흥 | 34" |
| Gwangyang 광양 | 28 | "익산, 김제, 순천, 고창, 광양 | 28" |
| Seosan 서산 (+Taean) | 34 | "서산, 태안 | 34" |
| Dangjin 당진 | 32 | "당진 | 32" |
| Incheon 인천 (+Ganghwa, Ansan, Siheung, Pyeongtaek) | 28 | "인천, 강화, 안산, 시흥, 평택 | 28" |
| Mokpo 목포 | 32 | "목포, 부안, 영암, 강진 | 32" |
| Jeju / Seogwipo | 44 | "서귀포, 제주 | 44" |
| Seoul | 26 | "서울, 김포, ... | 26" |

### 3c. V0 2022 map (500-yr) — approximate reading of the 504×753 px isotach image [LOW]
From doc_info_seq=5022 content #686 (Fig 5.5-1); isotachs 28–50 m/s in 2 m/s steps. Reading: Busan ≈42, Pohang ≈40, Ulsan ≈40, Seosan ≈34–36, Incheon ≈30–32, Yeosu ≈34–36, Jeju 46–50. ≈1.1 × the 100-yr table (consistent with a 100→500-yr Gumbel step). Not official numbers — get the high-res figure before design use.

### 3d. Korean crane wind rules (law text via law.go.kr open API)
Law: 산업안전보건기준에 관한 규칙 (고용노동부령 제450호, 공포 2025-09-01, 시행 2026-03-02) — https://www.law.go.kr/DRF/lawService.do?OC=test&target=law&MST=273603&type=XML
| Rule | Threshold | Quote | Conf. |
|---|---|---|---|
| Art. 37① general stop | qualitative | "비ㆍ눈ㆍ바람 또는 그 밖의 기상상태의 불안정으로 인하여 근로자가 위험해질 우려가 있는 경우 작업을 중지하여야 한다" | high |
| Art. 37② tower crane erection/repair/inspection/dismantling stop | instantaneous wind > 10 m/s | "순간풍속이 초당 10미터를 초과하는 경우 타워크레인의 설치ㆍ수리ㆍ점검 또는 해체 작업을 중지" | high |
| Art. 37② tower crane operation stop (amended 2017-03-03) | instantaneous wind > 15 m/s | "순간풍속이 초당 15미터를 초과하는 경우에는 타워크레인의 운전작업을 중지하여야 한다" | high |
| Art. 140 outdoor travelling crane anti-drift | expected instantaneous wind > 30 m/s | "순간풍속이 초당 30미터를 초과하는 바람이 불어올 우려가 있는 경우 옥외에 설치되어 있는 주행 크레인에 대하여 이탈방지장치를 작동" | high |
| Art. 143 inspection before reuse | after instantaneous wind > 30 m/s or ≥ 중진 quake | "순간풍속이 초당 30미터를 초과하는 바람이 불거나 중진(中震) 이상 진도의 지진이 있은 후에" | high |

Crane design wind — 위험기계·기구 안전인증 고시 (고용노동부 고시 2023-46, 시행 2023-09-01) [별표 2] 크레인 제작 및 안전기준: XML https://www.law.go.kr/DRF/lawService.do?OC=test&target=admrul&ID=2100000228814&type=XML ; 별표 2 PDF https://www.law.go.kr/LSW/flDownload.do?flSeq=132874543
| Clause | Content | Quote | Conf. |
|---|---|---|---|
| No.7 W = qCA; 나1) in service | q = 83.3·h^(1/4) N/m² | "크레인 작동시 83.3 × 4√h (N/㎡)" | high |
| No.7 나2) out of service | q = 9.8·M·h^(1/4) N/m², M = V²/30, V = design basic wind speed per 건축물의 구조기준 등에 관한 규칙 | "크레인 정지시 9.8 × M × 4√h (N/㎡)" ; "설계기본풍속은 「건축법」같은 법 「건축물의 구조기준 등에 관한 규칙」에 따른다" | high |
| h floor | h < 16 m → 16 | "그 높이가 16m 미만일 때는 16으로 함" | high |
| No.38 anti-drift (rail clamp) device | resist W = 120·h^(1/4)·C·A (N) | "옥외에 설치된 주행크레인에는 미끄럼방지를 위한 고정 장치를 설치해야 하며" ... "W = 120 × 4√h × C × A" | high |
| No.39 travel drive | reach anchoring point against 16 m/s | "미끄럼방지 고정 장치가 설치된 위치까지 매초 16m의 풍속을 가진 바람이 불 때에도 주행할 수 있는 출력" | high |
| No.11 다/라 out-of-service stability | unloaded, stopped, worst wind direction: stabilising ≥ overturning moment; travelling cranes need fixing device | "하물을 싣지 않은 정지상태에서 풍하중이 걸렸을 때 ... 안정모멘트 값은 그 전도지점에서 전도 모멘트 값 이상" | high |
| No.27 overload limiter | trips at 1.1× rated (1.05× jib); test wind ≤ 8.3 m/s | "시험시 풍속은 8.3m/s를 초과하지 않을 것" | high |
DERIVED: 83.3·16^0.25 = 166.6 N/m² ≈ ½·1.225·V² → V ≈ 16.5 m/s (in-service basis ≈16 m/s); 120·2 = 240 N/m² → V ≈ 19.8 m/s for the rail-clamp check; out-of-service q at h=16 with V=38 (Busan) = 9.8·48.1·2 ≈ 944 N/m².
Post-Maemi port-crane-specific Korean regulation: NOT FOUND (only press report of a Busan upgrade, see Q6).

---------------------------------------------------------------------------------------------------
## Q4. Port crane reliability

### 4a. STS — Hutchison Korea Terminal (Busan), 14 STS cranes, 2013–2018 [HIGH for this terminal]
Source: Jo & Kim, "Key Performance Indicator Development for Ship-to-Shore Crane Performance Assessment in Container Terminal Operations", J. Mar. Sci. Eng. 2020, 8(1), 6 — https://mdpi-res.com/d_attachment/jmse/jmse-08-00006/article_deploy/jmse-08-00006.pdf , Table 2 "Crane performance assessment of Hutchison Korea Terminal (2013~2018)" (per-crane annual averages; Tmc = 24 h/day).
| Year | TLC (moves) | Breakdowns | Util. % | Ai % | Ao % | PM % | EM % | Total downtime % | Net cycle (min) | MMBF | MTTR (h) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2013 | 67,549 | 35 | 27.41 | 95.11 | 99.56 | 4.71 | 0.18 | 4.89 | 1.50 | 1,926 | 0.377 |
| 2014 | 73,840 | 38 | 29.60 | 94.40 | 99.52 | 5.39 | 0.21 | 5.60 | 1.46 | 1,954 | 0.405 |
| 2015 | 82,626 | 42 | 32.57 | 94.55 | 99.54 | 5.22 | 0.23 | 5.45 | 1.44 | 1,971 | 0.392 |
| 2016 | 87,442 | 49 | 35.07 | 95.64 | 99.50 | 4.09 | 0.27 | 4.36 | 1.49 | 1,774 | 0.394 |
| 2017 | 95,241 | 57 | 39.07 | 94.99 | 99.97 | 4.68 | 0.33 | 5.01 | 1.52 | 1,680 | 0.421 |
| 2018 | 86,933 | 53 | 36.53 | 95.44 | 99.49 | 4.27 | 0.30 | 4.56 | 1.56 | 1,651 | 0.408 |
| Avg | 82,272 | 46 | 33.38 | 95.02 | 99.60 | 4.73 | 0.25 | 4.98 | 1.50 | 1,805 | 0.401 |
Definitions quoted: "MMBF: Mean movements between failure (calculation formula: TLC/Fem)"; "MTTR: Mean time to repair (calculation formula: Tem/Fem)"; "Ai: Overall availability (calculation formula: ((Tmc - Tem - Tps)/Tmc) × 100%)". Trend: "the aging of the cranes caused problems gradually" (MMBF falling, MTTR rising).
DERIVED: calendar-time MTBF ≈ 8,760/46 ≈ 190 h; EM 0.25% ≈ 22 h/yr unplanned repair; PM 4.73% ≈ 414 h/yr planned.

### 4b. Spreaders (vendor-reported)
Source: Port Strategy, "The Point of Contact" (May 2005), quoting Bromma at TOC Asia — https://www.portstrategy.com/?p=210818
| Item | Value | Quote | Conf. |
|---|---|---|---|
| Spreader MMBF (one customer) | ≈44,000 moves | "they average some 44,000 moves between failure" | med (single customer) |
| Mean time to diagnose & repair | 18 min | "their mean time for spreader diagnosis and repair – was 18 minutes" | med |
| Inherent spreader availability | 96% | "their inherent spreader availability was 96%" | med |
| Mechanical vs electrical share of spreader downtime events | mech 5–82%, elec 14–57% across terminals | "at one terminal mechanical problems represent 82% of downtime events, while at another terminal it is 5%" | high (that variation is huge — no universal split) |

### 4c. RTG / QC case studies (Indonesia) [LOW — single terminals]
| Item | Value | Source | Quote | Conf. |
|---|---|---|---|---|
| RTG downtime by subsystem (diesel RTGs, "PT XYZ") | engine 67%, spreader 15% | Adelencia et al., Logistik 18(1) 2025, doi 10.21009/logistik.v18i01.51280 — https://journal.unj.ac.id/unj/index.php/logistik/article/view/51280 | "kontribusi downtime sebesar 67%" / "diikuti oleh Spreader System sebesar 15%" | low |
| RTG unit availability spread (TPKS Semarang) | 0.312 (worst unit-month) to >0.95; best monthly MTBF 614 h | Watupongoh & Rusmiyanto, Ocean Engineering 3(4) 2024 (abstract via https://diajeng.lldikti6.id/articles/21928) | "RTG035-13, which only reached 0.312 in February" / "highest MTBF of 614 hours in December" | low |
| QC FMEA (Tanjung Perak, 2018–19) | highest RPN: main trolley travel 450; gantry 360; power supply 294; main hoist 280; boom hoist 42 | Ponidi & Bambang, J. Rekayasa Material Manufaktur dan Energi 3(2) — https://jurnal.umsu.ac.id/index.php/RMME/article/view/5268 | "the largest Risk Priority Number (RPN) is obtained on the main trolley travel system of 450" | low |
RMG/ARMG MTBF/availability: NOT FOUND in an openable source.

---------------------------------------------------------------------------------------------------
## Q5. Visibility / fog thresholds

| Item | Value | Source | Quote | Conf. |
|---|---|---|---|---|
| KR departure control, non-passenger ships, restricted visibility | 시정 ≤ 0.5 km: laden oil/gas/chemical tankers (unless escorted) and ships without radar+VHF | 해상교통안전법 시행규칙 [별표 10] 선박 출항통제의 기준 및 절차 (해양수산부령 제734호, 시행 2025-05-26) — https://www.law.go.kr/DRF/lawService.do?OC=test&target=law&MST=271375&type=XML | table 2.다 row "제한된 시계 / 시정 0.5킬로미터 이내": "1) 화물을 적재한 유조선ㆍ가스운반선 또는 화학제품운반선[향도선(嚮導船)을 활용하는 경우는 제외한다] 2) 레이더 및 초단파 무선전화(VHF) 통신설비를 갖추지 않은 선박" (cells re-joined) | high |
| KR departure control, domestic passenger ships | 시정 ≤ 1 km (WIG passenger craft ≤ 11 km) | same | "시정 1킬로 미터 이내 │모든 내항여객선" ; "출항통제권자는 시계제한 시 해당 내항여객선의 출항을 통제해야 한다" | high |
| Busan port navigation rule (restricted visibility) | no numeric limit; safe speed only | 부산항 항법 등에 관한 규칙 (부산지방해양수산청 고시 2026-124, 시행 2026-07-01) — https://www.law.go.kr/DRF/lawService.do?OC=test&target=admrul&ID=2100000281634&type=XML | "시계(視界) 제한이 있을 때 모든 선박은 다른 선박의 운항에 위험을 미치지 아니할 정도의 안전한 속력으로 운항해야 한다" | high |
| Crane/cargo-handling stop for fog (KR law) | no numeric threshold; only Art. 37① qualitative stop (Q3d). 항만안전특별법/령/규칙 (시행 2026-09-11) contain no wind/visibility criteria (searched XML for 시정/풍속/안개/기상) | law.go.kr MST 283835 / 289505 / 289507 | — | high (absence) |
| KMA fog definition | horizontal visibility < 1 km; fog day = observer-recorded | KMA 기후통계 안개일수 — https://data.kma.go.kr/climate/fog/selectFogChart.do?pgmNo=706 | "안개는 극히 작은 물방울들이 대기 중에 떠있어서 수평시정이 1 km 미만인 현상을 말하며" | high |

Fog days per year (KMA station data, annual totals averaged 1991–2020; same URL, station query) [HIGH unless noted]
| Station | Mean 1991–2020 | Mean 2011–2020 | Min–max year | Peak months (avg days) |
|---|---|---|---|---|
| Busan 159 | 14.4 | 13.6 | 2–33 | May 2.6, Jun 3.9, Jul 4.2 |
| Incheon 112 | 47.5 | 47.6 | 28–65 | Jul 9.1 |
| Ulsan 152 | 5.4 | 4.7 | 0–19 | Jun 1.4 |
| Pohang 138 | 2.8 | 1.2 | 0–11 | Jun 0.7 |
| Yeosu 168 | 19.8 | 18.7 | 4–34 | Jul 5.6 |
| Seosan 129 | 36.6 | 30.3 | 0–68 | Jul 4.7 (med: not on KMA's 2024 eye-observation list; some years may be partial) |
| Changwon 155 | 6.1 | 9.2 | 0–21 | — |
Busan port fog closure HOURS per year: NOT FOUND.

---------------------------------------------------------------------------------------------------
## Q6. Typhoon Maemi (2003-09-12) at Busan

| Item | Value | Source | Quote | Conf. |
|---|---|---|---|---|
| Crane losses by terminal | Jaseongdae: 2 destroyed + 3 derailed; Singamman: 6 crushed | Busan City news 2004-03-15 (from Busan Ilbo 2004-03-12) — https://www.busan.go.kr/eng/bsnews01/798208 | "destroyed 2 cranes and derailed 3 at Jaseongdae Container Terminal, while crushing six cranes at Singamman Container Terminal" | high |
| Totals | 8 collapsed + 3 damaged (=11) | Korea JoongAng Daily 2005-09-07 — https://www.koreajoongangdaily.com/korea/yearold-lessons-made-south-ready-for-nabi/11387846 | "Eight cranes collapsed and three were damaged, forcing the port to close temporarily." | high |
| Singamman / Jaseongdae detail | 6 of 7 (Singamman, Hanjin-built); 2 of 12 (Jaseongdae, Daewoo-built 2001) | Korea JoongAng Daily 2003-09-25 — https://www.koreajoongangdaily.com/business/shoddy-work-suspected-in-fallen-port-cranes/11210735 | "Six of the seven cranes at Singamman quay at Busan Port collapsed" / "Two of the 12 cranes at Jaseongdae quay were leveled" | high |
| Original design wind (builder claim) | 50 m/s | same | "winds of up to 50 meters per second (112 miles per hour)" | med |
| Wind at the piers (literature) | ≈50 m/s | 권순규·이성욱·한동섭·한근조 (동아대), 한국정밀공학회 2006 추계학술대회 — https://www.koreascience.or.kr/article/CFKO200617054768283.pdf | "신감만 및 자성대 부두에서 초속 50m에 이르는 강풍에 의해 총 11기의 컨테이너 크레인이 파손" | med |
| KMA Busan station max gust | 42.7 m/s on 2003-09-12 (2nd highest on record; record 43.0 on 1987-08-31) | KMA 기후극값 (AJAX selectExtremumAjaxList.do, schElem=2) — https://data.kma.go.kr/climate/extremum/selectExtremumList.do?pgmNo=103 | ranked list: "1:43.0@1987-08-31 ; 2:42.7@2003-09-12" | high |
| Same day, other stations | Yeosu 49.2 (station record; 10-min max 35.9), Changwon 38.8, Ulsan 33.2 m/s | same | "1:49.2@2003-09-12" (Yeosu) | high |
| Maemi peak gust nationally | 59.7 m/s at Jeju Gosan | 김종락 (숭실대), "태풍 피해의 교훈과 내풍설계", 특집, 2004-09 제11권 3호 — https://koreascience.kr/article/JAKO200462680274196.pdf | "제주고산 관측소 등에서 59.7m/s의 최대순간풍속으로 지금까지의 기록을 갱신" | med |
| Mechanism | tie-down tension grows faster than structural stress; small wind excess → fatal tie-down overload; recommend higher SF for mooring devices than structure | 김동현 et al., "Collapse of container cranes at Busan ports under typhoon Maemi", ISOPE 2004 pp.683–686 (KIOST record) — https://sciwatch.kiost.ac.kr/handle/2020.kiost/31852 | "very small amount of excessive wind speed may cause fatal increases of tension at tie-downs" / "safety factor for mooring devices should be larger than that for structural members" | high |
| Liability finding | not force majeure; concurrent negligence of crane maker and pier (tie-down/quay) builder; terminal's own share cut 20%→10% on appeal | 법무법인 율촌 case note — https://www.yulchon.com/ko/resources/deals-and-cases-view/1078/page.do | "크레인을 제작한 회사 및 부두시설을 시공한 회사의 과실이 경합하여 발생한 사고" | med |
| Recovery time | ≈6 months (replacement cranes installed Mar 2004; original estimate ≥1 yr) | Busan City 2004-03-15 (above) | "The re-installation will mark the end of the 6-month efforts" | high |
| Post-Maemi design change (Busan cranes) | 179 km/h (≈49.7 m/s) → 270 km/h (75 m/s) | Korea JoongAng Daily 2005-09-07 (above) | "originally designed to withstand winds of 179 kilometers ... but were upgraded to resist winds of up to 270 kilometers per hour" | med (press; no regulation text found) |
| Researchers' design-wind assumption after Maemi | 75 m/s instantaneous | 권순규 et al. 2006 (above) | "본 연구에서는 순간 최대 75m/s의 풍속이 컨테이너 크레인에 작용할 때를 가정" | med |

---------------------------------------------------------------------------------------------------
## Q7. Lighting (산업안전보건기준에 관한 규칙) — VERIFIED
Source: same law URL as Q3d (MST=273603, 시행 2026-03-02).
| Article | Requirement | Quote | Conf. |
|---|---|---|---|
| Art. 8 조도 (work surface of regular workplaces; excludes mines and photosensitive-material workplaces) | ultra-precision ≥750 lx; precision ≥300 lx; ordinary ≥150 lx; other ≥75 lx | "초정밀작업: 750럭스(lux) 이상 ... 정밀작업: 300럭스 이상 ... 보통작업: 150럭스 이상 ... 그 밖의 작업: 75럭스 이상" | high |
| Art. 21 통로의 조명 (walkways) | ≥ 75 lx | "통로에 75럭스 이상의 채광 또는 조명시설을 하여야 한다" | high |
No Korean night-shift/fatigue rule specific to crane operators found (not searched in depth).

---------------------------------------------------------------------------------------------------
## Gaps (searched, not found in an openable source)
- Recent (2015+) KOSHA 안전검사 대상 크레인 대수 by type; official yearly crane injured counts (MOEL 산업재해현황분석 기인물 table).
- Per-crane-hour accident rates (KR, UK, US); US/UK crane population denominators.
- RMG/ARMG/ASC availability and MTBF; electrical vs mechanical vs hydraulic vs structural failure shares for STS (only spreader-level spread in 4b).
- Busan port fog closure hours per year; terminal-level fog/visibility stop criteria for crane operation.
- Korean regulation raising port-crane design wind after Maemi (only press report).

## Unverified (remembered or seen only in search snippets — do NOT use as sourced)
- 2013–2017 Korea crane accidents: 185 deaths, 3,556 victims (울산대 thesis, https://oak.ulsan.ac.kr/handle/2021.oak/10122 — page is JS/403-blocked; only a search snippet). Would give a death:victim ratio ≈ 5.2%.
- "13 container cranes" broken by Maemi (search snippet, not opened; conflicts with verified 11).
- ~3,500 installed tower cranes in Korea (related-article teaser on dailian.co.kr; not opened).
- Heinrich 1:29:300 injury pyramid (generic textbook ratio; not crane- or Korea-specific).
