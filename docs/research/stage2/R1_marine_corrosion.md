# R1 — Marine (salt-induced) atmospheric corrosion of crane steel vs. distance to sea

Status: COMPLETE (2026-10-20). Every number carries the URL actually opened + short quote/table ref; values not confirmed online are listed under "Unverified".
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

### Q4a. Benchmark table — measured first-year (or annual) corrosion at coastal/marine sites
Source keys (URL actually opened → table/figure used):
- **[CHI17]** Chico B., de la Fuente D., Díaz I., Simancas J., Morcillo M., "Annual Atmospheric Corrosion of Carbon Steel Worldwide. An Integration of ISOCORRAG, ICP/UNECE and MICAT Databases", *Materials* 10(6) (2017) 601, PMC5553418. URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC5553418/fullTextXML — Table 5 (ISOCORRAG: "1st Year Corrosion, μm", T, TOW, SO₂, Cl), Table 6 (MICAT, adds RH and precipitation). Method quote: "Measurement of Cl deposition rate according to ISO 9225". Each ISOCORRAG/MICAT row = one 1-yr exposure series; ranges below are min–max over the listed series. NOTE: the Europe-PMC XML of Table 6 stops at code M27 (sites M28–M74 not in the opened text).
- **[LI11]** Li S.Y., "Monitoring of Atmospheric Corrosivity inside Steel Upper Box Girder in Yeongjong Grand Bridge", *Corros. Sci. Tech.* 10(3) (2011) 87–94. URL opened: https://koreascience.kr/article/JAKO201121161903206.pdf (text extracted locally; Tables 1–3, Fig. 11).
- **[KL13]** Kim J.H., Lee J.K., "Atmospheric corrosion rate and corrosivity categories of industrial metals in Asan area", *J. KAIS* 14(10) (2013) 4653–4657. URL opened: https://koreascience.or.kr/article/JAKO201333651339243.pdf (Table 2 "Corrosivity categories of regional", numbers + area labels extracted; Korean text not extractable). Category basis cited as "ISO9224/1992"; exposure duration behind each category not readable → categories only.
- **[CHOI23]** = Choi T. et al. 2023 (Q1a), Fig. 4(a)–(d) bar charts read by eye: https://cdn.ncbi.nlm.nih.gov/pmc/blobs/95b0/10419963/3c45d25ea31e/materials-16-05326-g004a.jpg and …/3d1a2c0a9483/materials-16-05326-g004b.jpg. Chloride "estimated by an X-ray photoelectron spectroscopy (XPS) analysis" of a silver specimen (non-standard; values ≪ ISO 9225 levels) in aircraft-parking revetments/shelters; bases anonymised.
- **[YAN20]** Yan L., Diao Y., Gao K., "Analysis of Environmental Factors Affecting the Atmospheric Corrosion Rate of Low-Alloy Steel Using Random Forest-Based Models", *Materials* 13(15) (2020) 3266, PMC7435615. URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC7435615/fullTextXML (Table 2) + Fig. 1 https://cdn.ncbi.nlm.nih.gov/pmc/blobs/394a/7435615/30f265a59862/materials-13-03266-g001.jpg (values read by eye; each point = mean of 13 low-alloy steels incl. SM490A and SMA490, NIMS MatNavi data).
- **[CHE26]** Chen T., Yang L., Liu C., Zhang T., Chen S., Deng X. et al., "Atmospheric Corrosion Behavior of Q235 Steel Exposed to the Subtropical Marine Environment in the East China Sea for Two Years", *Materials* 19(6) (2026) 1189, PMC13027690. URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC13027690/fullTextXML. Quote: "The field exposure test site was located on a 26 m-high offshore platform in the East China Sea".
- **[VER23]** = Vera et al. 2023 (Q2), Rapa Nui. Quote: "22 m above sea level, and 500 m away from the sea"; "The corrosion rate of steel, at one year of exposure, was 51.95 μm year−1".
- **[HER25]** = Heredia et al. 2025 (Q1d), Chocó, Colombia, Table 3; chloride by "the wet candle method" (NBR 6211).
- **[PAT24]** (Q1c), Bonassola: "about 10 m distance from the sea, at an altitude + 10 m a.s.l"; Table 6; Cl⁻ "mean value 209 mg/(m2·day)—level S2, with maximum values of about 500 mg/(m2·day)".
- **[VER09]** Vera R., Guerrero F., Delgado D., Araya R., "Evaluation of galvanized structural steel in front of marine environment corrosion part 1", *Revista de la Construcción* 8(2) (2009) 37–50 — abstract only, URL opened: https://investigadores.anid.cl/en/public_search/work?id=145962.

| # | Site (country) | Distance to sea / height | Cl⁻ mg/(m²·d) (method) | SO₂ mg/(m²·d) | T °C / RH % / TOW | Period | Carbon steel µm/a | Zinc / galv. µm/a | Src | Conf. |
|---|---|---|---|---|---|---|---|---|---|---|
| K1 | Yeongjong Grand Bridge, Incheon (KR) — sensor outside upper steel box girder | sea-crossing bridge; height n/g | 1.22 (dry gauze; ×2 → ≈2.4 wet-candle eq., see Q4c) S0 — pairing of the "1.220 / 1.502" row with the outdoor sampler inferred (only row with non-zero SO₂; text: inside "significantly less contaminated by chloride and SO2") | 1.50 (P0) | TOW 47.85 % of time (τ4; ≈4190 h/a derived) | 7 months, May–Nov (year n/g) | ISO estimate C3 (25–50); thin-film ER sensor "0.03 mm/y" (≈30); lab simulation 68 | — | LI11 Tab. 1–3 | high (values) / med (sensor ≠ ISO 9226 coupon) |
| K2 | Pohang, "marine" station of the Korean national exposure network (KR) | n/g | n/g | n/g | n/g | n/g | category 5 (C5 band 80–200) | hot-dip galv. category 5 (C5 band 4.2–8.4) | KL13 Tab. 2 | med |
| K3 | Busan, "industrial" station (KR) | n/g | n/g | n/g | n/g | n/g | category 4 (50–80) | Gal-S category 1 (as printed) | KL13 Tab. 2 | med |
| K4 | Dangjin, "industrial" (W coast) (KR) | n/g | n/g | n/g | n/g | n/g | category 4 (50–80) | Gal-S category 4 | KL13 Tab. 2 | med |
| K5 | Pohang-Wolpo, "industrial" (KR) | n/g | 40.9 (2007, S1; Q3-KR3 [Kim 2011]) | 3.6 (2007) | — | n/g | category 4 (50–80) | Gal-S category 3 | KL13 Tab. 2 + Kim 2011 Tab. 1 | low (two sources/years combined) |
| K6 | ROK-AF coastal base UC2R (roofed revetment) (KR) | ≈7 km | ≈0.33 (XPS on Ag) | n/g | TOW ≈2400 h | 1 yr | ≈128 g/(m²·a) ≈ 16 | — | CHOI23 Fig. 4 | low |
| K7 | ROK-AF coastal base UC1R (high-roofed revetment) (KR) | ≈1.5 km | ≈0.16 (XPS on Ag) | n/g | TOW ≈3000 h | 1 yr | ≈66 g/(m²·a) ≈ 8.4 | — | CHOI23 Fig. 4 | low |
| K8 | ROK-AF coastal bases SC1 / SC3 (sheltered) (KR) | ≈1 km / ≈0.3 km | ≈0.04 / ≈0.05 (XPS on Ag) | n/g | TOW ≈1250 / ≈950 h | 1 yr | ≈11 / ≈24 g/(m²·a) ≈ 1.4 / 3.1 | — | CHOI23 Fig. 4 | low |
| J1 | Choshi (JP), ISOCORRAG I23 | n/g in CHI17 (JWTC Choshi site: 4.0 km, OHA18 Table 1 — same site assumed) | 36.4–47.9 (ISO 9225) | 5.7–6.6 | 15.5–16.2 °C; TOW 0.64–0.68 | 6 series | 39.7–48.2 | — | CHI17 Tab. 5 | high (distance link low) |
| J2 | Okinawa (JP), ISOCORRAG I25 | n/g | 60.1–91.0 | 6.6–10.4 | 22.8–24.0 °C; TOW 0.34–0.54 | 6 series | 39.2–138 | — | CHI17 Tab. 5 | high |
| J3 | Choshi (JP), NIMS open rack | (JWTC Choshi 4.0 km, assumed) | 32.0–32.3 mg **NaCl** ≈ 19.4–19.6 Cl⁻ | 4.9–5.1 | 14.7–15.3 / 77.6–79.0 / 4629–4908 h; wind 3.0–3.5 m/s | 1-yr (series from 1998/2002/2003) | ≈42 open (≈54 rain-sheltered) | — | YAN20 Tab. 2, Fig. 1 | med |
| J4 | Miyakojima (JP), NIMS open rack | (JWTC Miyako-jima 2.0 km, assumed) | 45.8–49.2 mg NaCl ≈ 27.8–29.8 Cl⁻ | 2.1–2.4 | 23.9–24.0 / 78.5–79.0 / 5113–5249 h; wind 4.2–4.7 m/s; rain 1899–2314 mm | 1-yr | ≈57 open (≈84 rain-sheltered) | — | YAN20 Tab. 2, Fig. 1 | med |
| J5 | JWTC network, 25 sites (JP) — envelope of the fitted data | 0.008–76 km (OHA18 Table 1) | 1.3–129.4 (wet candle) | 1.1–8.2 | 6.4–23.5 / 58–80 | n = 118 one-year tests (from 2001) | 10.0–48.0 | 0.48–2.20 | OHA18 Tab. 1, 3 | high (ranges) |
| C1 | East China Sea offshore platform (CN), Q235 | offshore; specimens on 26 m-high platform | 81 (method n/g) | n/g | 20.5 / 77.7 / 4022 h | 2 yr | 217.2 at 12 months (51.2 at 6 mo; 143.2 at 24 mo) | — | CHE26 §2, §3.1 | high |
| E1 | Ostende (BE), ISOCORRAG I19 | n/g | 95.3–112.3 | 8.0–25.6 | 9.7 °C; TOW 0.66–0.73 | 3 series | 83.5–118 | — | CHI17 Tab. 5 | high |
| E2 | Salins de Giraud (FR), I18 | n/g | 83.7–149.9 | 31.4–32.5 | 13.6–15.4 °C; TOW 0.35–0.45 | 3 series | 70.2–82.1 | — | CHI17 Tab. 5 | high |
| E3 | Ponteau-Martigues (FR, marine-industrial), I15 | n/g | 125.0–158.4 | 35.8–120.8 | 14.6–16.2 °C; TOW 0.35–0.50 | 5 series | 66.4–83.5 | — | CHI17 Tab. 5 | high |
| E4 | Bohus-Malmön Kattesand (SE), I38 | n/g | 31.0–81.3 | 1.6–7.0 | 6.1–9.7 °C; TOW 0.45–0.47 | 6 series | 28.8–43.0 | — | CHI17 Tab. 5 | high |
| E5 | Lagoas-Vigo (ES), I35 | n/g | 11.5–18.2 | 35.7–49.3 | 15.2–17.2 °C; TOW 0.29–0.37 | 6 series | 22.7–34.4 | — | CHI17 Tab. 5 | high |
| E6 | Bonassola, Liguria (IT), MARECO site | ≈10 m; +10 m a.s.l. | 209 mean, max ≈500 (ISO 9225) S2 | 3.2 µg/m³ (P0) | 18.4 / 74 / TOW(80 %) 0.33 | Oct 2023–Oct 2024 | LPR×TOW estimate 79.3 (4 mo), 75.8 (8 mo); ISO DRF 83.2 | galv. 4.4 (4 mo), 9.4 (8 mo) (LPR) | PAT24 §4.1.2, Tab. 6 | high (values) / med (not 1-yr mass loss) |
| A1 | Kure Beach NC (US), ISOCORRAG I44 | n/g (LaQue's classic lot "250 m from the shoreline", ALC17 — same lot not verified) | 68.0–150.5 | 4.3–8.4 | 13.3–18.2 °C; TOW 0.46–0.50 | 5 series | 32.5–43.8 | — | CHI17 Tab. 5 | high |
| A2 | Panama, Fort Sherman coastal site, I46 | n/g | 324.7 | 42.4 | 27.3 °C; TOW 0.82 | 1 series | 373 | — | CHI17 Tab. 5 | high |
| A3 | Camet / Mar del Plata (AR), MICAT M01 + ISOCORRAG I02 | n/g | 18.2–70 | 2 (est.) | 13.9–14.5 / 79–80 | ≥5 series | 24.9–78.2 | — | CHI17 Tab. 5–6 | high |
| A4 | Arraial do Cabo (BR), M09 | n/g | 167.7–359.8 | 4.4–9.5 | 24.2–24.8 / 77–79; rain 605–985 mm | 3 series | 141.7–194.7 | — | CHI17 Tab. 6 | high |
| A5 | Ubatuba (BR), M11 | n/g | 113.2 | 2.6 | 22.1 / 80; rain 1689 mm | 1 | 301.9 | — | CHI17 Tab. 6 | high |
| A6 | Limón (CR), M23 | n/g | 376 | 3.5 | 25.3 / 88; rain 3531 mm | 1 | 371.5 | — | CHI17 Tab. 6 | high |
| A7 | Puntarenas (CR), M22 | n/g | 38.7 | 6.3 | 27.6 / 80 | 1 | 61.6 | — | CHI17 Tab. 6 | high |
| A8 | Cojímar, Havana (CU), M27 | n/g | 50.4–123.3 | 13.5–40.4 | 24.2–25.7 / 79–80 | 9 series | 71.6–391.2 | — | CHI17 Tab. 6 | high |
| A9 | Isla Naval, Cartagena (CO), M19 | n/g | 43.6–69.5 | 7.8–14.2 | 27.6–28.2 / 85–87 | 3 series | 16–54 | — | CHI17 Tab. 6 | high |
| A10 | Rapa Nui, Navy garrison (CL) | 500 m; 22 m a.s.l. | 53.04 (yr 1; 48.4–53.0 over 3 yr) | 1.89 (yr 1; mean 3.53) | ≈22 °C / ≈73 % / 2616 h (yr 1) | 3 yr | 51.95 (SAE 1020) | galv. 2.00 | VER23 Tab. 2, §3 | high |
| A11 | Rapa Nui, earlier MICAT station (CL) | 300 m; 0 m a.s.l. | "<50" | 1.15 | similar | 1 yr | 30.0–32.8 (0.7 % C steel) | Zn 3.92 | VER23 (citing MICAT) | med |
| A12 | Bahía Solano, Chocó Pacific coast (CO) — hyper-humid | "0 km" (coastal area) | 8.31 (wet candle) S1 | 1.38 | 26.6 / 94.6 / 8182 h; rain 5293 mm | Dec 2022–Nov 2023 (≤18 mo) | 510 = highest rate of the time series (rates then "decreased gradually … stabilization was observed after eight months"); category ">C5" | galv. 70 after 4 months | HER25 Tab. 3, §3 | high (values) / note: not a 1-yr value |
| A13 | Valparaíso (CL) — galvanized only | n/g | n/g | n/g | n/g | 1 yr | — | hot-dip galv. 16.4 ("the corrosion velocity determined in time was 16.4 μm/year") | VER09 abstract | med |

Row count 33 (Korea 8, Japan 5, China 1, Europe 6, Americas 13). "n/g" = not given in the opened source. Distances for ISOCORRAG/MICAT stations are not tabulated in CHI17 (see Unverified).

### Q4b. Empirical relationships drawn from these benchmark data
- **Marine regression on the integrated ISOCORRAG+MICAT data** (CHI17 Eq. 6, [high]): **C = −24.50 + 0.75·Cl + 0.67·SO₂ + 77.32·TOW** (R² = 0.474, N = 206), C = first-year carbon-steel corrosion (µm), Cl and SO₂ in mg/(m²·d), TOW as annual fraction. Valid range (CHI17 Table 7, marine): C 8.6–411.2 µm, T 3.9–28.2 °C, TOW 0.16–0.99, SO₂ 0.3–171.7, Cl 3.03–376 mg/(m²·d). Quote: "Cl is the variable that contributes with the highest percentage (R2 = 0.411)"; "Temperature is excluded as a significant variable."
- Other DRFs listed in CHI17 Table 8 [high, as reproduced]: ISOCORRAG (TOW-based) **C = 0.091[SO₂]^0.56·TOW^0.52·exp(f(T)) + 0.158[Cl]^0.58·TOW^0.25·exp(0.050·T)**, f(T) = 0.103(T−10) for T ≤ 10 °C, otherwise −0.059(T−10) (N = 125, R² = 0.85); MICAT **C = −0.44 + 6.38·TOW + 1.58[SO₂] + 0.96[Cl]** (N = 172, R² = 0.56); ISOCORRAG/MICAT incl. Russian frigid sites C = 0.085·SO₂^0.56·TOW^0.53·exp(f) + 0.24·Cl^0.47·TOW^0.25·exp(0.049·T), f = 0.098(T−10) for T ≤ 10 °C, otherwise −0.087(T−10) (N = 119, R² = 0.87).
- Outliers / saturation (CHI17 §2.4): points with salinity above 200 mg Cl⁻/(m²·d) form a cloud "which does not seem to agree with the relatively low carbon steel corrosion values found (50–100 µm)"; Saint Remy, Tannanger and Kvarnvik were removed as anomalous. [high]
- Piecewise salinity trend (Morcillo et al. 2000, intro; URL opened: https://www.sciencedirect.com/science/article/abs/pii/S0010938X99000487): below 100 mg Cl⁻/(m²·d) "a slight increase in steel atmospheric corrosion", faster increase up to 400, after which "corrosion seems to stabilize". [high]
- Distance law (Feliu, Morcillo, Chico, *Corrosion* 55 (1999) 883, abstract via OpenAlex, URL opened: https://api.openalex.org/works/doi:10.5006/1.3284045): "the effect of marine atmospheres extends principally some few hundred meters from the shoreline and decays rapidly further inland"; for "(~ 400 m to 600 m) … a simple exponential relationship". [high]
- Alcántara et al., *Corros. Sci.* 97 (2015) 74–88 (abstract/highlights, URL opened: https://www.sciencedirect.com/science/article/abs/pii/S0010938X15001754): one-year study "in six pure marine atmospheres with annual average chloride deposition rates of 70–1906 mg Cl−/m2 day" (April 2013–March 2014); highlight: "Salinity and steel corrosion obey an exponential decay function with shore distance." [high] (Per-site corrosion values are in figures/tables not opened → Unverified.)
- European Federation of Corrosion exposure-site catalogue, 16 marine sites (PAT24 §4.2): carbon steel "ranging from 200 to 2300 g/(m2·y)" (≈25–290 µm/a), "The mean corrosion rate is 817 g/(m2·y) (C5 category)"; at 7 of 13 sites the zinc-based category is one class lower than the steel-based one. [high as review statement]

### Q4c. Method / unit notes for using the table
- Dry gauze vs wet candle (LI11 §2, citing JIS Z 2382): "Clwet candle = 2Cldry gauze" (Eq. 1). [med — secondary citation of the JIS] Relevant to the Korean K3-box (dry-gauze type) data in Q3-KR/Q3-KR2 and K1 above. For dry *plate* vs wet candle see Corvo (Q1d / Q3-general).
- mg NaCl → mg Cl⁻: ×0.607 (35.45/58.44) (NIMS rows J3/J4 report NaCl; Korean KICT data report NaCl).
- g/(m²·a) → µm/a for steel: ÷7.86 (density); used for K6–K8.
- Korean national exposure network (Kim Y.S., Lim H.K., Kim J.J., Hwang W.S., Park Y.S., "Corrosion Cost and Corrosion Map of Korea – Based on the Data from 2005 to 2010", *Corros. Sci. Tech.* 10(2) (2011) 52–59; URL opened: https://koreascience.kr/article/JAKO201121161896203.pdf): outdoor exposure tests "performed at 21 sites in Korea; 10 marine regions including industrial, 5 rural regions, and 6 urban regions". Maps (Fig. 9–11) are based on 12-month data; per-site values exist only as map graphics (not extracted → Unverified). [high for the programme description]

## Q5. Shelter, orientation, rain washing, fresh-water washing
ALC17 [high]:
- Washing (ALC17 §4.7, Hara et al. [90]): periodic washing with pressurised water at "2–4 MPa" suppresses thick rust growth; Cl⁻ in rust "exceeding approximately 0.2–0.3 wt %" accelerated rust thickening.
- Bridge girders (ALC17 §4.7, Cook et al. [42]): "total thickness losses of about 1.5 mm have been measured on the girders over a period of 20 years."

### Q5a. Rain-sheltered vs boldly exposed racks at the same coastal site (Japan, NIMS) [med — values read from YAN20 Fig. 1]
Source: YAN20 (see Q4a keys), URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC7435615/fullTextXML and Fig. 1 https://cdn.ncbi.nlm.nih.gov/pmc/blobs/394a/7435615/30f265a59862/materials-13-03266-g001.jpg. Set-up: open racks horizontal and 45° facing south; "Under the sheltered environment, the specimens were horizontally set up under the roof of the shelter." Each value = mean corrosion rate (µm/a, "annual corrosion depth") of 13 low-alloy steels.

| Site (Cl⁻ as mg NaCl/(m²·d), Table 2) | open 1 yr | sheltered 1 yr | ratio sh/open | open 10 yr | sheltered 10 yr | ratio sh/open |
|---|---|---|---|---|---|---|
| Tsukuba, rural (2.8–3.3) | ≈16 | ≈14 | ≈0.85 | ≈4.5 | ≈4.5 | ≈1.0 |
| Choshi, coastal (32.0–32.3) | ≈42 | ≈54 | ≈1.3 | ≈17 | ≈63 | ≈3.7 |
| Miyakojima, subtropical coastal (45.8–49.2) | ≈57 | ≈84 | ≈1.5 | ≈43 | ≈111 | ≈2.6 |

- Quotes (YAN20 §3): "the roof of the shelter significantly changed the actual environmental parameters (e.g., precipitation, chloride deposition rate, etc.) near the steel samples"; "surfaces sheltered from the sun and rain tend to form a loose and non-compact oxide"; high-rate samples "under both open (>80 μm/a) and sheltered (>100 μm/a) environments".
- Reading: at chloride-bearing sites the rain-sheltered/open ratio grows with time (≈1.3–1.5 at 1 yr → ≈2.6–3.7 for the 10-yr mean rate), because the open surfaces' rate falls while the sheltered rate does not; at the low-chloride rural site the ratio stays ≈1.

### Q5b. Enclosed / dehumidified interiors (box girders, closed shelters)
- Yeongjong Grand Bridge, Incheon (LI11, Table 3) [high]: corrosion rate inside the dehumidified upper box girder "< 0.0004" mm/y (thin-film ER sensor) vs "~ 0.03" mm/y outside → inside/outside ≈ 1/75; ISO-method estimates "< 0.0013" vs "0.025-0.05" mm/y. Inside: "the RH was kept below about 50% inside the box girder"; system "generally set to keep the RH at 40%", with "margin of safety below the maximum level of 60% RH"; inside TOW 0–2.96 % (τ1–τ2), Cl⁻ ≈0.56–0.65 mg/(m²·d) (dry gauze). Cited background (LI11 §1, Fujino 1989): steel inside a box girder corrodes "as small as 0.003 to 0.3" [µ]m/y (µ lost in text extraction; context = inside-girder rates). Caveat (LI11 §3): water entering via the hanger-rope anchorage caused local corrosion → enclosure only works if water ingress is prevented.
- ROK Air Force bases (CHOI23 Fig. 4, rows K6–K8) [low]: coastal *sheltered* bases SC1/SC3 (≈0.3–1 km from sea) ≈11–24 g/(m²·a) vs coastal *unsheltered, roofed* revetments UC1R/UC2R ≈66–128 g/(m²·a); XPS chloride on Ag ≈0.04–0.05 vs ≈0.16–0.33 mg/(m²·d). Different bases, not a controlled pair.
- Contrast with Q5a (derived interpretation, not from a source): a *rain-sheltered but ventilated* surface (open-sided roof, underside of a girder or boom) receives airborne salt but no rain rinsing → factor > 1 in marine air; a *closed, dry* enclosure (box-girder interior, machinery house, hardened shelter) blocks salt and moisture → factor ≪ 1. Model these as two different exposure classes.

### Q5c. Orientation: horizontal (upward-facing) vs vertical surfaces under a bridge deck [high]
Source: **[KRI19]** Křivý V., Kubzová M., Konečný P., Kreislová K., "Corrosion Processes on Weathering Steel Bridges Influenced by Deposition of De-Icing Salts", *Materials* 12(7) (2019) 1089, PMC6479928. URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC6479928/fullTextXML (Tables 2–4, §4.1–4.2). De-icing-salt spray from a highway below (bridge B1) vs railway below (B2); weathering steel coupons, 1-yr exposures 2016/17 and 2017/18; all positions are under the deck (rain-sheltered).

| Bridge / position | rcorr vertical (web) µm/a | rcorr horizontal (bottom-flange top) µm/a | ratio | dry-plate Cl⁻ vertical → horizontal mg/(m²·d) (2016/17) |
|---|---|---|---|---|
| B1 P1 | 19.3 | 49.2 | 2.5 | 2.97 → 12.90 |
| B1 P2 | 19.0 | 45.8 | 2.4 | 2.58 → 11.38 |
| B1 P3 | 22.0 | 49.4 | 2.2 | 5.20 → 10.43 |
| B1 P4 | 16.1 | 49.9 | 3.1 | 3.60 → 13.69 |
| B2 P6 (no de-icing traffic below) | 13.6 | 20.4 | 1.5 | 1.05 → 4.14 |

- Quotes: "up to three times the thickness of the corrosion layer was observed compared to the adjacent vertically-oriented surface"; "Similar ratios were also found … when evaluating corrosion losses"; "On horizontally-oriented dry plates, up to five times the deposition rate of chlorides (Sd,p) was found compared to the vertically-oriented dry plates".
- Inner vs outer girder (rain/wind cleaning): B1 P1 (external girder) mCl = 5.68 g/m², Sd = 14.72 mg/(m²·d); P2 (internal girder) mCl = 11.40 g/m², Sd = 9.80 — "the external surfaces are also regularly cleaned by wind and rainfall". 1-yr rcorr showed "no significant differences" between outer and inner girders; the difference is expected "only after long-term exposure".
- Ebrahimi et al. 2025 (PMC12347768, opened — see Q3-JP) [high, guideline statements]: "Corrosion rates are higher on horizontal, sloping, and downward-facing surfaces in sheltered interior locations than on comparable upward-facing surfaces." (vertical surfaces "less likely to experience pitting and corrosion"); "rain-sheltered surfaces, where chlorides accumulate and are never washed away"; "the lack of pollutant wash-off by rain".
- Skyward vs groundward faces of boldly exposed panels in marine air: no quantitative open source found (see Unverified).

### Q5d. Rain washing — where it helps and where it does not
- Helps (chloride removal on exposed faces): KRI19 inner/outer girder (above); YAN20 open vs sheltered (Q5a); PennDOT report (below): a roadside member "is also more exposed to rain, which could reduce the salt concentration on the member between wash cycles".
- Does not help when wetness dominates: Chocó, Colombia (HER25, Table 3): rainfall 5293–7631 mm, TOW 7246–8257 h/a, Cl⁻ only 1.35–8.31 mg/(m²·d), yet carbon steel 494–596 µm/a (highest rates of the series) and category ">C5" at all three stations — "high environmental humidity generates a longer wet period, increasing carbon steel's corrosion rates". [high]
- Sawant & Wagh 1991 (India, TRID abstract — see Q7d): "The increase in corrosion rate for an extended exposure period at the coastal station is attributed to the wetting of coupons by heavy rain." [high, abstract]

### Q5e. Periodic fresh-water washing — measured effects
- Hara S. et al., "Suppression of deicing salt corrosion of weathering steel bridges by washing", *Corros. Sci.* 47 (2005) 2419–2430 (abstract/conclusions, URL opened: https://www.sciencedirect.com/science/article/abs/pii/S0010938X05001058) [high]: three-year washing experiment (2001–2003, Shikoku expressway, high-pressure "2–4 MPa"); "Washing with water suppressed corrosion owing to the disappearance of chloride ions"; conclusion (1): chloride in rust "more than approximately 0.2 mass%, with sprinkling of deicing salts in a winter, remains in a summer and accelerates rust layer growth remarkably".
- **[PENN13]** Alland K., Vandenbossche J.M., Vidic R., Ma X., "Evaluation of Bridge Cleaning Methods on Steel Structures", Univ. of Pittsburgh for PennDOT, FHWA-PA-2013-007-PIT WO 2 (12/31/2013). URL opened: https://www.pa.gov/content/dam/copapwp-pagov/en/penndot/documents/research-planning-innovation/research-projects/evaluation%20of%20bridge%20cleaning%20methods%20on%20steel%20structures%20-%20final%20report.pdf (text extracted locally) [high]. Surface soluble-salt concentration before → after one wash (coated steel, river/tap water):

| Location (report section) | before mg/m² | after mg/m² | removal |
|---|---|---|---|
| Vanport Br., Loc. 1 | 13 | 5 | 61 % |
| Vanport Br., Loc. 2 (roadside face) | 17 | 10 | 40 % |
| Hassam Rd, bearing stiffener (rusted) | 250 | 36 | 85 % |
| I79 / Thoms Run, Loc. 3 web of fascia beam (washed from bucket truck) | 248 | 16 | 93 % |
| I79 / Thoms Run, Loc. 4 underside of interior beam — washer losing pressure | 247 | 149 | 40 % |
| same, rewash after pressure restored | (247) | 41 | 73 % |
| I79 / Thoms Run, Loc. 5 beam web — damaged washer → repaired | 70 | 49 → 18 | 29 % → 65 % |
| I79 / Thoms Run, Loc. 6 topside of bottom flange under thick debris — damaged → repaired | 298 | 246 → 200 | 17 % → 18.5 % ("very small washing angle") |

  Other quoted findings: "pressurized water between 12 and 16 MPa (1750-2000 psi) is capable of removing a large portion of the soluble salts"; crews removed salts at "10 MPa (1500 psi)"; "Washing efficiency could be compromised if the elevation of the washing nozzle is even 1.5 m below the targeted washing area"; debris-covered upward-facing flange tops kept ≈200–250 mg/m² even after washing (Loc. 6); "Washing with the specified pressure is ineffective in cleaning areas with a failed coating that are exhibiting corrosion"; members "close to the roadway (6.5 m or 21.5 feet) contain some of the highest salt concentrations"; and "it is unknown how quickly the surface salt concentration on bridges increases after washing" (re-contamination rate not measured).
- Modelling note (derived, not from a source): treat a wash as a multiplicative removal of the *surface* chloride inventory, η ≈ 0.4–0.9 per wash at working pressure (10–16 MPa, nozzle level with/above the surface), ≈0.2–0.4 with low pressure, grazing angle or debris (PENN13 table), and little effect on actively rusting areas, followed by re-accumulation at the local deposition rate; chloride already incorporated in rust (Hara: >≈0.2 mass %) is not removed by a single wash.

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

## Q7. Sea-salt aerosol / chloride deposition vs wind, run of marine wind, and height

### Q7a. Open-sea production vs wind speed (whitecaps) [high]
Source: **[GRY14]** Grythe H., Ström J., Krejci R., Quinn P., Stohl A., "A review of sea-spray aerosol source functions using a large global set of sea salt aerosol concentration measurements", *Atmos. Chem. Phys.* 14 (2014) 1277–1297. URL opened: https://acp.copernicus.org/articles/14/1277/2014/acp-14-1277-2014.pdf (text extracted locally; Appendix A, §2, Conclusions).
- Whitecap fraction (Monahan & O'Muircheartaigh 1980, GRY14 Eq. A1): **W(U10) = 3.84 × 10⁻⁶ · U10^3.41** (U10 = 10-m wind speed, m/s). GRY14 §2: "the difference in whitecap fraction for e.g., 5 and 10 m s-1 winds translates into approximately a 1 : 11 difference in whitecap fraction."
- Size-resolved flux (Monahan et al. 1986, GRY14 Eq. A2): dF/dDp = W_M · 3.6 × 10⁵ · Dp⁻³ · (1 + 0.057·Dp^1.05) · 10^(1.19·exp(−B²)), B = (0.38 − log Dp)/0.65 — i.e. production ∝ U10^3.41 at every size.
- GRY14 Conclusions: "Wind speeds in the range 5-14 m s-1 are very frequent and are responsible for about 80 % of the global SSA production"; "A power law dependence where SSA production is proportional to U10^3.5 was found to best describe the observed SSA concentrations."
- Spume: larger particles "are produced as spume is torn off the wave crests, which only happens in strong winds" (GRY14 §1).

### Q7b. Near-surface sea-salt concentration vs wind speed (exponential laws)
- Erickson D.J., Merrill J.T., Duce R.A., *J. Geophys. Res.* 91 (1986) 1067 — abstract, URL opened: https://earthref.org/ERR/56671/ [high, abstract]: concentrations "15 m above the sea"; combined data yield "relationships of form C =exp ( a s + b ), where C is sea-salt concentration in micrograms per cubic meter" (s = horizontal wind speed, m/s).
- Gong S.L., Barrie L.A., Blanchet J.-P., *J. Geophys. Res.* 102 (1997), doi:10.1029/96JD02953 — abstract, URL opened: https://earthref.org/ERR/52987/ [high, abstract]: surface-layer concentration χ (µg m⁻³) vs U10 "is given by χ=b·e^(a·U10)"; "The value a and b range from 0.20 and 3.1 for Mace Head, Ireland to 0.26, and 1.4 for Heimaey, Iceland"; "The dependence of χ on surface wind speed is weaker for smaller particles and for particles at higher altitudes"; residence time in the first model layer (0–166 m) "ranges from 30 min for large particles (r=4–8 μm) to ~60 hours for small particles (r=0.13–0.25 μm)".
- Lovett (1978) ln C = 0.16·U + 1.45 has the same functional form (a = 0.16 s/m, b = 1.45) but the coefficients were NOT verified in an opened source (see Unverified). Morcillo et al. 2000 (intro, URL in Q4b) only state: "Previous studies of Lovett [10] and Taylor [12] find a linear relationship between both variables" (salinity vs wind velocity).

### Q7c. Coastal deposition vs wind regime: onshore direction, ≈3 m/s threshold, run of marine wind, storms
- **Morcillo M., Chico B., Mariaca L., Otero E., "Salinity in marine atmospheric corrosion: its dependence on the wind regime existing in the site", *Corros. Sci.* 42 (2000) 91–104** (abstract + snippets, URL opened: https://www.sciencedirect.com/science/article/abs/pii/S0010938X99000487) [high]: Tarragona (Spain) site; monthly salinity Aug 1994–Apr 1995 and 3-monthly Mar 1996–Feb 1997 vs marine (M) and total (T) wind-frequency tables. Quotes: "there are certain marine wind directions (referred to here as 'saline winds') which especially contribute to the entrainment of marine aerosol"; "There seems to be a critical velocity (close to 3 m s−1) for saline winds, above which coastal atmospheric salinity notably increases"; at that site "the saline winds correspond mainly to direction ENE"; "an increase in wind velocity … does not always lead to an increase in salinity, as the final result is dependent on the wind direction".
- Storm pulses (Morcillo 2000 intro, citing a Swedish west-coast study): "during two westerly storms (half day and 3 days long, respectively), the total salt quantity deposited was 4900 mg m−2 of exposed area"; storms can deposit "even more than is deposited during an entire month with the normal wind regime". [high]
- **Run of marine wind** (ALC17 §4.3, opened): "the sum of adding together the marine wind speed in each direction multiplied by the time it has been blowing"; it, rather than mean wind speed, has "the greatest influence on the atmospheric salinity of the test site" (see Q3-general). Russian long-term studies (Murmansk, Vladivostok): chloride entrainment depended on "the product of the wind speed by its duration (wind power)". [high]
- Meira G.R. et al., *Constr. Build. Mater.* 135 (2017) 287–296 (abstract/intro, URL opened: https://www.sciencedirect.com/science/article/abs/pii/S0950061816321018) [high]: wind thresholds "between 3 m/s and 7.1 m/s can be found in literature"; "previous Spanish and Brazilian experiences show that wind speeds above 3 m/s are high enough to generate a sharp rise in salinity"; best predictor = "wind speeds stronger than 3 m/s multiplied by the length of time in which they are observed", written Σv₃·t₃/t (v₃ = speeds > 3 m/s, t₃ their durations, t total time).
- Alcántara et al. 2015 (Q4b, opened abstract): highlight "Relationship between wave height, marine wind run and Cl− deposition rate"; ALC17 (Q3-general): monthly spectral wave height 1.5–2.0 m ⇒ 100–200 mg Cl⁻/(m²·d) monthly salinity; surf-zone aerosol dominates near-ground coastal Cl⁻.
- Model suggestion (derived, not from a source): drive deposition with an onshore "run" R = Σ max(0, U − 3 m/s)·Δt over onshore sectors only (or Σ U·Δt for U > 3 m/s, per Meira), add storm pulses, and calibrate the site constant on measured annual Cl⁻ (Q3-KR2 table) rather than on mean wind speed.

### Q7d. Vertical profile with height (crane girders at 20–95 m)
- Meira et al. 2017 (above) [high]: wet-candle measurements "at heights lower than 10 m from the ground at a monitoring station 100 m far from the shoreline" (João Pessoa, Brazil, one year 2011–2012); highlights: "Marine aerosol salinity exponentially decreases with height from the ground"; model "Dep = a1·e^(a2·Σv3·t3/t) · e^(−b1·e^(b2·Σv3·t3/t)·h)" (h = height; coefficients not in the opened abstract). From their literature review: the decrease is "much stronger in the first metres from the ground level"; "this region of stronger variation is in the first 20 m or 30 m of altitude"; "Studies focused in higher layers show a slighter decrease tendency with the altitude"; mechanism: "large salt particles, with short residence time, do not rise much above the first metres over the sea".
- Morcillo 2000 intro [high]: "The concentration of marine aerosol decreases with altitude … For this reason the raising of a metal above ground level generally results in less corrosion".
- Sawant S.S., Wagh A.B., "Effect of height on the marine atmospheric corrosion of steel", *Corrosion Prevention & Control* 38(3) (1991) 75 — abstract, URL opened: https://trid.trb.org/View/439005 [high, abstract]: motivated by "cranes, helidecs and other aerial components of offshore platforms"; coastal and offshore stations, west coast of India; "The corrosion rates of steel at both the stations were found to decrease with increase in height above sea level"; anomaly at 2 m attributed to "the high wind speed which prevailed at this level, thereby preventing the s[a]lt from forming an electrolyte". (No numbers in the abstract.)
- Counter-evidence that height alone does not protect over open water [high]: East China Sea offshore platform, specimens on a "26 m-high offshore platform", Cl⁻ 81 mg/(m²·d), Q235 217.2 µm/a at 12 months, ISO category CX (CHE26, row C1). Gong 1997: the wind dependence is "weaker … for particles at higher altitudes" (the coarse, corrosive fraction is the short-lived one).
- Over Incheon waters (west coast), the outdoor sampler at the upper box girder of the Yeongjong suspension bridge recorded only 1.22 mg Cl⁻/(m²·d) dry gauze (S0) (LI11, row K1) — height above sea not stated in the paper.
- Model suggestion (derived, not from a source): for a quay crane, use a two-layer profile — a surf/near-ground component decaying exponentially with height (decay length of order metres to ~10 m, per Meira's <10 m data and the "first 20 m or 30 m" statement) plus a height-independent background of ocean-produced aerosol (well mixed above ~20–30 m). Under this assumption deposition at boom/girder level (20–95 m) ≈ background × exposure factor, so the girder sees less than the ground-level ISO 9225 value at a surf-exposed quay but not much less at a calm-water harbour where the background dominates. Calibrate the ratio with a wet candle on the crane at two heights before relying on it.

### Q7e. Far-field (regional) chloride deposition vs distance — Australia [high values / med applicability]
Source: Wilkins A., Crosbie R., Louth-Robins T., Davies P. et al., "Australian gridded chloride deposition-rate dataset", *Data in Brief* 42 (2022) 108189, PMC9065633. URL opened: https://www.ebi.ac.uk/europepmc/webservices/rest/PMC9065633/fullTextXML (§2.3).
- Model: D = A1·exp(−d/λ1) + A2·exp(−d/λ2) (D in kg ha⁻¹ yr⁻¹, d = distance to coast in km); least-squares fit to 367 observations: **A1 = 84 kg ha⁻¹ yr⁻¹, λ1 = 25 km; A2 = 17 kg ha⁻¹ yr⁻¹, λ2 = 279 km**. Quote: "The only strong correlation was between deposition and distance to the coast." The fast term "represents the wet and dry deposition in aerosols sourced from the ocean"; the slow term "the deposition of gaseous chloride".
- Conversion (derived): 1 kg ha⁻¹ yr⁻¹ = 0.274 mg m⁻² d⁻¹ ⇒ A1 ≈ 23, A2 ≈ 4.7 mg/(m²·d). These are bulk (mostly rain-borne) regional deposition values from water-chemistry data, not ISO 9225 near-shore values; useful only as the inland floor beyond the first few km (cf. the ≈100 m-scale decay in Q3).

## Unverified (remembered, not confirmed online)
Items referenced from earlier sections ("see Unverified") come first. Use these only as placeholders until checked against the cited primary source.

1. **ISO 9223:2012 DRFs for copper and aluminium** (Q1a) [low — from memory; best citation ISO 9223:2012 §8, Eqs. for Cu and Al]:
   - Copper (µm/a): r = 0.0053·Pd^0.26·exp(0.059·RH + f_Cu) + 0.01025·Sd^0.27·exp(0.036·RH + 0.049·T); f_Cu = 0.126·(T−10) for T ≤ 10 °C, −0.080·(T−10) for T > 10 °C.
   - Aluminium (g/(m²·a)): r = 0.0042·Pd^0.73·exp(0.025·RH + f_Al) + 0.0018·Sd^0.60·exp(0.020·RH + 0.094·T); f_Al = 0.009·(T−10) for T ≤ 10 °C, −0.043·(T−10) for T > 10 °C.
2. **ISO 9223:2012 SO₂ classes with concentration equivalents** (Q1b, Q1d) [low for P2/P3; P0/P1 deposition limits are verified in Q1d]: P0 Pd ≤ 4 mg/(m²·d) ≈ Pc ≤ 5 µg/m³; P1 4–24 ≈ 5–30; P2 24–80 ≈ 30–100; P3 80–200 ≈ 100–250. Ratio Pd/Pc = 0.8 at every boundary, which conflicts with the 0.67 factor quoted by PAN17 (Q1b). Partial support [verified]: the C3/C4/C5 descriptions in PAT24 Table 1 use SO₂ bands "5 µg/m3 to 30 µg/m3", "30 µg/m3 to 90 µg/m3", "90 µg/m3 to 250 µg/m3". Best citation: ISO 9223:2012, Annex (classification of pollution by SO₂).
3. **ISO 9223 TOW classes τ1–τ3** (Q1d) [med — partially verified]: τ1 ≤ 10 h/a, τ2 10–250, τ3 250–2500 (τ4 2500–5500 and τ5 > 5500 verified in Q1d). Support: Ebrahimi et al. 2025 (PMC12347768, opened) Table 3 "The category of time of wetness according to the US guidelines": "T1 | 10; T2 | 250; T3 | 2500; T4 | 5500; T5 | >5500" (h/year, upper bounds).
4. **Japanese weathering-steel rule: distances beyond which airborne-salt measurement may be omitted** (Q3-JP) [low — from memory]: Sea-of-Japan coast I (Hokkaido/Tohoku/Hokuriku zone) > 20 km; Sea-of-Japan coast II > 5 km; Pacific coast > 2 km; Seto Inland Sea coast > 1 km; Okinawa — no omission distance (measure everywhere). Best citation: Specifications for Highway Bridges Part II commentary (道路橋示方書・同解説 II 鋼橋編, JRA, 2002) and PWRI/JSSC/JASBC joint report on weathering steel bridges (1993); NILIM TN 828 (URL in Q3-JP, not opened).
5. **Lovett (1978)** (Q7b) [low]: R.F. Lovett, "Quantitative measurement of airborne sea-salt in the North Atlantic", *Tellus* 30 (1978) 358–364: ln C = 0.16·U + 1.45 (C in µg/m³ near ship-deck height, U in m/s). The exponential form is verified (Erickson 1986, Gong 1997); the coefficients are not. DTIC copies (ADA134792) were under maintenance when tried.
6. **Blanchard & Woodcock (1980)** (cited by ALC17 [51] and Morcillo 2000) [low]: over the open sea, sea-salt mass is roughly uniform with height through the mixed layer up to cloud base, with steep gradients only in the lowest metres/surf zone. Not opened; consistent with Meira 2017 statements in Q7d.
7. **Kure Beach 25-m vs 250-m lots** (Q4, row A1) [low]: classic LaQue/INCO data reportedly show carbon-steel loss on the 25-m (80-ft) lot several-fold (often quoted ≈10×) higher than on the 250-m (800-ft) lot. Best citation: F.L. LaQue, *Marine Corrosion: Causes and Prevention*, Wiley 1975. Which lot ISOCORRAG used is also unverified.
8. **Ambler & Bain (1955), Nigeria** (ALC17 [9], Fig. 7) [low]: steel corrosion on the Lagos surf coast falling by more than an order of magnitude between the beach (tens of metres) and ~1–2 km inland. Figure not read; original *J. Appl. Chem.* 5 (1955) 437 not opened.
9. **Distances to the sea of the ISOCORRAG and MICAT stations** in Q4a (Ostende, Salins de Giraud, Kure Beach, Limón, Cojímar, Arraial do Cabo, Ubatuba…) [—]: not in CHI17. Best citations: Morcillo M. (coord.), *Corrosión y protección de metales en las atmósferas de Iberoamérica* (MICAT, CYTED 1998); Knotkova D. et al., ISOCORRAG results, ASTM STP 1239 (1995).
10. **Chinese stations** (Q4) [—]: first-year Q235 rates at Wanning (Hainan), Xisha/Yongxing Is. and Qingdao not opened (ALC17 Fig. 44 shows Wanning kinetics only). Verified environment only (CHE26 Table 1, opened): South China Sea "28.8" °C, "77.8" %, "400" mg/(m²·d); Qingdao "12.3" °C, "72" %, "16.6" mg/(m²·d) (each citing other papers).
11. **Taiwan, Singapore** (Q4) [—]: no open source with site-specific first-year rates found within the search budget (e.g., the Taiwan atmospheric-corrosion map papers).
12. **Nha Trang, Vietnam — six sites at different distances and heights** [low]: PDF link returned 404 (https://vjol.info.vn/JSTD/vi/article/download/3459/3364/12745); search-result snippet only: "in S1 class by aerosol salinity and C3 class by corrosion rate of carbon steel after first year exposure test" (Bùi Văn Thảo et al., *Sci. Tech. Dev. J.* 13(1M) (2010) 26–34).
13. **Korean data not extracted** [—]: per-site 12-month carbon-steel values of the Korean national corrosion map (Kim Y.S. et al., CST 10(2) 2011, Fig. 9 — map graphics only); airborne chloride on Korean highway bridges incl. Seohae Grand Bridge (Jung H.M. et al., Proc. KCI Spring Conf. 2002, pp. 743–748, https://www.koreascience.or.kr/article/CFKO200211922760843.page — PDF text not extractable); height of the Yeongjong upper-girder sampler (row K1).
14. **Vera R. et al., "Effect of height on the corrosion of steel and galvanized steel in nontropical coastal marine environments", *Mater. Corros.* 71 (2020) 1125–1137** [—]: title verified in HER25 reference 53; content (height profile of corrosion near Valparaíso) not opened — high-priority source for Q7d.
15. **Surf-zone production and holistic transport models** [—]: de Leeuw G. et al., surf-zone source function, *J. Geophys. Res.* 105 (2000) 29397; Cole I.S. et al., "Holistic model for atmospheric corrosion", *Corros. Eng. Sci. Technol.* 38 (2003); Meira G.R. et al., *Corros. Sci.* 50 (2008) 2724 (sea-salt transport/deposition model). None opened.
16. **Skyward vs groundward faces of boldly exposed panels** (Q5c) [low]: general literature states that groundward faces corrode more than skyward faces in marine atmospheres (no rain rinsing, longer wetness), the reverse of many industrial sites; no ratio verified. Best citation: ISO 8565/ISO 9226 practice and ASTM STP 1239 (ISOCORRAG).
17. **Dry-gauze → wet-candle factor 2** (Q4c) [med]: only the secondary citation of JIS Z 2382 in LI11 was opened; the JIS itself was not.
18. **JWTC per-site one-year rates** (Q4a J5) [—]: only the envelope (OHA18 Table 3) and site list/distances (Table 1) were verified; per-site values are in the JWTC data report cited as OHA18 ref. 8.
19. **Abrasive water-jet cleaning** (Q5e) [low]: a search snippet (Taylor & Francis open chapter, Toyota/Kainuma et al.) states the method "significantly reduced surface chloride concentration, consistently achieving levels below 30 mg/m²" on a steel I-girder bridge; chapter not opened.
