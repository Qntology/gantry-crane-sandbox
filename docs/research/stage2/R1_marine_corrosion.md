# R1 — Marine (salt-induced) atmospheric corrosion of crane steel vs. distance to sea

Status: IN PROGRESS (written incrementally). Every number carries the URL actually opened + short quote/table ref.
Confidence tags: [high] = read in primary/official source or reproduced verbatim in peer-reviewed OA paper; [med] = secondary source / summary; [low] = indirect or partially verified.

Key OA review used throughout (opened via Europe PMC full text, MDPI itself returns 403 to the fetcher):
- **[ALC17]** Alcántara J., Chico B., Díaz I., de la Fuente D., Morcillo M., "Marine Atmospheric Corrosion of Carbon Steel: A Review", *Materials* 2017, 10(4), 406. PMC5506973.
  URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC5506973/fullTextXML (article: https://www.mdpi.com/1996-1944/10/4/406)

## Q1. ISO 9223:2012 dose–response functions and categories

### Q1a. DRFs (first-year corrosion rate r_corr in µm/a) [high — reproduced identically by two independent peer-reviewed sources]
Sources:
- **[OHA18]** Ohara S., Kosaki A., Shumuta Y., Murao N., "Evaluation of the Applicability of Dose-response Functions to the Data of Outdoor Exposure Tests for Carbon Steel and Zinc in Japan", *Zairyo-to-Kankyo (Corrosion Engineering)* 67 (2018) 416–425, doi:10.3323/jcorr.67.416. URL opened: https://www.jstage.jst.go.jp/article/jcorr/67/10/67_416/_pdf/-char/en (Eqs. 1–2, Tables 2–4).
- **[PAN17]** Panchenko Y.M., Marshakov A.I., "Prediction of First-Year Corrosion Losses of Carbon Steel and Zinc in Continental Regions", *Materials* 2017, 10, 422 (PMC5507004). URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC5507004/fullTextXML (Eqs. 3–4).

Carbon steel (PAN17 Eq. 3 verbatim: "K1 = 1.77 × Pd^0.52 × exp(0.020 × RH + fSt) + 0.102 × Sd^0.62 × exp(0.033 × RH + 0.040 × T)"):

    r_corr,St = 1.77·Pd^0.52·exp(0.020·RH + f_St) + 0.102·Sd^0.62·exp(0.033·RH + 0.040·T)
    f_St = 0.150·(T − 10)   for T ≤ 10 °C
    f_St = −0.054·(T − 10)  for T > 10 °C

Zinc (PAN17 Eq. 4 verbatim: "K1 = 0.0129 × Pd^0.44 × exp(0.046 × RH + fZn) + 0.0175 × Sd^0.57 × exp(0.008 × RH + 0.085 × T)"):

    r_corr,Zn = 0.0129·Pd^0.44·exp(0.046·RH + f_Zn) + 0.0175·Sd^0.57·exp(0.008·RH + 0.085·T)
    f_Zn = 0.038·(T − 10)   for T ≤ 10 °C
    f_Zn = −0.071·(T − 10)  for T > 10 °C

Variables (PAN17): "Pd and Sd are SO2 and Cl− deposition rates expressed in mg/(m2day)", T = annual mean air temperature (°C), RH = annual mean relative humidity (%).
OHA18 Table 4 lists the same coefficients b1…b8 (steel: 1.77, 0.52, 0.020, 0.150/−0.054, 0.102, 0.62, 0.033, 0.040; zinc: 0.0129, 0.44, 0.046, 0.038/−0.071, 0.0175, 0.57, 0.008, 0.085).

**Validity ranges of the ISO 9223 DRFs** (OHA18 Table 3, column "Interval for the ISO9223DRF"): **T −17.1 to 28.7 °C; RH 34 to 93 %; Pd 0.7 to 150.4 mg m⁻² d⁻¹; Sd 0.4 to 760.5 mg m⁻² d⁻¹.** [high]
Uncertainty (OHA18 introduction, citing ISO 9223): corrosion-rate "determination" by 1-yr exposure has ±2 to ±5 % uncertainty while DRF "estimation" has **−33 % to +50 %** uncertainty. [high]

Japanese refit (OHA18 Table 4, "fitted DRF", JWTC data, single temperature branch): steel b1..b8 = 1.26, 0.34, 0.028, 0.085, 0.004, 0.61, 0.089, −0.099; zinc = 0.0564, 0.29, 0.032, 0.038, 0.0287, 0.86, −0.007, −0.041. Fitted-data ranges (OHA18 Table 3): T 6.4–23.5 °C, RH 58–80 %, Pd 1.1–8.2, Sd 1.3–129.4 mg m⁻² d⁻¹, measured steel 10.0–48.0 µm/a, zinc 0.48–2.20 µm/a. OHA18 abstract: ISO DRF "shows insufficient agreement with the measured one from the JWTC data". [high]

Korean calibration (13 ROK Air Force bases): Choi T. et al., "Physics-Informed, Data-Driven Model for Atmospheric Corrosion of Carbon Steel Using Bayesian Network", *Materials* 2023, 16, 5326 (PMC10419963). URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC10419963/fullTextXML. Calibrated model Eq. (11): **r [µm/y] = 0.3219·Sd^0.4411·exp(3.453·TOW)** (Eq. 10 same in g/m²·y with 0.04082), Sd in mg/m²·d, TOW = annual fraction; Pearson r = 0.9 between chloride deposition and corrosion rate, 0.6 for TOW. Note: the paper's "ISO 9223" Eq. (9) does NOT match the ISO 9223:2012 DRF above — do not use it. [med]

Copper / aluminium DRFs: not verified online (see Unverified).

### Q1b. SO2 deposition vs concentration
- PAN17 states (citing ISO 9223): "Pd,p mg/(m2·day) = 0.67 Pd,c µg/m3" (factor **0.67**). [med — single secondary source; the ISO 9223:2012 P-class table boundaries (below, if verified) imply ≈0.8; see Unverified]

### Q1c. Corrosivity categories (ISO 9223:2012 Table 2, first-year corrosion rate r_corr)
Sources: **[PAT24]** Paterlini L. et al., "Atmospheric Corrosion of Different Steel Types in Urban and Marine Exposure", *Materials* 2024, 17, 6211 (PMC11679332), Table 1 — URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC11679332/fullTextXML ; **[TID]** J. Tidblad (ISO TC156/WG4), "Classification — Outdoor (ISO 9223-9226) and indoor (ISO 11844) classification", presentation, URL opened: https://www.rrcap.ait.ac.th/male/Meeting%20Document/14%20JT%20Classification.pdf (draft of the 2012 revision; table identical to PAT24 for steel and zinc).

| Cat. | Corrosivity | Carbon steel g/(m²·a) | Carbon steel µm/a | Zinc g/(m²·a) | Zinc µm/a | Copper g/(m²·a) | Copper µm/a | Aluminium g/(m²·a) |
|---|---|---|---|---|---|---|---|---|
| C1 | very low | ≤ 10 | ≤ 1.3 | ≤ 0.7 | ≤ 0.1 | ≤ 0.9 | ≤ 0.1 | negligible |
| C2 | low | 10–200 | 1.3–25 | 0.7–5 | 0.1–0.7 | 0.9–5 | 0.1–0.6 | ≤ 0.6 |
| C3 | medium | 200–400 | 25–50 | 5–15 | 0.7–2.1 | 5–12 | 0.6–1.3 | 0.6–2 |
| C4 | high | 400–650 | 50–80 | 15–30 | 2.1–4.2 | 12–25 | 1.3–2.8 | 2–5 |
| C5 | very high | 650–1500 | 80–200 | 30–60 | 4.2–8.4 | 25–50 | 2.8–5.6 | 5–10 |
| CX | extreme | 1500–5500 | 200–700 | 60–180 | 8.4–25 | 50–90 | 5.6–10 | > 10 |

Steel & zinc [high — PAT24 + TID agree]; copper & aluminium [med — TID draft-presentation only]. TID note 5: "Corrosivity category CX refers to marine and marine/industrial environments." Each lower bound is exclusive (e.g. 10 < r ≤ 200).
TID also reproduces the DRF parameter intervals "Temperature −17.1 – 28.7 °C, RH 34 – 93 %, SO2 0.7 – 150.4, Cl⁻ 0.4 – 760.5 mg m⁻² day⁻¹" (same as OHA18).

### Q1d. Classes of chloride (S), SO₂ (P), time of wetness (τ)
- Chloride deposition (Heredia A.A. et al., *Heliyon* 2025, e41281, PMC11730541, §3.1 — URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC11730541/fullTextXML): "δ0 corresponds to deposition rates δd < 3 mg/m2 per day", then 3–60, 60–300, 300–1500 mg/(m²·d) → **S0 ≤ 3; S1 3–60; S2 60–300; S3 300–1500 mg Cl⁻/(m²·d)**. Consistent with Chico et al. 2017 (PMC5553418 §2.4): "the 0–3 mg Cl−/m2.d range (level S0)" and ">3 mg/m2.d (lower level S1 according to standard ISO 9223)"; PAT24: Bonassola 209 mg/(m²·d) = "level S2". [high]
- SO₂ deposition: P0 "Pd ≤ 4 mg/m2 - day" and P1 "4 < Pd ≤ 24 mg/m2 - day" (Heredia 2025); Chico 2017: "0–4 mg SO2/m2.d range (level P0)". P2/P3 limits and concentration equivalents: see Unverified. [high for P0/P1]
- TOW: τ4 "2500 < τ < 5500 h/a" (Vera R. et al., *Heliyon* 2023, e17811, PMC10362324, §3.1.2 — URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC10362324/fullTextXML); τ5 "5500 h/y < τ" (Heredia 2025). τ1–τ3 limits: see Unverified. TOW definition (ISO 9223, via ALC17 §9.5.1): time when "the RH ... is equal to or greater than 80% at temperatures above 0 °C". [high for τ4/τ5]
- Informative typical environments (TID, Table C.1 draft): C3 "coastal areas with low deposition of chlorides"; C4 "coastal areas, without spray of salt water"; C5–CX "coastal and off shore areas, with salt spray". NOTE 1: chloride deposition "strongly dependent on ... wind direction, win[d velocity, local] topography, wind sheltering islands outside the coast, distance of the site from the sea". NOTE 3: "Sheltered surfaces in marine atmospheric environment where chlorides are deposited can experience a higher corrosivity category due to the presence of hy[groscopic] salts". [med — draft text]
- Chloride measurement: ISO 9225 wet candle (and dry plate). Wet candle vs dry plate (Corvo, via ALC17 Eq. 13): "[Cl⁻]wc = −54.5 + 1.6 [Cl⁻]dc" (mg/m²·d) at high salinity; "the wet candle method is more sensitive to the presence of NaCl". [high]

## Q2. ISO 9224:2012 long-term exponents
Source: **[ISO9224]** ISO 9224:2012(E) preview (iTeh) — URL opened: https://cdn.standards.iteh.ai/samples/53500/48d0ad5886f74a05a0d0faf4a71da09e/ISO-9224-2012.pdf (text extracted locally). Cross-check: TID presentation (same values). [high]

- Eq. (1): **D = r_corr · t^b** — "t is the exposure time, expressed in years; rcorr is the corrosion rate experienced in the first year ... b is the metal-environment-specific time exponent, usually less than 1."
- Table 2 "Time exponent values for predicting and estimating corrosion attack":

| Metal | B1 (average) | B2 (upper, +2σ) |
|---|---|---|
| Carbon steel | 0.523 | 0.575 |
| Zinc | 0.813 | 0.873 |
| Copper | 0.667 | 0.726 |
| Aluminium | 0.728 | 0.807 |

- "The B1 values were taken as the average time exponents from regression analyses of the flat panel long-term results of the ISO CORRAG atmospheric exposure programme"; "The B2 values in Table 2 include the two standard deviation additions and may be used where an upper limit of corrosion attack is desired". Standard deviations of b: "Carbon steel: 0,026 0; Zinc: 0,030 0".
- If composition unknown: "select the B1 value from Table 2"; "In cases where long-term metal loss data are available, use the b value from this data."
- **t > 20 years (Clause 7):** "Equation (1) has been observed to be valid for exposures of up to 20 years' duration"; "an approach that yields the maximum estimate of attack is to assume that the corrosion rate becomes linear at 20 years of exposure": Eq. (2) dD/dt = b·r_corr·t^(b−1); Eq. (3) **D(t > 20) = r_corr·[20^b + b·20^(b−1)·(t − 20)]**. Table 3 values of t^b (e.g. steel B1: t=10 → 3.334, t=20 → 4.791).
  (PAT24 paraphrase: "after the 20th year of exposure, the corrosion rate is to be considered constant".)

Published long-term marine exponents:
- ALC17 Table 6 (see below): marine mean n = 0.73 (range 0.37–0.98) vs non-marine 0.49.
- Rapa Nui (Chile), 500 m from sea, Cl⁻ ≈ 51 mg/(m²·d), 3 yr: carbon steel n = 0.4877 ("close to 0.5"), galvanized n = 0.8248 (Vera et al. 2023, PMC10362324, Eqs. 2–3). [high]
- Japan-type refits / Korea: see Q1a.

ALC17 Table 6 "Average time exponent n for plain carbon steel" (power law C = A·t^n) [high, review table]:

| Atmosphere | mean n | range |
|---|---|---|
| Non-marine (rural, urban, industrial) | 0.49 | 0.26–0.76 |
| Marine | 0.73 | 0.37–0.98 |

Quotes (ALC17 §9.3–9.4): "n values of more than 0.5 arise due to acceleration of the diffusion process" ... "typical of marine atmospheres, even those with low Cl⁻ contents"; for low-SO₂ marine sites n "can acquire values of more than unity".

## Q3. Airborne chloride vs distance / wind / height

### Q3-KR. Korea — airborne sea salt vs distance by coast (KICT/Hanyang, 1-yr monthly survey) [high — read from the scanned paper's figures]
Source: 이종석·최원성·문한영 (Lee J.S., Choi W.S., Moon H.Y.), "해안으로부터 거리별 비래염분량의 분포에 대한 고찰 / A study on the distribution of airborne sea salt to the distance from seashore", Proc. Korea Concrete Institute Spring Conf. 2004, pp. 842–845.
URL opened: https://www.koreascience.or.kr/article/CFKO200411722756623.page (abstract) and PDF https://koreascience.kr/article/CFKO200411722756623.pdf (scanned; pages rendered locally and read).
- Abstract quote: "airborne sea salt was investigated ... at 33 spots, 6 areas in East, West, South coast for 1 year. Results indicated that airborne sea salt is decreased by y = a·x^−b equation".
- Method (§2.1–2.3): K3-type dry-gauze collector (opening 10 cm × 10 cm, after JIS Z 2382 dry-gauze and PWRI stainless collectors) mounted at **2 m height**; monthly 1-month cumulative samples, **June 2002 – May 2003**; Cl⁻ by AgNO₃ precipitation titration; unit **NaCl mg/(100 cm²·day) = mdd (NaCl)**.
- Fitted power laws (annual means; y in mg NaCl/(100 cm²·d), x = distance from shoreline in m), read from Figs. 3–5:

| Coast | Site | Distances sampled (m) | Fit (annual mean) | R |
|---|---|---|---|---|
| East | Sokcho 속초 | 5, 40, 200, 500, 1000, 2000 | y = 13.874·x^−0.3016 | 0.98 |
| East | Uljin 울진 | 30, 70, 130, 200, 500 | y = 296.31·x^−0.844 | 0.99 |
| West | Gochang 고창 | 5, 60, 120, 200, 500 | y = 48.628·x^−0.5081 | 0.93 |
| West | Byeonsan 변산 | 5, 60, 120, 200, 500 | y = 11.974·x^−0.3728 | 0.91 |
| South | Jangheung 장흥 | 10, 60, 120, 200, 460, 1000, 2000 | y = 1.5296·x^−0.0995 (typhoon month excluded) | 0.84 |
| South | Sacheon 사천 | 30, 60, 120, 230, 430 | non-typhoon: y = 5.5421·x^−0.4014; typhoon month: y = −11.513·ln(x) + 74.149 | 0.96 / 0.98 |

  (Sokcho June–Sept data lost to typhoon damage.) Unit conversion for the model: 1 mdd NaCl = 100 mg NaCl/(m²·d) = 60.7 mg Cl⁻/(m²·d) (Cl/NaCl mass ratio 35.45/58.44).
- Conclusions (§4, translated): East coast airborne salt drops sharply up to **~200 m**, West coast up to **~60 m** (then roughly flat 200–500 m); South coast decays gently overall, but in **typhoon months it drops almost linearly and steeply out to 500 m**. Using the Japanese PWRI damage criterion of **≥ 1 mg/(100 cm²·d)** (1 mdd), "salt-damage-risk zone" would be within **100 m** on the South coast but extend to **2000 m** at Sokcho (East coast).
- Background cited in the paper: Japan's PWRI measured airborne salt at **266 sites nationwide for 3 years from 1984** to zone weathering-steel bridges and concrete salt-damage areas (土木研究所資料 第3175号, 1993, "飛来塩分量全国調査(IV)—飛来塩分量の分布特性と風の関係").

### Q3-KR2. Korea — airborne salt at 100 m from shoreline, 27 coastal areas, 3-yr monthly survey (KICT) [high — read from scanned table]
Source: 이종석·최원성·안기홍·이장화 (Lee J.S., Choi W.S., Ahn K.H., Lee J.H., KICT), "우리나라 해안의 비래염분에 대한 지역계수 산정 / Area index of airborne chlorides at the seashore in Korea", Proc. Korea Concrete Institute Autumn Conf. 2006, pp. 609–612.
URL opened: https://koreascience.kr/article/CFKO200625522717202.pub?lang=ko and PDF https://koreascience.kr/article/CFKO200625522717202.pdf (scanned, rendered locally).
- Abstract quote: "Salinity measurement devices were installed at 27 areas along with seashore throughout the country ... In Gochang and Taean area showed higher value of area index over 10. In addition, the Eastern coast showed higher index compared to the Western and Southern coast."
- Method: same K3 dry-gauze box collector; East 9 / West 10 / South 8 areas ≈ 50 km apart; ≥2 collectors per area at **5–30 m and 50–200 m** from shore (some to 500 m / 2000 m); monthly, **3 years** (other KICT abstract gives Jul 2003 – Jun 2006); all values converted to the **100 m point** using the distance-decay rates of the 2004 study (Eqs. 1–2).
- **Table 1 — airborne salt at 100 m from shoreline, mdd = NaCl mg/(100 cm²·day):**

| East coast | mdd | West coast | mdd | South coast | mdd |
|---|---|---|---|---|---|
| Goseong 고성 | 4.3 | Ganghwa-1 강화1 | 1.0 | Haenam 해남 | 2.0 |
| Sokcho 속초 | 2.4 | Ganghwa-2 강화2 | 0.8 | Jangheung 장흥 | 1.2 |
| Jumunjin 주문진 | 7.7 | Incheon 인천 | 1.1 | Goheung 고흥 | 0.7 |
| Samcheok 삼척 | 6.7 | Asan Bay 아산만 | 1.0 | Gwangyang 광양 | 1.2 |
| Uljin 울진 | 5.1 | Taean 태안 | 16.0 | Sacheon 사천 | 1.6 |
| Yeongdeok 영덕 | 3.0 | Cheonsu Bay 천수만 | 0.9 | Geoje 거제 | 3.5 |
| Pohang 포항 | 6.2 | Boryeong 보령 | 4.7 | Masan 마산 | 1.1 |
| Ulsan 울산 | 7.6 | Byeonsan 변산 | 2.2 | Jinhae 진해 | 0.9 |
| Busan 부산 | 2.2 | Gochang 고창 | 11.4 | | |
| | | Hampyeong 함평 | 4.2 | | |
| | | Mokpo 목포 | 1.0 | | |

- Coast means at 100 m (Fig. 3 / §3.2): **East ≈ 5, West ≈ 4, South ≈ 1.5 mdd**, little year-to-year difference; East ≈ 3× South. Explanations given (§3.2, translated): East coast — small tidal range, deep water, relatively strong waves; West coast — salt concentrated in winter by the NW monsoon; South coast — shallow water and many bays make sea-salt generation "very difficult".
- Conversion to ISO units: 1 mdd NaCl = 60.7 mg Cl⁻/(m²·d) ⇒ East coast mean at 100 m ≈ 300 mg Cl⁻/(m²·d) (ISO S2/S3 border), South ≈ 90. CAUTION [med]: the K3 box collector is reported by the authors to collect more efficiently than the Japanese PWRI/JIS dry-gauze devices, and dry-gauze results are not directly equal to ISO 9225 wet-candle values (ISO 9223: methods "are not always directly comparable or convertible", via ALC17). Use as relative/regional factors, or scale.

### Q3-KR3. Korea — measured chloride deposition at city monitoring sites, 2007 (ISO 9223 classes) [high]
Source: Kim Y.S., Lim H.K., Kim J.J., Park Y.S., "Corrosivity of Atmospheres in the Korean Peninsula", *Corrosion Science and Technology* 10(4) (2011) 109–117, doi:10.14773/cst.2011.10.4.109. URL opened: https://koreascience.kr/article/JAKO201121161905207.do and PDF https://koreascience.kr/article/JAKO201121161905207.pdf (text extracted).
Table 1 (2007; SO₂ / Cl⁻ deposition in mg m⁻² d⁻¹ and ISO class): Goyang 4.0 P0 / 1.3 S0; Gwangju 3.6 / 0.4; Suwon 5.5 / 1.9; **Ulsan 25.8 P1 / 6.0 S1**; Seoul-Shinchon 5.7 / 4.0 S1; Andong 1.9 / 0.3; Asan 3.6 / 1.4; Ansan 4.9 / 0.6; Jochiwon 2.2 / 0.2; **Incheon 4.6 / 1.2 S0**; **Pohang-Wolpo 3.6 / 40.9 S1**; Pohang-RIST 1.5 / 0.2; Seoul-Anam 4.0 / 0.3.
Abstract: "SO2 classes of each sites were in P0~P1, and chloride classes were in S0~S1, and TOW classes were in τ3~τ4"; carbon steel estimated "C2-C3", zinc/copper/aluminium "C3". TOW (10-yr 2001–2010 KMA) τ5 in Daegwallyeong, Mokpo, Heuksando, Jindo, Jeju areas; τ4 elsewhere (summer). Note: these are urban/industrial monitoring stations, not shoreline sites.

### Q3-JP. Japan — highway-bridge salt-damage zones by region and distance (Specifications for Highway Bridges, H14 = 2002; unchanged in H24 = 2012) [high]
Source: NILIM (国土技術政策総合研究所) Technical Note No. 711, Chapter 2, Table 2.1 "塩害の影響地域区分（道路橋示方書（H14））" and Table 2.2. URL opened: https://www.nilim.go.jp/lab/bcg/siryou/tnn/tnn0711pdf/ks071105.pdf (text extracted locally). Note "道路橋示方書（H24）では ... 対象とする地域区分は従来の道路橋示方書（H14）と同じである".

| Region | Area | Distance from coastline | Countermeasure class |
|---|---|---|---|
| A | Okinawa Prefecture | sea area and ≤ 100 m | S (most severe) |
| | | 100–300 m | I |
| | | everywhere else | II |
| B | Sea-of-Japan coast: Hokkaido (Sōya islands/Wakkanai/Sarufutsu/Toyotomi, Rumoi, Ishikari, Shiribeshi, Hiyama, Matsumae), NW Aomori (Tsugaru, Shimokita parts), Akita, Yamagata, Niigata, Toyama, Ishikawa, Fukui | sea area and ≤ 100 m | S |
| | | 100–300 m | I |
| | | 300–500 m | II |
| | | 500–700 m | III |
| C | all other coasts | sea area and ≤ 20 m | S |
| | | 20–50 m | I |
| | | 50–100 m | II |
| | | 100–200 m | III |
| (D) | beyond | — | no salt-damage class |

(These classes set concrete cover/coating countermeasures; they encode the Japanese empirical judgement that significant airborne-salt influence reaches ~700 m on winter-monsoon (Sea of Japan) coasts, everywhere on Okinawa, and ~200 m on other coasts.)
- Weathering-steel criterion (Japan): unpainted weathering steel allowed where airborne salt "0.05mg/100cm²/day を超えない地域" (≤ 0.05 mdd NaCl ≈ 3 mg Cl⁻/m²·d), measurement may be omitted beyond region-specific distances (5 regions) — quoted in search-result snippet of NILIM TN 828 (https://www.nilim.go.jp/lab/bcg/siryou/tnn/tnn0828pdf/ks082804.pdf, not opened) [med]; Ebrahimi et al. 2025 (PMC12347768, opened) state a critical level of "3 mg/m²/day (0.05 mg NaCl/dm²/day)" was "established for airborne salinity in Japan", limiting corrosion to ~6 µm/year, later relaxed to 6 mg/m²/day [high for the review statement]. Distances per region: see Unverified.
- Other national weathering-steel distance rules (Ebrahimi 2025, §4, [high as review statements]): US "less than two miles (3.22 km)" from coast (with RH ≥ 75 % ≥ 8 months/yr) is "coastal"; France 2 km (North Sea, Channel, Atlantic) / 1 km (Mediterranean); UK no WS bridge "up to 2 km from the coast"; Germany ≥ 500 m outside permanent fog; Australia not within 2 km of an open coast, extendable "up to 40 km"; chloride limits: Japan 3 (later 6), UK 10, US avoid ">50 mg Cl/m²/day".

### Q3-general. Review findings
ALC17 findings [high, peer-reviewed review; original refs cited in brackets]:
- Salinity vs distance at a Spanish (Galicia, Cabo Vilano area) site, ALC17 §4.3, Eq. (11), citing Alcántara et al. [64]:
  "Y = 78288.23 exp (−X/91.34) + 108.40" with "Y the atmospheric salinity expressed as mg Cl⁻/m²·d and X the distance from the shore in meters". → e-folding length ≈ 91 m, inland floor ≈ 108 mg Cl⁻/m²·d at that (very exposed, surf) coast. Note: fit is of very high near-shore values; treat the amplitude as site-specific.
- Corrosion vs distance (ALC17 §4.4.2, Eq. (12), citing [60] = Feliu/Morcillo/Chico "Effect of distance from sea..."): "C = C₀ exp(−βX) + A" (C₀ rate at shoreline, β constant, X distance inland, A rate at zero salinity); for "areas closest to the shoreline (~400 to 600 m)" the decrease "is fairly well represented by a simple exponential relationship"; "the effect of marine atmospheres basically runs to a few hundred metres from the shoreline."
- Wind threshold (ALC17 §4.2): aerosol production is especially sensitive "when the wind speed exceeds some 3–5 m/s"; citing Morcillo et al. [66]: "the wind only needs to blow short time at speeds above 3 m/s in directions with high entrainment of marine aerosol."
- ALC17 §4.3: "An increase in wind speed, even on the same coast, does not always lead to an increase in salinity." and "the total run of marine winds is the parameter that has the greatest influence on the atmospheric salinity of the test site" [64].
- Waves/surf (ALC17 §4.2): "monthly average spectral wave height values of 1.5–2.0 m are sufficient to produce high monthly average salinity" [64]; "In the coastal surf zone, waves can break without the need for simultaneous wind action."; surf-zone aerosol "seems to be the main contributor to the Cl⁻ levels measured in the lower layers of the atmosphere".
- Range (ALC17 §4.1): salinity in marine atmospheres "varies within very broad limits (<5 to >300 mg Cl⁻/m²·d)".
- Particle size (ALC17 §2.2, Ambler & Bain): "only salt particles and droplets of more than 10 µm cause corrosion when deposited on a metallic surface."
- Wet candle vs dry plate (ALC17 §4.5, Corvo et al. [80], Eq. 13): "[Cl⁻]wc (mg/m²·d) = −54.5 + 1.6 [Cl⁻]dc (mg/m²·d)", valid only "for salinity values of a considerable magnitude"; "the wet candle method is more sensitive to the presence of NaCl."
- Shelter from wind (ALC17 §9.5.2): Klassen & Roberge reported "a 34-fold difference between the average mass loss of the most wind-protected and the least wind-protected points."

## Q4. Measured coastal corrosion rates (table)
ALC17 partial items [high]:
- Corrosion vs salinity (ALC17 §4.4.1): "For salinities of less than 600 mg Cl⁻/m²·d, a linear relationship trend between both parameters can be deduced."
- Cuba (Corvo, ALC17 Eq. 34, annual data): "C(g/m²) = 243.7 + 6.7 [Cl⁻]" ([Cl⁻] in mg/m²·d).
- Saturation at very high salinity (ALC17 §4.2, Morcillo et al. [12]): steel corrosion lower at a site with 1905 mg/m²·d than at a nearby site with 824 mg/m²·d (lower O₂ solubility).
- Exfoliation (ALC17 §6.2.4, Chico et al.): requires average Cl⁻ deposition "close to 300 mg/m²·d", with annual corrosion above 100 µm.
- Akaganeite (ALC17 §7.5): associated with "annual average RH around 80% or higher" with Cl⁻ deposition ≈ 60 mg/m²·d or higher.
- ALC17 Table 4 (1-yr rust phases, Galicia, Spain): Cl⁻ deposition Ponte do Porto 4, Cabo Vilano-1 30, Cabo Vilano-2 70, Cabo Vilano-3 665 mg/m²·d.

## Q5. Shelter, orientation, rain washing, fresh-water washing
ALC17 [high]:
- Washing (ALC17 §4.7, Hara et al. [90]): periodic washing with pressurised water at "2–4 MPa" suppresses thick rust growth; Cl⁻ in rust "exceeding approximately 0.2–0.3 wt %" accelerated rust thickening.
- Bridge girders (ALC17 §4.7, Cook et al. [42]): "total thickness losses of about 1.5 mm have been measured on the girders over a period of 20 years."

## Q6. Deliquescence / critical RH

### Q6a. Equilibrium RH over saturated solutions (= deliquescence RH of the pure salt) [high]
Source: L. Greenspan, "Humidity Fixed Points of Binary Saturated Aqueous Solutions", *J. Res. NBS* 81A(1) (1977) 89–96, Table 2. URL opened: https://nvlpubs.nist.gov/nistpubs/jres/81A/jresv81An1p89_A1b.pdf (OCR text extracted locally).

| T (°C) | NaCl RH % | MgCl₂ RH % |
|---|---|---|
| 0 | 75.51 ± 0.34 | 33.66 ± 0.33 |
| 5 | 75.65 ± 0.27 | 33.60 ± 0.28 |
| 10 | 75.67 ± 0.22 | 33.47 ± 0.24 |
| 15 | 75.61 ± 0.18 | 33.30 ± 0.21 |
| 20 | 75.47 ± 0.14 | 33.07 ± 0.18 |
| 25 | 75.29 ± 0.12 | 32.78 ± 0.16 |
| 30 | 75.09 ± 0.11 | 32.44 ± 0.14 |
| 35 | 74.87 ± 0.12 | 32.05 ± 0.13 |
| 40 | 74.68 ± 0.13 | 31.60 ± 0.13 |

→ NaCl DRH ≈ 75 % almost independent of T; MgCl₂ DRH ≈ 33 % (weakly decreasing with T). Model suggestion (derived, not from a source): wet-surface threshold RH_crit = 75 % for NaCl-dominated deposits, falling toward ~33 % as the MgCl₂ fraction of sea salt controls the residual brine; ISO TOW uses 80 %.

### Q6b. Corrosion-relevant thresholds from the literature
ALC17 [high]:
- §4.6: "the critical RH for MgCl₂ is 35%, much lower than the 75% corresponding to NaCl."
- §4.4 (Preston & Sanyal): corrosion under NaCl deposits "starts to be seen at 70% RH"; (Evans & Taylor) "sea salt particles cause corrosion at a lower RH than NaCl particles."
- §9.5.1 wetting model: surface wetted when surface RH exceeds DRH of surface salts; Eq. (44) "TOW = TCD + rain period"; ISO 9223 TOW = time when "the RH ... is equal to or greater than 80% at temperatures above 0 °C."

## Unverified (remembered, not confirmed online)
(pending)
