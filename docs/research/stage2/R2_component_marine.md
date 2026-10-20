# R2 — Component durability in marine / salt environments (염해 방지 preset research)

Status: FINAL (2026-10-20). 22 web searches used (target was 15). About 45 URLs opened; PDFs that WebFetch could not parse were saved locally and text-extracted with pdftotext.
Rules: each number has a URL I actually opened, plus a short verbatim quote (25 words or fewer) or a table reference. Items I could not open are marked "snippet only" or listed under **Unverified**.
Confidence tags: **[high]** = the standard's table as reproduced by a reputable body, a primary document, or peer-reviewed work. **[med]** = reputable secondary source or association guide. **[low]** = vendor claim, blog, or single weak source.

---

## 0. Key numbers at a glance (details and quotes in the sections below)

| Topic | Value | Conf | Section |
|---|---|---|---|
| ISO 12944-1:2017 durability L / M / H / VH | ≤7 / 7–15 / 15–25 / >25 years | high | 1b |
| ISO 12944-2 steel first-year loss C4 / C5 / CX | 50–80 / 80–200 / 200–700 µm | high | 1a |
| ISO 12944-5:2018 C5, zinc-rich primer: min coats / NDFT for L, M, H, VH | 2/160, 2/200, 3/260, 3/320 µm | high | 1e |
| ISO 12944-6/-9 lab tests: C5-VH / CX cyclic ageing | 2688 h / 4200 h | high | 1c |
| C5-M and C5-I merged into C5 (2017); CX = offshore | — | high | 1e |
| Paint maintenance cycle (2-coat EP/PU, C3) over black steel / over HDG | touch-up 11 / 22 y; full repaint 20 / 51 y | med | 1g |
| Zinc first-year loss C3 / C5 / CX | 0.7–2.1 / 4.2–8.4 / 8.4–25 µm/a | high | 2a |
| HDG 85 µm life to first maintenance, CX / C5 / C4 | 3–10 / 10–20 / 20–40 years | high | 2b |
| Duplex synergy factor (extreme marine … non-aggressive) | 1.4–1.6 … 2.0–2.3 (AGA); 1.4–2.7 (HDGASA) | med | 2c |
| Relubricated vs dry rope service life | 2–3× (Chevron, vendor) | low-med | 3c |
| IEEE 43 IR1min at 40 °C: random-wound or <1 kV / form-wound post-1970 | 5 MΩ / 100 MΩ; PI ≥ 2.0 | med | 4a |
| Space heater target | winding 5–10 °C above ambient | med | 4b |
| Motor failure shares (IEEE-IAS 1985): bearing / stator / rotor / other | 44 / 26 / 8 / 22 % | med | 4d |
| ISA-71.04 copper reactivity G1 / G2 / G3 / GX | <300 / <1000 / <2000 / ≥2000 Å per 30 days | high | 5a |
| ISA humidity rule | +1 severity level per +10 % RH above 50 % | high | 5b |
| Cantley water-in-oil bearing life | L = (100/ppm)^0.6, valid 25–400 ppm | high | 6a |
| Typhoon Maemi, Busan | 8 cranes toppled + 3 derailed; Busan wind 42.7 m/s | high/med | 7a |
| Koper port tender wind regimes | operate ≤22; limited 22–35; parked 35–44; stowed >44 m/s | high | 8a |

---

## Q1. Protective coatings (ISO 12944, ISO 4628-3)

### 1a. Corrosivity categories (ISO 12944-2:2017, which cites ISO 9223): first-year steel thickness loss
Source: steelconstruction.info, "Corrosion protection". https://www.steelconstruction.info/Corrosion_protection
Quote (footnote): "The thickness loss values are after the first year of exposure. Losses may reduce over subsequent years."

| Cat | Steel loss, year 1 (µm) | Exterior example (paraphrased) | Conf |
|---|---|---|---|
| C1 very low | ≤1.3 | (interior: heated, clean) | high |
| C2 low | >1.3–25 | low pollution, mostly rural | high |
| C3 medium | >25–50 | urban/industrial, moderate SO2; low-salinity coast | high |
| C4 high | >50–80 | industrial; moderate-salinity coast | high |
| C5 very high | >80–200 | industrial, high humidity; high-salinity coast | high |
| CX extreme | >200–700 | offshore, high salinity; tropical/sub-tropical | high |

Mass-loss values for C5 and CX (steel and zinc) are in section 2a, from the Hempel booklet.

### 1b. Durability ranges (ISO 12944-1:2017)
- Cloverdale Paint, "ISO 12944-2017 Systems Guide for C4, C5 and CX Extreme" (2021 PDF): https://www.cloverdalepaint.com/docs/default-source/industrial/cloverdale-paint-iso12944-2017-systems-guide-forc4-c5-and-cx-extreme-2021.pdf
  - Quote: "Low (L): up to 7 years / Medium (M): 7-15 years / High (H): 15-25 years / Very High (VH): > 25 years"
  - Quote: "The durabilities represent the approximate service time expectancy"
- Hempel, "ISO 12944:2018 – Summary of main changes" (the text is in English despite the FI URL): https://www.hempel.com/fi-fi/-/media/Files/Local/EU-condensed/Brochures/FI/ISO-Booklet-FI.pdf
  - The booklet compares the old and new schemes: L "2 to 5 years" became "up to 7 years"; M "5 to 15 years" became "7-15 years"; H "> 15 years" became "15-25 years". VH ("> 25 years") is new.
- Axalta, "Determining environment & durability": https://secure.axalta.com/powdercoatings_gb/en_GB/our-products/anticorrosion/environment.html
  - Quote: "Durability is not a guaranteed period." The page adds that the contractual warranty is generally shorter than the durability.

| Durability | Years (2017) | Years (1998 edition) | Conf |
|---|---|---|---|
| L | up to 7 | 2–5 | high |
| M | 7–15 | 5–15 | high |
| H | 15–25 | >15 | high |
| VH | >25 | (did not exist) | high |

Definition of "durability" (time to first major maintenance, reached at Ri 3 per ISO 4628-3): I could not find this in any opened page. See **Unverified U1–U2**.

### 1c. Laboratory pre-qualification (ISO 12944-6:2018 and ISO 12944-9), in hours
Source: the Cloverdale 2021 PDF, "LABORATORY TEST REQUIREMENTS" table (re-extracted in raw order).
Columns: ISO 6270-1 water condensation / ISO 9227 neutral salt spray / ISO 12944-6 Annex B cyclic ageing.

| C5 durability | Condensation (h) | NSS (h) | Cyclic (h) | Conf |
|---|---|---|---|---|
| Low | 240 | 480 | – | high |
| Medium | 480 | 720 | – | high |
| High | 720 | 1440 | 1680 | high |
| Very High | – | – | 2688 | high |
| CX (ISO 12944-9) | – | – | 4200 | high |

The table does not say whether the High row means "1440 NSS and 1680 cyclic" or "or". Check the standard for that.

### 1d. Example manufacturer systems (Cloverdale, same PDF) [med, single manufacturer]
- C5.07 (High): zinc-rich epoxy 60–80 µm + epoxy 100–150 µm + PU 30–100 µm; total "260" µm.
- C5.08 (Very High): zinc-rich epoxy 60–80 µm + epoxy + PU; total "320" µm.
- CX: zinc-rich epoxy 50–75 µm + epoxy 150–250 µm + epoxy 150–250 µm; total "350" µm. Surface preparation "Sa3 (SSPC-SP5)".
- Teknos CX system, zinc-rich EP 60 + EP 160 + PUR 60 = 280 µm: snippet only (search result; https://teknos.com/contentassets/5381d7ae1eb84838b70f7e766405bb8e/cx-280-m-offshore-carbon-steel-.pdf not opened) [low].

### 1e. ISO 12944-5:2018 minimum number of coats (MNOC) and nominal DFT (NDFT)
Source: Hempel booklet, p.11 (link above). I re-extracted the table from the PDF with `pdftotext -table`.
Caption quote: "Summary of the minimum number of coats and NDFT of the paint system depending on durability and corrosivity"
- Zn(R) = zinc-rich primer (ESI/EP/PUR). Misc = other primer (EP/PUR/ESI).
- The AK/AY column is omitted; it is not offered for C5.
- Cells are MNOC / NDFT in µm.

| Cat | L Zn(R) | L Misc | M Zn(R) | M Misc | H Zn(R) | H Misc | VH Zn(R) | VH Misc | Conf |
|---|---|---|---|---|---|---|---|---|---|
| C4 | 1/60 | 1/120 | 2/160 | 2/180 | 2/200 | 2/240 | 3/260 | 2/300 | high |
| C5 | 2/160 | 2/180 | 2/200 | 2/240 | 3/260 | 2/300 | 3/320 | 3/360 | high |

- So C5-VH needs at least 3 coats and 320 µm NDFT with a zinc-rich primer. This matches Cloverdale C5.08. Your guess of "280–320 µm" for C5-M is consistent: 260 µm (H) to 320 µm (VH).
- From the same booklet: "C5-M and C5-I merged. C5 is now for land-based structures." It also says "The new CX Extreme category is for offshore structures (as outlined in Part 9)". ISO 12944-9 replaces ISO 20340. [high]

### 1f. Siting a crane in a category
- The GAA chart (section 2b, AS/NZS 2312.2) gives distance rules:
  - **CX:** "within 50 metres of the shoreline of coasts exposed to surf or very rough seas".
  - **C5:** 50–500 m from a surf coast, or "up to 50 metres from the shoreline of a semi-sheltered coast".
  - **C4:** "within 50 metres of the shoreline of a sheltered bay".
- The same chart warns: "Surfaces which are sheltered and not rain-washed … where chlorides are deposited, can experience a higher rate of corrosion". This matters for crane undersides, girder interiors and machinery-house floors. [high]
- The KSCRANE blog maps "C5-M" ("Near-sea, high salt spray, offshore") to "Port terminals, offshore platforms, ship deck cranes". https://www.kscranegroup.com/posts/anti-corrosion-coating-for-overhead-crane/ [low]

### 1g. Published maintenance-cycle data (touch-up, maintenance repaint, full repaint)
Source: American Galvanizers Association case study, "Galvanized Duplex System Return on Investment" (2014). It cites "NACE Paper #C2014-4088 KTA Tator, Helsel, Lanterman, Reina, 2014". https://galvanizeit.org/uploads/publications/Galvanized_Duplex_System_Return_on_Investment.pdf
Context: two-coat epoxy/polyurethane system; "Planned project life of 60 years in an industrial service life environment (ISO C3)". Table 2 is "Time to Practical Maintenance of Paint (Years)".

| Event | Over black steel (y) | Over HDG steel, 2.0× (y) |
|---|---|---|
| Original painting | 0 | 0 |
| Touch-up repaint | 11 | 22 |
| Maintenance repaint | 14 | 29 |
| Full repaint | 20 | 51 |
| Touch-up repaint | 31 | 58 |
| Maintenance repaint | 34 | not required |
| Full repaint | 40 | not required |
| Touch-up / maintenance repaint | 51 / 54 | not required |

- The black-steel cycle repeats every 20 years.
- The case study reports life-cycle cost of $21.76/ft² (black steel) vs $12.95/ft² (duplex), a "40% Cost Savings". [med]
- This is C3 data, not marine. Use it for the shape of the cycle (touch-up ≈ 0.55 T, maintenance ≈ 0.7 T, full repaint ≈ T), not for absolute marine values.

### 1h. Stochastic coating-degradation models (found, but no parameters)
- Nicolai, Dekker, van Noortwijk, "A comparison of models for measurable deterioration: an application to coatings on steel structures". Econometric Institute Report EI 2006-28 (abstract opened): https://repub.eur.nl/pub/7974
  - It was later published in Reliability Eng. & System Safety 92(12):1635–1650 (2007), according to search results.
  - It fits three models to two Dutch datasets (expert judgement, and inspections): Brownian motion with non-linear drift, a non-stationary gamma process, and a two-stage "hit-and-grow" process.
  - Parameters are estimated by least squares (expert data) and maximum likelihood (inspection data).
  - The PDF (https://repub.eur.nl/pub/7974/ei2006-28.pdf) uses font encoding that cannot be extracted, so **I have no parameter values** [high for the method, no numbers].
- No opened source gave field "time to first maintenance" for port cranes specifically. The nearest things are the Koper 10-year coating warranty (8a) and the ZPMC recoating programme (8b).

---

## Q2. Hot-dip galvanizing (HDG) and duplex systems

### 2a. Zinc corrosion rates (ISO 9223 / ISO 12944-2, first year)
- **C3:** "For zinc, ISO C3 is defined as 0.7 to 2.1 µm per year." Source: V&S Galvanizing, https://www.hotdipgalvanizing.com/technical-resources/estimating-hdg-corrosion-rates-using-iso-9223-categories-c1-to-cx [high]
- **C5 and CX:** Hempel booklet, table "New and adjusted corrosivity categories" (link in 1b). Quote: "Significant difference between C5 and CX".

| Cat | Steel g/m² | Steel µm | Zinc g/m² | Zinc µm | Conf |
|---|---|---|---|---|---|
| C5 very high | >650–1,500 | >80–200 | >30–60 | >4.2–8.4 | high |
| CX extreme | >1,500–5,500 | >200–700 | >60–180 | >8.4–25 | high |

- **C4 (>2.1–4.2 µm/a zinc):** snippet only, from a search result for a PMC table (https://pmc.ncbi.nlm.nih.gov/articles/PMC8467660/table/materials-14-05219-t001, blocked by reCAPTCHA) [med, because it agrees with 2b].
- These are first-year rates. For multi-year life, the life-to-first-maintenance table below is the better input.

### 2b. HDG life to first maintenance (LtFM)
Source: Galvanizers Association of Australia chart, based on AS/NZS 2312.2, v2.1 Aug 2026. https://gaa.com.au/wp-content/uploads/2026/08/Life-to-First-Maintenance-Chart-Australia-v2.pdf
Quote: "steel with an initial coating thickness of 85 µm in a C4 corrosivity zone has an expected life to first maintenance of 20 - 40 years."

| Coating (µm) | CX | C5 | C4 | C3 | C2 | Conf |
|---|---|---|---|---|---|---|
| 125 | 5–15 | 15–30 | 30–60 | 60–>100 | >100 | high |
| 85 (steel >6 mm) | 3–10 | 10–20 | 20–40 | 40–>100 | >100 | high |
| 70 (steel 3–6 mm) | 2–8 | 8–16 | 16–33 | 33–>100 | >100 | high |
| 55 (steel 1.5–3 mm) | 2–6 | 6–13 | 13–26 | 26–78 | 78–>100 | high |
| 45 (steel <1.5 mm) | 1–5 | 5–10 | 10–21 | 21–64 | 64–>100 | high |

- The chart says its estimates are "very conservative estimates when the steel surface is fully rain washed".
- Implied zinc rates (my arithmetic): 85 µm over 10–20 years gives 4.25–8.5 µm/a in C5; over 3–10 years gives 8.5–28 µm/a in CX. Both agree with 2a.

### 2c. Duplex synergy (HDG + paint)
**Source A:** AGA, Galvanize It! seminar. https://www.galvanizeit.org/duplex-systems-gi-seminar/why-duplex/synergistic-effect/
- Quote: "1.5 to 2.3 times the sum of the expected life of each system alone."
- Formula: "Mduplex = Duplex Multiplier x (Mgalvanizing + Mpaint)", where M = time to first maintenance.
- AGA's own example: 50-year HDG plus 10-year paint gives "90 to 138 years".

| Environment (AGA) | Multiplier | Conf |
|---|---|---|
| Extreme marine/pollution | 1.4–1.6 | med |
| Sea water (immersion) | 1.5–1.6 | med |
| Industrial marine | 1.8–2.0 | med |
| Non-aggressive | 2.0–2.3 | med |

**Source B:** Hot Dip Galvanizers Association Southern Africa, "Duplex Coatings – An Overview" (PDF, 2009). https://www.hdgasa.org.za/wp-content/uploads/2015/09/Overview-of-Duplex-Coatings.pdf
- Quote: "The synergy factors vary from 1.4 in the extreme corrosive environments to 2.7 in less-aggressive environments."
- C5 example: "1.4 x (15 years + 10 years) = 35 years".
- Mechanism: zinc corrosion products "have a volume, which is only 15-20% more than the volume of zinc", so they seal pores, whereas rust occupies about twice the steel's volume. [med]

Origin of the factor: Source B credits the term "Duplex Coating" to Jan van Eijnsbergen (early 1950s). I could not trace the 1.5–2.3 range to his 1994 book.

---

## Q3. Steel wire ropes in marine / salty environments

### 3a. Corrosion and fretting fatigue in seawater (peer-reviewed; direction only, no magnitude)
**Takeuchi, M. (1990),** "Fretting and fatigue of a roping steel in seawater", J. Shimonoseki Univ. Fisheries 38(2/3):71–131. Wire UTS 1770 MPa. https://fra.repo.nii.ac.jp/records/2011523
- "Fretting fatigue in seawater had a serious damaging effect on the steel wire". Life was reduced at stresses well below the fatigue limit in air.
- "A galvanised coating significantly improved the fatigue strength in seawater". In air, however, galvanizing lowered fatigue strength somewhat, which the author "attributed to the existence of the brittle Fe-Zn alloy layer."
- Cathodic protection "had a dramatic effect in restoring the fatigue strength". It was less effective once cracks were deeper than about 30 µm. [high]

**Heller, Matanzo, Metcalf (1972),** "Axial fatigue of wire rope in sea water", Catholic Univ. of America / NTIS. https://trid.trb.org/View/5540
- "In marine applications of wire rope, fatigue failure is accelerated by the corrosive attack of sea water."
- Wire, strand and rope results showed no predictive relationship to each other. [high]

### 3b. Galvanized vs bright rope
- Patent US7272921 reports 150,000 vs 200,389 bend-over-sheave cycles (+34 %) for galvanized outer wires. Snippet only; not opened [low].
- Koper tender (8a) specifies "Galvanized and plastificated steel core" ropes. That is current port practice. [high]

### 3c. Relubrication
- Chevron Marine Products, "Information Bulletin 9: Wire Rope Lubrication" (2019). Text extracted from the PDF: https://www.chevronmarineproducts.com/content/dam/chevron-marine/technical-bulletins-2020/09%20Wire%20Rope%20Technical%20Bulletin.pdf
  - "Rope tests show that the service life of a regularly relubricated wire rope is two to three times longer than that of a dry rope." [low-med, vendor]
  - Intervals are "dictated by various factors, such as the type of lubricant used, rope storage conditions, and method and frequency of use." No numeric interval is given.
  - It prefers a high-pressure lubricator, because painting and swabbing "rarely penetrate the lubricant down to the core".
- Chesterton Case Study 062 (01/2020), marine port cranes: https://chesterton.com/Files/Product%20Assets/Lubricants/EN350514_IL_MRO_CS062_Lubrication_Extends_Wire_Rope_Life.pdf
  - With ordinary grease the ropes needed "Monthly replacement of wire ropes".
  - After switching to internal plus external lubricants: "decreased from 12 times a year to twice a year. A 600% increase in service life." [low, vendor case]
- Solazzi & Donzella (2024), relubrication bending-fatigue study: listing only (https://www.oipeec.org/product/effect-of-re-lubrication-in-extending-service-life-of-steel-wire-ropes/). The repository returned 403, so I have no numbers.
- ISO 4309 corrosion discard criteria: not verified. See **U5**.

---

## Q4. Electric motors in humid / salty environments

### 4a. IEEE 43 minimum insulation resistance (IR1min corrected to 40 °C) and polarization index (PI)
Source: TestGuy wiki, "Field Testing of Motors" (a secondary summary of IEEE 43). https://wiki.testguy.net/t/field-testing-of-motors/249 [med]

| Winding | IR1min at 40 °C | Quote |
|---|---|---|
| Windings made before 1970 | kV + 1 MΩ | "minimum insulation resistance of 1 Megohms per kV + 1 Megohm" |
| Form-wound coils after 1970 | 100 MΩ | "should have a minimum insulation resistance of 100 Megohms" |
| Random-wound, or form-wound below 1 kV | 5 MΩ | "should have a minimum insulation resistance of 5 Megohms" |
| PI, all classes except A | ≥ 2.0 | NETA: "polarization index values shall not be less than 2.0" |
| PI, Class A | ≥ 1.5 | "the odd-ball insulation in IEEE 43" |

- Temperature correction factors "use a 40°C base temperature".
- A search summary says IEEE Xplore marks IEEE 43-2013 "inactive - reserved". Not opened.
- Crane hoist and travel motors are mostly random-wound LV motors, so the 5 MΩ / PI 2.0 thresholds apply.

### 4b. Anti-condensation (space) heaters
- Ryan, "Preventing condensation in 3-phase AC motors", EC&M, 1 Nov 1996. https://www.ecmweb.com/content/preventing-condensation-3-phase-ac-motors
  - "maintaining a winding temperature 5 [degrees] C to 10 [degrees] C above the surrounding air temperature." [med]
  - The heater is wired so that the circuit "energizes the heater when power to the motor is disconnected."
  - Trickle heating (low voltage on the winding) is "a viable alternative in a pinch", but "not practical as a permanent solution".
- Search snippets (not opened) say the same 5–10 °C: an EASA trade-press article and a NEMA application-manual copy. A forum post says IEEE 43-1974 suggested about 5 °C. These only corroborate [low].

### 4c. IP codes (IEC 60529)
Source: Wikipedia, "IP code". https://en.wikipedia.org/wiki/IP_code [high for definitions]

| Digit | Meaning and test |
|---|---|
| 1st = 5 | Dust protected: "Ingress of dust is not entirely prevented, but it must not enter in sufficient quantity to interfere with" safe operation |
| 1st = 6 | "No ingress of dust; complete protection against contact (dust-tight)." Vacuum test of up to 8 h |
| 2nd = 5 | 6.3 mm nozzle, "12.5 litres per minute Pressure: 30 kPa … at distance of 3 meters", at least 3 min |
| 2nd = 6 | "powerful jets (12.5 mm … nozzle)", "100 liters per minute … 100 kPa … at distance of 3 meters" |
| 2nd = 7 | Immersion for 30 min, lowest point 1,000 mm below the surface |

- IP55 = dust-protected + water jets. IP56 = dust-protected + powerful jets. IP66 = dust-tight + powerful jets.
- My inference, not from a source: no IP test includes salt fog or condensation. Corrosion resistance and heaters have to be specified separately.

### 4d. Motor failure shares and failure rates
Source: ReliaMag, "Electric motor failure statistics" (secondary compilation). https://reliamag.com/guides/electric-motor-failure-statistics/ [med]

| Survey | Bearing | Stator/winding | Rotor | Other | Sample |
|---|---|---|---|---|---|
| IEEE-IAS 1985 | 44 % | 26 % | 8 % | 22 % | 5,085 motor unit-years, 360 failures; >200 hp, ≤13.8 kV |
| EPRI 1982 | 41 % | 36 % | 9 % | 14 % | about 4,800 utility motors ≥100 hp; 1,227 failures on 872 motors |

Failure rates from the same page:
- IEEE 1985: "0.0708 failures per unit per year".
- EPRI: about 3.1–3.5 per 100 motor-years.
- Thorsen & Dalva 1995: 0.0202 per unit-year with embedded thermal protection, vs 0.0707 with minimal protection.
- IEEE 1985 by maintenance quality, counterintuitive as reported: "excellent" plants 0.1115 vs "fair" plants 0.0719.

What is not quantified: neither page gives a numeric share for moisture or salt contamination.
- ReliaMag "Large motor reliability studies" (https://reliamag.com/articles/large-motor-reliability-studies/) gives EPRI apparent causes: misapplication or misoperation 34.1 %; no cause specified 50.2 %.

---

## Q5. Electronics and electrical cabinets (ISA-71.04)

### 5a. Severity levels
Source: Purafil paper, "Combination Corrosion Coupon Testing Needed for Today's Control Equipment" (hosted by LBNL; it reproduces the ISA-71.04-1985 table). https://datacenters.lbl.gov/sites/default/files/Combination%20Corrosion%20Coupon%20Testing%20Needed%20for%20Today's%20Control%20Equipment.pdf
Quote: copper values are "Measured in angstroms after one month's exposure".

| Level | Name | Cu (Å per 30 days) | Ag (Å per 30 days, 2013 edition) | SO2+SO3 (ppb) |
|---|---|---|---|---|
| G1 | Mild | <300 | <200 | <10 |
| G2 | Moderate | <1000 | Unverified (U4) | <50 |
| G3 | Harsh | <2000 | Unverified (U4) | <300 |
| GX | Severe | ≥2000 | Unverified (U4) | ≥300 |

- Copper and SO2 values: [high]. Silver G1: [med], from the Purafil text in 5b.

### 5b. What the levels mean, and the humidity rule
Source: Purafil, "ISA environmental classifications changes / new requirements". https://www.purafil.com/isa-environmental-classifications-changes-new-requirements/
- G1: "corrosion is not a factor in determining equipment reliability."
- G2: "effects of corrosion are measurable and may be a factor in determining equipment reliability."
- G3: "high probability that corrosive attack will occur."
- GX: "only specially designed and packaged equipment would be expected to survive."
- Silver G1: "should be established at 200 angstroms / 30 days."
- **Humidity rule:** copper reactivity "can be expected to be increased by one level for each 10% increase in relative humidity above 50%". It also names a "relative humidity rate of change greater than 6% per hour". The same rule appears in the LBNL PDF. [high]

### 5c. Port-crane practice
- The Koper tender (8a) requires the electric control room to have air conditioning with "2 units with full capacity each". The cabin needs filtered ventilation to "provide overpressure". [high]
- No opened source gave e-house pressurisation figures (Pa) or filter classes.

---

## Q6. Bearings and gearboxes: water in oil

### 6a. Cantley (1977)
Source: STLE TLT, "From the Editor" (June 2021). https://stle.org/files/TLTArchives/2021/06_June/From_the_Editor.aspx
- Test setup: "tapered roller bearings", "an SAE 20 oil with water concentrations of 25, 100 and 400 ppm", "2.03 GPa maximum compressive stress at 2,700 rpm", 66 °C.
- Results were normalised to a relative life of 1 at 100 ppm. The fitted relation is **L = (100/X)^0.6**, where X is water in ppm.
- Citation: ASLE Trans. 20(3):244–248, DOI 10.1080/05698197708982838.
- The editor warns against extrapolating beyond the tested 25–400 ppm, or to free or emulsified water, or to other speeds, loads or oils. [high]
- Values from the formula (my arithmetic): 25 ppm → 2.30; 50 → 1.52; 100 → 1.00; 200 → 0.66; 300 → 0.52; 400 → 0.44.

### 6b. Corroborating figures
- Eliot (Mobil), Machinery Lubrication: https://www.machinerylubrication.com/Articles/Print/76
  - "as little as 0.04% (400 PPM) water … can cut the fatigue life of tapered roller bearings by as much as 48%". [med]
  - Water can "cut bearing life by as much as 80%" (no conditions given). [low]
  - "it is still incredibly difficult to keep water levels below 300 PPM in many lube systems". [med]
- Dinwiddie, Wind Systems Mag. (10 Apr 2014): https://www.windsystemsmag.com/excessive-water-in-gear-oil-shortens-bearing-life/
  - US Navy data: "when water increases from 50 to 500 ppm … a reduction in L10 bearing life by a factor of three". [med]
  - Gearbox limits: IEC 61400-4 work moving monitor/action levels from 500/1,000 to 300/600 ppm. The Danish Wind Standard uses "200 ppm water for monitor and 400 ppm for action". [med]
- Desiccant breathers and typical ppm in humid port service: no source found (U10).

---

## Q7. Rail clamps, storm pins and tie-downs; storm accidents

### 7a. Typhoon Maemi (12 Sep 2003), Busan

| Fact | Value | Source (opened) | Quote | Conf |
|---|---|---|---|---|
| Cranes affected (Sinkamman + Jaseongdae) | 8 toppled + 3 derailed | Kyunghyang Shinmun, 13 Sep 2003. https://www.khan.co.kr/article/200309132224031 | "대형 크레인 8기가 줄줄이 넘어지고 3기가 궤도를 이탈" | high |
| Collapsed at Sinkamman (DBCT) | 6 | Yulchon LLC case note. https://www.yulchon.com/ko/resources/deals-and-cases-view/1078/page.do | "부산 신감만부두 컨테이너 크레인 6기가 붕괴한 사고" | high |
| Court finding | Not force majeure: crane manufacturing fault (Hanjin HI) plus quay construction fault (Daewoo E&C). Operator fault 20 % at first instance, 10 % on appeal. Damages 238억 → 273억 KRW | Yulchon (same page) | "그 과실비율이 10%로 대폭 축소" | high |
| Sinkamman share of Busan throughput | 25 % | Khan (same) | "부산항 전체 물량의 25%를 처리" | high |
| Recovery time | more than 1 year | Khan (same) | "크레인 해체와 제작에만 1년 이상이 걸릴 것으로" | high |
| Wind at Busan | 42.7 m/s. The storm-wide figures (max gust 60 m/s, daily max 51.1 m/s) were given without a location | E-Today (2022). https://www.etoday.co.kr/news/view/2170218 | "부산에서는 초속 42.7m, 통영에서는 초속 43.6m의 강풍이 기록됐습니다" | med |
| Mechanism | Wind load grows with speed², but tie-down tension grows much faster once design wind is exceeded. The authors call this the "most probable" cause at Shingamman and recommend higher safety factors for mooring devices than for the structure | Kim D.H. et al., ISOPE-2004 pp. 683–686 (KIOST record). https://sciwatch.kiost.ac.kr/handle/2020.kiost/31852?mode=full | (abstract, paraphrased) | high |

### 7b. TT Club / PEMA / ICHCA guidance
- **TT Club press release, 16 Oct 2008:** https://ttclub.com/news-and-resources/news/article/press-release-tt-club-calls-for-more-stringent-crane-tie-down-procedures
  - Wind damage to quay cranes is "the biggest weather related cost to terminals".
  - "many of the service brakes and park brakes being inoperative due to poor maintenance".
  - Cranes need "storm pins and tie-downs of sufficient number and size to hold a crane structure stationary". [high]
- **TT Talk, 3 Nov 2020:** https://www.ttclub.com/news-and-resources/news/article/tt-talk-mitigating-the-impact-of-windstorms-in-container-terminals/
  - "while many terminals have storm-pins, storm tie-downs are less frequently specified."
  - Brakes "are not, however, an acceptable alternative to pins or tie-downs for forecast windstorms."
  - "Well designed and maintained braking systems, can significantly help in conditions of sudden wind microbursts." [high]
- **TT Club / PEMA / ICHCA, "Recommended Minimum Safety Features for Quay Container Cranes" Rev.1** (reported by AJOT, 20 Nov 2019): https://www.ajot.com/news/quay-cranes-minimum-safety-features-updated
  - Recommends "storm pins on both waterside and landside, as well as crane tie-downs on each corner of the crane".
  - Recommends an "anemometer … with an appropriate shutdown function". [high]
- **Gap:** no opened source gave numeric inspection or functional-test intervals for rail clamps, storm pins or tie-downs, or statistics on corrosion-related failures of them (U7).

---

## Q8. Port-crane anti-corrosion specification practice

### 8a. A real port tender: Luka Koper (Slovenia), 32 t grab ship-unloader, specification dated 26.02.2018
Source: https://www.luka-kp.si/wp-content/uploads/2018/05/Technical-specification-32t-x-40-18-10.pdf (text extracted locally) [high, primary document]

| Item | Spec (verbatim) |
|---|---|
| Ambient | "-12 .... + 45°C"; "Humidity 10 - 90 %" |
| Wind | operational "0 ... 22 m/s"; limited performance "22 ... 35 m/s"; "wheels in parking position 35 ... 44 m/s"; "stowed condition > 44 m/s" |
| Storm devices | "Ram type rail brake"; "Storm pin - on both legs ( >180km/h)" (180 km/h = 50 m/s) |
| Electric control room | air conditioning, "2 units with full capacity each", channelised airflow, alarms on the CMS |
| Cabin | "Ventilation system with air filtration to provide overpressure in the cabin" |
| Ropes | "Galvanized and plastificated steel core" (Casar/Teufelberger or equivalent) |
| Paint | "Thickness of dry layer Min. 250 [µ]m" (the µ glyph was lost in extraction); RAL 5017 |
| Coating warranty | "Warranty for anticorrosion protection Min. 10 years" |
| Access steel | stairs, walkways, ladders and handrails "Hot-dip galvanised" |
| Lubrication | "Automatic centralized lubrication system" |
| Floodlights | "IP65" |
| Warranties (37.1, re-read with `pdftotext -table`) | mechanical/electrical components "Min. 2 years"; steel structure "Min. 5 years"; anticorrosion protection "Min. 10 years". Spare parts must be available for "Min. 10 years" |

Luka Rijeka crane spec (2020), https://lukarijeka.hr/wp-content/uploads/2020/07/specification-for-cranes.pdf:
- "The maximum wind speed at which the crane can work is 20 m/s - 72 km/h"
- "Cabin with air conditioning (heating-cooling)" [high]

### 8b. ZPMC STS cranes at German terminals: remedial recoating (Maritime Executive, 30 Dec 2024)
Source: https://maritime-executive.com/corporate/hamburg-terminals-opt-for-steelpaint-s-new-corrosion-protection-system
- ZPMC STS cranes in Hamburg, Bremerhaven and Wilhelmshaven are being recoated with a ZPMC-approved single-pack moisture-cure PU (Stelcatec).
- Vendor claims: "corrosivity category C5", "meeting ISO12944 standards for 25 years' protection", DFT "between 80 and 120µm".
- The work runs over "the next two to three years". Rationale: "Taking a ship-to-shore crane out of commission costs the terminal money." [med]

### 8c. Crane-maker blog (KSCRANE, 10 Jun 2026) [low]
- C5 systems "start at 300μm total DFT, with 4–6 coats".
- Field anecdote: a 160 µm C3 system in a coastal plant rusted at girder welds in "less than five years".
- Another anecdote: on a coast, "a C3 coating would start blistering in three years".

### 8d. Not found in any opened source
- Fresh-water wash frequency
- Stainless-steel fastener requirements
- Port-crane inspection and touch-up intervals
- IP rating and heater clauses for crane motors (U8, U9)

---

## Unverified (from memory, or seen only in search snippets; do NOT use as sourced)

| # | Item | Value I remember / saw in a snippet | Why unverified |
|---|---|---|---|
| U1 | ISO 4628-3 rust grades (rusted area %) | Ri0 0, Ri1 0.05, Ri2 0.5, Ri3 1, Ri4 8, Ri5 40–50 | Preview PDFs (ANSI 403, UNI front matter only) and three searches did not show the table. The 2024 edition renames "rusted area" to "corroded area" (seen in the UNI preview) |
| U2 | Definition of durability in ISO 12944-1 | Expected life to first major maintenance painting, normally due at Ri 3 (ISO 4628-3) | Not stated in any opened page |
| U3 | ISO 9223 / 14713-1 zinc rates for C1, C2, C4 | ≤0.1; 0.1–0.7; 2.1–4.2 µm/a | C4 seen only in a snippet; C1 and C2 from memory |
| U4 | ISA-71.04-2013 silver limits for G2, G3, GX | <1000, <2000, ≥2000 Å per 30 days | Only the G1 value (200) was verified |
| U5 | ISO 4309:2017 corrosion discard criteria | Severity rated in 20 % steps up to discard: external corrosion from superficial (can be wiped off) through rough surface to heavy pitting and slack wires; internal corrosion means discard | Standard not accessible |
| U6 | Rocol: lubricated rope bend-fatigue life "up to 60% longer" | snippet | Page returned 404 |
| U6b | Marine Log: bulk-carrier crane failure from corrosion hidden under grease; operator now replaces ropes every 5 years | snippet | Page returned 403 |
| U7 | Functional-test intervals for rail clamps and storm pins | none found | No source |
| U8 | Fresh-water wash interval for port cranes | none found | No source |
| U9 | IR rule of thumb: halves per +10 °C; test only above the dew point | from memory | Not on the opened page |
| U10 | Desiccant breathers; oil water-saturation of roughly 200–600 ppm | from memory | No source opened |
| U11 | Moisture/contamination share of motor failures in the IEEE 1985 Part I contributor table | from memory, vague | The tables are images on ReliaMag |
| U12 | Patent US7272921: galvanized outer wires +34 % bending cycles | snippet | Not opened |
| U13 | Location of the Maemi 60 m/s gust (believed to be Jeju Gosan) | from memory | E-Today did not name the station |

---

## Derived modelling hints (my synthesis from the sourced numbers above, not from a source)

**Coating.** Draw time to first major maintenance from the ISO 12944-1 band for the chosen class:
- C5-M baseline (old spec, about 250–260 µm): H = 15–25 years. If the spec is unknown, use M = 7–15 years.
- Preset "염해 방지": C5-VH (320 µm, ≥3 coats, zinc-rich primer) gives >25 years. CX siting cuts the system choice or the life.
- Cycle shape from 1g: touch-up ≈ 0.55 T, maintenance repaint ≈ 0.7 T, full repaint = T.

**HDG parts** (walkways, handrails): life = thickness / rate.
- Use 4.2–8.4 µm/a for C5 and 8.4–25 µm/a for CX.
- Cross-check against GAA: 85 µm lasts 10–20 years in C5 and 3–10 years in CX.
- Paint over HDG: life = 1.4–1.6 × (HDG life + paint life) in extreme marine conditions.

**Gearbox and bearing life.** Multiply life by (100/ppm)^0.6, with ppm clamped to 25–400. The preset (breathers, seals) could lower the ppm state variable.

**Electronics.** Use ISA class G1→GX as a hazard multiplier. Raise the class by one level per +10 % RH above 50 %. An air-conditioned e-room (Koper: 2×100 % units) holds RH below 50 %, which keeps the class at its base level.

**Motors.**
- Treat IR below 5 MΩ (LV) or PI below 2.0 as a "fail inspection" state.
- Heaters hold the winding 5–10 K above ambient, which prevents condensation at standstill.

**Ropes.**
- Relubrication: life multiplier 2–3× (vendor). Seawater fretting reduces fatigue life, but no magnitude was found.
- Galvanized rope helps in seawater but is slightly worse in air.

**Storm.**
- Koper wind regimes: operate ≤22 m/s; limited 22–35 m/s; parked 35–44 m/s; stowed above 44 m/s.
- Storm pins rated above 50 m/s.
- Tie-down tension is strongly nonlinear above design wind (the Maemi mechanism). Busan saw 42.7 m/s.
