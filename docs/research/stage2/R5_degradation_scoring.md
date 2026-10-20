# R5 — Degradation time-series, prognostics & forecast scoring (sourced)

Status: COMPLETE for this pass (15 web searches used; many direct fetches). Date: 2026-10-20.
Rule: every number carries a URL actually opened + verbatim quote (<=25 words) or equation ref.
Confidence tags: [high] primary source opened & quoted; [med] secondary source or partial text; [low] indirect.
Anything remembered, or seen only in search-result snippets (page not opened), is listed under "Unverified".
Note on PDF extraction: several PDFs were text-extracted with pdftotext; Greek letters were sometimes dropped by
the extractor, so equations below are re-typeset from the extracted structure + equation numbers (flagged where so).
"My arithmetic" = numbers I derived from sourced parameters (not printed in the source).

## At a glance (what to put in the Rust sim)
| Item | Value / formula | Source (section) | Conf. |
|---|---|---|---|
| Gamma process | X(tau)-X(t) ~ Ga(v(tau)-v(t), u), v(t)=c·t^b, E=v/u, Var=v/u^2, CoV=1/sqrt(v) | Strauss&Demissie citing van Noortwijk (1.2) | high |
| Exponent b | 1 linear (rebar corrosion), 2 parabolic (sulphate), 0.5 sqrt (diffusion) | same (1.2) | high |
| MoM estimator | c/u = x_n/t_n^b ; (x_n/u)(1-SUM w_i^2/(SUM w_i)^2) = SUM(delta_i-(c/u)w_i)^2 | same, eq. 15–16 (1.2) | med (typeset) |
| Wiener RUL | first passage ~ IG(a/nu, (a/sigma)^2) | Wikipedia IG (1.4) | med |
| Rope discard | ISO 4309 broken wires, e.g. RCN 1 ordinary lay 2 per 6d / 4 per 30d; diameter -10 % FC, -7.5 % IWRC, -5 % rot.-resistant; severity 20 % slight ... 100 % discard | Certex, Cranes Today (2.1–2.2) | med |
| Bearing Weibull slope | 1.11 ball / 1.125 roller (L-P); tests 1–2 | NASA TP-2013-215305 (3.4) | high |
| Gear / gearbox slope | gear e=2.5 assumed; gearbox field e=2.189, L10=5627 h | same (3.4) | high |
| Weibull table | bearings beta 1.3, gears 2, AC motors 1.2, couplings 2, solid-state controllers 0.7 ... | Barringer/Bloch&Geitner (4.1) | med |
| PRONOSTIA | 3 conditions, 25.6 kHz, 0.1 s/10 s, fail at 20 g, lives 1–7 h | PHM 2012 challenge doc (3.2) | high |
| PHM-2012 score | A=exp(-ln0.5·Er/5) Er<=0; exp(+ln0.5·Er/20) Er>0 | same (3.2) | high |
| Insulation | PI=R10/R1; min PI 1.5 (A), 2.0 (B, F); Rm=kV+1 MOhm @40 °C | USBR FIST 3-1 (4.3) | high |
| Wheel wear analogue | fail at 60 mm wear; ~420–560 kkm (≈0.11–0.14 mm/1000 km, my arithmetic) | arXiv 2212.02688 (4.2) | high/analogue |
| Saxena metrics | PH, alpha-lambda (alpha=0.1, lambda=0.5 example), RA, CRA, C_M | Saxena 2010 (Q5) | high |
| Scores | pinball (factor-2), CRPS kernel/quantile forms, WIS (w0=1/2, wk=alpha_k/2), interval score (2/alpha penalty), PIT, Brier+Murphy | G&R 2007, Bracher 2021, FPP3, GBR 2007 (Q6) | high |
| Calibration | split conformal q=ceil((n+1)(1-alpha))/n, n≈1000; ACI alpha_{t+1}=alpha_t+gamma(alpha-err_t), gamma=0.005 | A&B 2021, G&C 2021 (Q7) | high |
| HI bands | ISO 4309 severity %; transformer THI 85/70/(50/30) bands; mean+2sigma alarms | (Q8) | med/low |

## Q1 Gamma process / Wiener process

### 1.1 van Noortwijk (2009) survey — what could be verified
Primary: J.M. van Noortwijk, "A survey of the application of gamma processes in maintenance", RESS 94(1):2–21, 2009,
DOI 10.1016/j.ress.2007.03.019 — full text paywalled (no OA copy found). Abstract opened at
https://ideas.repec.org/a/eee/reensy/v94y2009i1p2-21.html : "Because gamma processes are well suited for modelling the temporal
variability of deterioration" ... "an extensive catalogue of inspection and maintenance models under gamma-process deterioration". [high, abstract only]

### 1.2 Formulation (as restated, with eq. numbers, by Strauss & Demissie, REC 2012, citing van Noortwijk 2009) [high for the restatement]
Source opened: https://rec2012.fce.vutbr.cz/documents/papers/strauss.pdf ("Gamma Process – Life cycle analysis of the Neumarkt Bridge")
- Gamma density (eq. 1): Ga(x | v, u) = u^v / Gamma(v) · x^(v-1) · exp(-u x), x > 0 (v = shape; u is called the "scale parameter"
  in this notation but enters as a RATE, so mean = v/u — beware when mapping to Rust libraries that use scale = 1/u).
- Process (eq. 3–5): X(0) = 0 w.p. 1; increments X(tau) - X(t) ~ Ga(v(tau) - v(t), u) for tau > t, independent.
  "the essential positive increments are determined by a gamma distribution with identical scale parameter and a time dependent shape function".
- Moments (eq. 6–8): E[X(t)] = v(t)/u; Var[X(t)] = v(t)/u^2; CoV[X(t)] = 1/sqrt(v(t)).
- Power law (eq. 9): v(t) = c·t^b — "a linear shape (b = 1) for the corrosion of concrete reinforcement, a parabolic (b = 2) for the
  sulphate attack, and a square root (b = 0.5) for the diffusion-controlled aging according to Elingwood and Mori."
  "in most cases the shape of the expected deterioration b is known and can be taken as a constant, as pointed out by VanNoortwijk".
- Estimation: "The three most suited statistical methods are the Maximum Likelihood Method, the Method of Moments and Bayesian Statistics (VanNoortwijk, 2009)."
- Method of moments (eq. 13–16, "recommended by Van Noortwijk"), inspection times t_1..t_n, transformed z_i = t_i^b,
  w_i = t_i^b - t_{i-1}^b, delta_i = x_i - x_{i-1}:
    c_hat/u_hat = SUM delta_i / SUM w_i = x_n / t_n^b                                   (15)
    (x_n/u_hat)·(1 - SUM w_i^2 / (SUM w_i)^2) = SUM (delta_i - (c_hat/u_hat)·w_i)^2       (16)
  [equations re-typeset from garbled extraction; structure matches the eq. numbers — med for exact typesetting]
  Caveat quoted: "the Method of Moments provides unreliable results in the early lifetime of a structure" (use expert priors early).
- MLE (standard, not quoted): log L(c,u) = SUM_i [ c·w_i·ln u - ln Gamma(c·w_i) + (c·w_i - 1)·ln delta_i - u·delta_i ];
  dlogL/du = 0 gives u_hat = c_hat·t_n^b / x_n, then 1-D root-find for c. [standard result; med]

### 1.3 Choosing the CoV (simulation calibration rule)
Because CoV[X(t)] = 1/sqrt(c·t^b) (eq. 8 above), if a source gives the mean degradation m(t_ref) and its CoV at a reference time
t_ref, then c = 1/(CoV^2 · t_ref^b) and u = c·t_ref^b / m(t_ref). The CoV decays with time (noise averages out) — this is the
key behavioural difference from a random-rate (random-variable) model whose CoV stays constant.
No primary source opened gives "typical CoV for wear/corrosion" — see Unverified.
Worked, sourced example (arXiv 2212.02688 §4, laser data; ar5iv HTML): inspections "every 250 hours, and the termination time of the
experiment is 4000 hours"; threshold "10% increase in the operating current"; increment density beta^{alpha(t-s)} y^{alpha(t-s)-1} e^{-beta y}/Gamma(.)
(beta is a RATE); posterior means alpha ≈ 0.0309 h^-1, beta ≈ 15.34 %^-1 (Gibbs, Table 1). Derived (my arithmetic): mean rate
alpha/beta ≈ 0.0020 %/h (≈ 2 % per 1000 h; first-order hitting time of 10 %: D·beta/alpha ≈ 4,960 h); CoV[X(t)] = 1/sqrt(alpha·t) ≈ 0.18 at 1000 h and
≈ 0.09 at 4000 h — i.e. a homogeneous gamma process alone gives a small CoV at end of life; unit-to-unit spread must come from a
random rate/scale (the paper's heterogeneous model GP(alpha·t, beta_i): "different scales β_i s represent heterogeneity existed among systems").

### 1.4 Gamma vs Wiener — when each is used
- Gamma = monotone: "Gamma process has been extensively used to model monotone degradation data." (arXiv 2212.02688 abstract,
  https://arxiv.org/abs/2212.02688) and "is often adopted when the PC deterioration is strictly monotone" (intro, ar5iv HTML). [high]
  Same paper's examples: laser current, 15 GaAs devices (threshold "10% increase in the operating current"; GP(alpha·t, beta) posterior
  means alpha ≈ 0.031, beta ≈ 15.3 in the paper's own time/percent units, Table 1) and train-wheel wear (threshold 60 mm, see Q4).
- Wiener (Brownian motion with drift): X_t = nu·t + sigma·W_t; first passage of fixed level a>0 is inverse Gaussian:
  T_a ~ IG(a/nu, (a/sigma)^2), f(t) = a / sqrt(2·pi·sigma^2·t^3) · exp(-(a - nu·t)^2 / (2·sigma^2·t)).
  Source: https://en.wikipedia.org/wiki/Inverse_Gaussian_distribution ("the first passage time for a fixed level α > 0 by X_t is
  distributed according to an inverse-Gaussian"). [med — secondary but standard]
- Practice (synthesis, [med]): use gamma (or IG process) for physically monotone damage (wear depth, corrosion loss, crack length,
  broken-wire count, lining thickness loss); use Wiener/diffusion for health indicators that can go down as well as up
  (vibration RMS, insulation resistance, temperature rise) or when measurement noise dominates; Wiener allows Kalman-type online updates
  (arXiv 2212.02688: Kalman-based methods are "founded on the Gaussian distribution").
- Simulation design implied: latent monotone gamma state + Gaussian measurement noise (noisy-gamma; cf. Le Son, Fouladirad & Barros 2016 RESS,
  cited in search results, not opened).

Unverified (Q1): any "typical" CoV value for wear/corrosion gamma processes (none verified — calibrate per 1.3); the survey's own examples
(e.g., dyke/berm erosion, steel coating breakdown percentages, thinning of pressure vessels) and their fitted c, b, u values;
ISO 9224 atmospheric-corrosion time exponent b ≈ 0.523 for carbon steel (useful as v(t) exponent for girder corrosion) — not opened.

## Q2 Wire rope deterioration

### 2.1 ISO 4309 broken-wire discard numbers [med — third-party reproduction of ISO 4309:2010 Table; column order inferred]
Source opened: CERTEX Danmark, "Discard of steel wire rope",
https://www.certex.dk/en/knowhow/articles-about-steel-wire-rope/discard-of-steel-wire-rope (tables labelled ISO 4309-2010).
The page text lost column headers; values per row are printed in this order. Column assignment below = ISO 4309 layout
[sheave/single-layer drum: ordinary lay 6d, 30d | Lang lay 6d, 30d | multi-layer drum: 6d, 30d]; supporting evidence:
(i) internal consistency of the n>300 row on the opened page (0.04n, 0.08n, 0.02n, 0.04n, 0.08n, 0.16n — Lang = half of ordinary,
multi-layer = double), (ii) search-result snippets only (pages not fully opened): a summary of the same Certex page ("RCN 1 ... 2 visible
breaks over 6d and 4 over 30d for ordinary lay, and 1 and 2 for Lang lay") and an OIPEEC abstract snippet ("fixed ratio of approximately
2:1 ... between ordinary lay and Lang lay"). d = nominal rope diameter; n = load-bearing wires in outer strands.

Single-layer & parallel-closed ropes (excerpt):
| RCN | n | Ord 6d | Ord 30d | Lang 6d | Lang 30d | Multi-layer 6d | Multi-layer 30d |
|---|---|---|---|---|---|---|---|
| 1 | <=50 | 2 | 4 | 1 | 2 | 4 | 8 |
| 2 | 51–75 | 3 | 6 | 2 | 3 | 6 | 12 |
| 3 | 76–100 | 4 | 8 | 2 | 4 | 8 | 16 |
| 4 | 101–120 | 5 | 10 | 2 | 5 | 10 | 20 |
| 5 | 121–140 | 6 | 11 | 3 | 6 | 12 | 22 |
| 6 | 141–160 | 6 | 13 | 3 | 6 | 12 | 26 |
| 7 | 161–180 | 7 | 14 | 4 | 7 | 14 | 28 |
| 8 | 181–200 | 8 | 16 | 4 | 8 | 16 | 32 |
| 9 | 201–220 | 9 | 18 | 4 | 9 | 18 | 36 |
| 10 | 221–240 | 10 | 19 | 5 | 10 | 20 | 38 |
| 11 | 241–260 | 10 | 21 | 5 | 10 | 20 | 42 |
| 12 | 261–280 | 11 | 22 | 6 | 11 | 22 | 44 |
| 13 | 281–300 | 12 | 24 | 6 | 12 | 24 | 48 |
| – | >300 | 0.04n | 0.08n | 0.02n | 0.04n | 0.08n | 0.16n |
Page note: for Seale constructions with <=19 wires per outer strand the row is placed "two rows above" its normal position.
Rotation-resistant ropes (sheave/single-layer 6d,30d | multi-layer 6d,30d): RCN 21: 2,4 | 2,4; RCN 23-1 (71–100): 2,4 | 4,8;
RCN 24 (141–160): 3,6 | 6,13; RCN 26 (181–200): 4,8 | 8,16; RCN 31 (281–300) and n>300: 6,12 | 12,24.
Unverified: whether these apply to classes M1–M4 only with relaxed values for M5–M8 (a Scribd presentation in search results suggested so).
Simulation use: count visible breaks in the worst 6d and 30d windows of the simulated rope; severity_breaks = 100·max(B6/B6_lim, B30/B30_lim).

### 2.2 Diameter decrease and severity rating (ISO 4309:2010) [med — trade-press summary]
Source opened: Cranes Today, 20 Oct 2011, https://www.cranestodaymagazine.com/?p=4775
- "single layer ropes with fibre cores are allowed to decrease up to 10%"; "single layer ropes with steel cores or parallel closed ropes
  are allowed to decrease up to 7.5%"; "rotational resistant ropes should only be allowed to decrease up to 5%" (from reference diameter).
- Reference diameter: "The reference diameter is always larger than the catalog value due to the manufacturing tolerances" (measured after run-in).
- Severity: "a Severity Rating of 20% is 'slight', 80% 'very high' and 100% means that it is time to 'discard'."
- Combined effect: ISO 4309:2010 formalises "a method of assessing rope deterioration known as the cumulative effect".
Carl Stahl calculator page (https://www.carlstahl.com/gb/en/services/wire-rope-diametric-reduction-calculator):
"The Severity Rating represents the amount of deterioration expressed as a percentage towards discard";
"The Reference Diameter is the measured diameter of a section of rope that is not subject to bending".
=> Natural rope health score: H_rope = 100 - min(100, SUM_modes severity_%) (cumulative-effect rule); severity per mode =
   observed/limit x 100 in 20 % steps (20 slight ... 80 very high, 100 discard). Intermediate labels 40 % "medium", 60 % "high": Unverified.
ISO 4309 current edition: ISO 4309:2017 (iso.org listing seen in search results; SFS notes a 2021 national edition) — 2010 values above may differ slightly.

### 2.3 Broken-wire growth vs cycles, and fatigue-life scatter
- Feyrer (Wire Ropes: Tension, Endurance, Reliability, Springer 2007/2015) is the primary source; not accessible. OIPEEC abstract
  (https://www.oipeec.org/?p=15755, Feyrer): his formulae evaluate "the probable number of bending cycles, the maximum rope tension allowed,
  the discarding number of wire breaks and the optimal rope diameter" [high for that sentence only; no numbers available].
Unverified (Q2.3) — seen only in search-result snippets (pages not opened): Feyrer's regression of "678 bending tests with 55 wire ropes"
giving mean bending cycles N_m and N_10 (cycles at which, with 95 % confidence, no more than 10 % of ropes have failed); an unnamed
experimental study where broken wires "develop slowly in the early stage" and "rapidly when the fatigue times exceed 60,000" cycles;
a patent defining rope life as broken wires reaching 10 % of total wires in a repeated-bending test at 20 % of breaking load (USPTO 5545482).
Unverified (Q2.3) — needed from Feyrer's book: the wire-break growth law vs N/N_break (described in the literature as accelerating toward
end of life), discard number of wire breaks B_A30 vs N_A, the scatter of lg N and any Weibull shape for rope life (no value verified).
Simulation fallback (explicitly a modelling choice, not sourced): broken-wire count as a non-homogeneous Poisson/gamma process with
rate ∝ exp(k·N/N_life), N_life ~ lognormal with sigma(lg N) to be calibrated against the project's field inspection records.

## Q3 Rolling-bearing degradation

### 3.1 Exponential degradation-signal model (Gebraeel et al.) [med — primary IIE Trans. 2005 paywalled; model form via MathWorks doc]
Primary refs: Gebraeel, Lawley, Li, Ryan (2005) "Residual-life distributions from component degradation signals: A Bayesian
approach", IIE Transactions 37:543-557; Gebraeel (2006) IEEE T-ASE 3(4):382-393 (sensory-updated RLD). Neither opened.
Model form opened at https://www.mathworks.com/help/predmaint/ref/exponentialdegradationmodel.html (cites Gebraeel 2006):
  S(t) = phi + theta(t)·exp( beta(t)·t + eps(t) - sigma^2/2 )
  "θ(t) is a random variable modeled as a lognormal distribution"; "β(t) is a random variable modeled as a Gaussian distribution";
  "ε(t) is the model additive noise and is modeled as a normal distribution with zero mean"; rho = corr(theta, beta) in [-1,1].
  Defaults in toolbox: Rho = 0, NoiseVariance = 1, SlopeDetectionLevel = 0.05 (toolbox defaults, not physics).
Log form used for conjugate Bayesian updating: L(t) = ln(S(t) - phi) = ln(theta) + beta·t + eps(t) - sigma^2/2 — linear in t,
so (ln theta, beta) ~ bivariate normal prior -> closed-form normal posterior after each observation; RUL = first passage of
L(t) over ln(D - phi) (D = failure threshold). [model algebra: standard; med]
Unverified (Q3.1): search-result snippet (page not opened) attributing to Chen & Tsui 2013 (IIE Trans.) that Gebraeel et al. used
"an exponential model with random coefficients" after a change point (two-phase: flat, then exponential); Gebraeel's bearing test-rig
numbers (thrust ball bearings, failure threshold ~0.025 Vrms, prior means/variances) — remembered, not verified.

### 3.2 PRONOSTIA / FEMTO (IEEE PHM 2012 challenge) [high]
Source opened: IEEE PHM 2012 Prognostic Challenge details (FEMTO-ST), copy at
https://raw.githubusercontent.com/wkzs111/phm-ieee-2012-data-challenge-dataset/master/IEEEPHM2012-Challenge-Details.pdf
(platform paper: Nectoux et al., IEEE PHM 2012 — HAL copy blocked by bot wall, not opened).
- Conditions: "1800 rpm and 4000 N"; "1650 rpm and 4200 N"; "1500 rpm and 5000 N". 6 learning run-to-failure bearings, 11 test bearings.
- Failure definition: "RUL was defined as time to accelerometer exceeding 20g."
- Acquisition: vibration "Sampling frequency: 25.6 kHz"; 2560 samples (1/10 s) "recorded each 10 seconds"; temperature 10 Hz, 600 samples/min.
- Life scatter: "high variability in experiment durations (from 1h to 7h)"; learning-set recording durations 7h47m00s, 2h25m00s,
  2h31m40s, 2h12m40s, 1h25m40s, 4h32m40s (Appendix A.4; order Bearing1_1, 1_2, 2_1, 2_2, 3_1, 3_2).
- Degradation is not seeded: "the defects are not initially initiated on the bearings" and degraded bearings contain "almost all the
  types of defects (balls, rings and cage)"; "Existing reliability laws for bearings' life duration, such as the L10, do not give same results".
- Challenge RUL score (eq. 1–3): %Er_i = 100·(ActRUL_i - RUL_i)/ActRUL_i;
  A_i = exp(-ln(0.5)·(Er_i/5)) if Er_i <= 0 (late/over-estimate, harsher); A_i = exp(+ln(0.5)·(Er_i/20)) if Er_i > 0 (early);
  Score = (1/11)·SUM A_i. (So A=0.5 at Er=+20 % and at Er=-5 %.) — a ready-made asymmetric RUL score for the crane forecaster.

### 3.3 NASA IMS bearing data [low/med]
Opened https://www.nasa.gov/intelligent-systems-division/discovery-and-systems-health/pcoe/pcoe-data-set-repository/ :
"Experiments on bearings. The data set was provided by the Center for Intelligent Maintenance Systems (IMS), University of Cincinnati";
citation "J. Lee, H. Qiu, G. Yu, J. Lin, and Rexnord Technical Services (2007)". Test-rig details are only in the zip's readme (not opened).
Unverified (IMS): 4 Rexnord ZA-2115 double-row bearings, 2000 rpm, 6000 lb radial load, 20 kHz sampling, 20,480-point snapshots
every 10 min, 3 test sets; failures: inner-race (bearing 3) and roller (bearing 4) in set 1, outer race in sets 2/3 (Qiu et al. 2006 JSV 289:1066).

### 3.4 Bearing life scatter (Weibull slope) [high]
Source opened: Zaretsky, "Rolling Bearing Life Prediction, Theory, and Application", NASA/TP-2013-215305,
https://ntrs.nasa.gov/api/citations/20130011515/downloads/20130011515.pdf
- "Lundberg and Palmgren (Ref. 9) assumed the value of the Weibull slope e ... to be 1.11 for ball bearings ... and 1.125 for roller bearings".
- Test data: "rolling-element fatigue data reported by Parker and Zaretsky (Ref. 47) reflect slopes in the range of 1 to 2".
- Sample-size effect on estimated slope (Johnson; 90 % of cases), r = number failed: e_max = 1.11 + 1.31·r^-0.5 (88a); e_min = 1.11 - 1.31·r^-0.5 (88b).
- "The scatter in the data is inversely proportional to the Weibull slope".
- Gears: "eG is the Weibull slope for the gear and is assumed to be 2.5"; gear load-life exponent "usually taken as 4.3".
- Gearbox field data: 64 turboprop gearboxes, 59 failed; "The resultant L10 life from the field data was 5627 hr and the Weibull slope e was 2.189"
  (L-P prediction: 774 hr, e = 1.125) — L-P/ISO under-predicts gearbox life.
- System rule: "the system Weibull slope approximates that of the lowest lived component in the system."
Unverified (Q3.4): ISO 281:2007 a1 reliability factor a1 = 0.95·[ln(100/R)/ln(100/90)]^(2/3) + 0.05 (implies Weibull slope 1.5);
a1(95 %) = 0.64, a1(99 %) = 0.25. Not opened (ISO paywalled; SKF page not fetched).
Onset of detectable degradation as % of life: no primary source opened; one open-access article (PMC7795917, "Data-Driven Method for
Predicting Remaining Useful Life of Bearing Based on Bayesian Theory", https://pmc.ncbi.nlm.nih.gov/articles/PMC7795917/, XJTU-SY data)
reports HI "increases suddenly when the time series reaches 60" for a 158-min bearing (≈38 % of life, my arithmetic) — single case, [low].
Same article: exponential HI model H(t) = exp[a·(t - T) + b] + c with measurement Y_t = H_t + eps, eps ~ N(0, sigma^2) (eqs. 11–12, 6–7).

## Q4 Brakes, wheels, insulation, Weibull shapes

### 4.1 Weibull shape/characteristic-life table (Barringer & Associates, from Bloch & Geitner 1994 App. A) [med]
Source opened (2016 Wayback snapshot; live site dead): https://web.archive.org/web/2016id_/http://www.barringer1.com/wdbase.htm
Caveat quoted: "this database is solely intended for educational purposes"; "Data, described in this database, are very general."
Origin: "Bloch and Geitner's Appendix A is given in failures per million hours ... converted into ages-to-failure ... along with
estimates of typical Weibull values" (Barringer's engineering judgement).

| Component | beta low | beta typical | beta high | eta low (h) | eta typical (h) | eta high (h) |
|---|---|---|---|---|---|---|
| Ball bearing | 0.7 | 1.3 | 3.5 | 14,000 | 40,000 | 250,000 |
| Roller bearings | 0.7 | 1.3 | 3.5 | 9,000 | 50,000 | 125,000 |
| Sleeve bearing | 0.7 | 1 | 3 | 10,000 | 50,000 | 143,000 |
| Belts, drive | 0.5 | 1.2 | 2.8 | 9,000 | 30,000 | 91,000 |
| Bolts | 0.5 | 3 | 10 | 125,000 | 300,000 | 100,000,000 |
| Clutches, friction | 0.5 | 1.4 | 3 | 67,000 | 100,000 | 500,000 |
| Clutches, magnetic | 0.8 | 1 | 1.6 | 100,000 | 150,000 | 333,000 |
| Couplings | 0.8 | 2 | 6 | 25,000 | 75,000 | 333,000 |
| Couplings, gear | 0.8 | 2.5 | 4 | 25,000 | 75,000 | 1,250,000 |
| Cylinders, hydraulic | 1 | 2 | 3.8 | 9,000,000 | 900,000 | 200,000,000 |
| Gears | 0.5 | 2 | 6 | 33,000 | 75,000 | 500,000 |
| Pins | 0.5 | 1.4 | 5 | 17,000 | 50,000 | 170,000 |
| Seals, mechanical | 0.8 | 1.4 | 4 | 3,000 | 25,000 | 50,000 |
| Springs | 0.5 | 1.1 | 3 | 14,000 | 25,000 | 5,000,000 |
| Circuit breakers | 0.5 | 1.5 | 3 | 67,000 | 100,000 | 1,400,000 |
| Motors, AC | 0.5 | 1.2 | 3 | 1,000 | 100,000 | 200,000 |
| Motors, DC | 0.5 | 1.2 | 3 | 100 | 50,000 | 100,000 |
| Transformers | 0.5 | 1.1 | 3 | 14,000 | 200,000 | 14,200,000 |
| Controllers, solid state | 0.5 | 0.7 | 1.1 | 20,000 | 100,000 | 200,000 |
| Solenoid valves | 0.5 | 1.1 | 3 | 50,000 | 75,000 | 1,000,000 |
| Electro-mechanical parts | 0.5 | 1 | 3 | 13,000 | 25,000 | 1,000,000 |
| Transducers | 0.5 | 1 | 3 | 11,000 | 20,000 | 90,000 |
| Greases | 0.5 | 1.1 | 3 | 7,000 | 10,000 | 33,000 |
| Lube oils, mineral | 0.5 | 1.1 | 3 | 3,000 | 10,000 | 25,000 |
(Table copied as printed — note some rows have eta "low" > "typical", e.g. hydraulic cylinders; treat with care.)
No brake-lining or wire-rope row exists in this table; "Clutches, friction" (beta 1.4, eta 100,000 h) is the closest analogue
for a friction brake *as a unit*, not for lining wear (lining should be modelled as wear-out degradation, see 4.2).
Mapping for crane sim: hoist/travel gearbox -> gears beta≈2 (or 2.189 field, NASA 3.4); motors -> 1.2; bearings -> 1.3;
drive electronics/controllers -> 0.7 (infant-mortality-like); couplings -> 2.

### 4.2 Wheel wear (rail-wheel analogue for crane travel wheels) [high for the dataset facts; analogue only]
Source opened: arXiv 2212.02688 §7.2 (ar5iv HTML https://ar5iv.labs.arxiv.org/html/2212.02688), data from Almeida (2011, Table 5.1):
- "When the wear of wheel diameter attains 60 (mm), the wheel is considered to have failed."
- 14 train wheels (11 used); wear measured at 50, 100, ..., 600 thousand km ("equally spaced from 50 to 600 in increments of 50").
- Interpolated failure distances for three wheels: 523.537, 558.861 and 421.508 thousand km -> mean wear rate ≈ 0.11–0.14 mm per 1000 km
  (my arithmetic: 60 mm / 421.5–558.9 kkm). Modelled as a gamma process with a unit-specific scale beta_i (unit-to-unit heterogeneity).
- For a crane: replace km with wheel revolutions or travelled metres x load factor; keep gamma process + heterogeneous scale.
No crane-specific wheel-wear rate source was found/opened — see Unverified.

### 4.3 Motor insulation resistance & PI [high]
Source opened: US Bureau of Reclamation FIST Vol. 3-1 "Testing Solid Insulation of Electrical Equipment" (Dec 1991; internet version 2000),
https://www.usbr.gov/power/data/fist/fist3_1/vol3-1.pdf
- PI definition: "Polarization index = R10/R1 = I1/I10 (if the voltage is constant)".
- Minimum PI (citing IEEE 43): "For class A insulation - 1.5 For class B insulation - 2.0 For class F insulation - 2.0".
- Minimum IR (IEEE 43-1974 as quoted): Rm = Vt + 1 [MOhm at 40 °C, Vt in kV rms]; 1-min IR "must be corrected to 40 °C".
- Temperature: "Insulation resistance varies inversely with temperature for most insulting [sic] materials"; trend only at a common base
  temperature ("take each measurement at the same temperature, or to convert each measurement to the same base temperature").
- Moisture: moisture "will make a surprisingly large difference in the insulation resistance"; Table I (Grand Coulee generator):
  PI moist 1.88 @36 °C, 2.50 @75 °C; dry 3.11 @75 °C, 2.16 @36 °C — "the hot test is more sensitive".
- Ageing: asphalt-mica curing -> "decrease in insulation resistance with increasing age".
- Record "ambient temperature and humidity" with each test (procedure 13c).
Simulation hook: IR(t) = IR_ref·exp(-k_T·(T - 40))·m(RH)·a(t) with ageing a(t) decreasing; observed PI noisy; seasonal humidity enters
via m(RH) and makes IR a *non-monotone* indicator (Wiener-type), while the latent insulation age is monotone (gamma). k_T, m(RH): not sourced.
Unverified (Q4.3): IEEE 43-2013 temperature correction Kt = 0.5^((40 - T)/10) ("halves every 10 °C"); IEEE 43-2013 minimum IR 100 MOhm
for form-wound post-1970 windings / 5 MOhm random-wound <1 kV; PI not required when IR1min > 5000 MOhm.

### 4.4 Brake lining wear variability
No open primary data source found within the search budget. Model as monotone gamma wear (thickness loss per stop proportional to
dissipated energy) with discard at manufacturer's minimum lining thickness — parameters must come from brake maker data or the
project's own calibration. Unverified: any wear-rate CoV for crane drum/disc brake linings.

## Q5 Prognostic metrics (Saxena et al. 2010, IJPHM 1(1))

Source opened: Saxena, Celaya, Saha, Saha, Goebel, "Metrics for Offline Evaluation of Prognostic Performance",
Int. J. PHM 2010 — https://ntrs.nasa.gov/api/citations/20100039169/downloads/20100039169.pdf
(also listed at https://papers.phmsociety.org/index.php/ijphm/article/view/1336). [high for definitions; Greek symbols re-typeset]

Notation (paper's "Definitions"): t_P = time of first prediction; t_EoL = end of life (ground truth EoL*); t_EoP = end of
prediction; t_EoUP = end of useful predictions ("time index beyond which it is futile to update a RUL prediction because
no corrective action is possible"); r*(i) = ground-truth RUL at t_i; r(i) = predicted RUL (distribution) at t_i;
p = set of time indexes when predictions are made; pi[r(i)] = probability mass of the predicted RUL PDF inside the bounds;
beta = "minimum acceptable probability mass".

Hierarchy (quote): "These metrics can be considered as a hierarchical test" — PH -> alpha-lambda -> RA/CRA -> Convergence.

| Metric | Formula (paper eq.) | Notes / quote |
|---|---|---|
| Prognostic Horizon | PH = t_EoL - t_i, i = min{ j : j in p and pi[r(j)]_{-alpha}^{+alpha} >= beta }  (eq. 4) | bounds are r* - alpha·t_EoL and r* + alpha·t_EoL (alpha is a fraction of EoL, fixed-width band). "an algorithm with longer prediction horizon would be preferred" |
| alpha-lambda accuracy | = 1 if pi[r(i_lambda)]_{-alpha}^{+alpha} >= beta, else 0 (eq. 5); t_lambda = t_P + lambda·(t_EoL - t_P) | bounds (1-alpha)·r*(i_lambda) and (1+alpha)·r*(i_lambda) — a cone that shrinks with RUL. Example in paper: "within 10% accuracy (alpha = 0.1) of the true RUL halfway to failure ... (lambda = 0.5)" |
| Relative Accuracy | RA_lambda = 1 - abs(r*(i_lambda) - r(i_lambda)) / r*(i_lambda)  (eq. 6) | r(i) = central-tendency point estimate (mean/median per their Table 3). "The range of values for RA is [0,1], where the perfect score is 1" (negative if error >100%) |
| Cumulative RA | CRA_lambda = (1/abs(p_lambda)) · SUM_{i in p_lambda} w(r(i)) · RA_lambda(i)  (eq. 7) | "normalized weighted sum of relative accuracies"; weights: "desirable to weigh those relative accuracies higher that are closer to tEoL" |
| Convergence | C_M = sqrt((x_c - t_P)^2 + y_c^2) (eq. 8); x_c = [ (1/2)·SUM (t_{i+1}^2 - t_i^2)·M(i) ] / [ SUM (t_{i+1}-t_i)·M(i) ], y_c = [ (1/2)·SUM (t_{i+1}-t_i)·M(i)^2 ] / [ SUM (t_{i+1}-t_i)·M(i) ] (eq. 9), sums over i from P to EoUP | M(i) = non-negative error/precision metric (e.g. abs RUL error). "Lower distance means a faster convergence." |

Extraction caveat: Greek letters were dropped by pdftotext; the bound expressions above are re-typeset from the surviving structure
("r* ... t EoL" for PH; "r*(i) ... r(i)" for alpha-lambda). For alpha-lambda the surviving text reads like r*(i) ± alpha·r(i); the
ground-truth form (1 ± alpha)·r*(i_lambda) is used here because the paper describes "bounds that shrink as time passes" — [med] on that detail.
Recommended values: the paper gives no universal alpha/beta/lambda; its worked example is alpha = 0.1, lambda = 0.5 [high].
beta is user-chosen ("minimum acceptable probability mass"); a common choice in later NASA papers is beta = 0.5 — see Unverified.
Paper's RUL-vs-time example: "p in [70,240], tEoL = 240" (Figure 7 caption) [high].

Simulation hook: for each simulated unit with known t_EoL, store the forecaster's RUL quantiles at each prediction time,
compute PH (alpha≈0.1 of EoL), alpha-lambda at lambda ∈ {0.25,0.5,0.75}, RA/CRA, and C_M on abs-error.

Unverified (Q5): beta = 0.5 as "typical"; alpha = 0.2 used in "On Applying the Prognostic Performance Metrics" (Saxena et al. 2009 PHM conf.) — not opened.

## Q6 Probabilistic forecast scoring

### 6.1 Quantile (pinball) loss [high]
- FPP3 §5.9 (Hyndman & Athanasopoulos) https://otexts.com/fpp3/distaccuracy.html — "sometimes called the 'pinball loss function'":
  Q_{p,t} = 2(1-p)(f_{p,t} - y_t) if y_t < f_{p,t};  2p(y_t - f_{p,t}) if y_t >= f_{p,t}.
  "A low value of Q_{p,t} indicates a better estimate of the quantile." (note the factor 2 convention)
- Gneiting & Raftery 2007 (JASA 102:359-378) eq. (41), positively oriented: S(r; x) = (x - r)(1{x <= r} - alpha);
  "the econometric literature refers to the scoring rule (41) as the tick or check loss function."
  PDF: https://sites.stat.washington.edu/raftery/Research/PDF/Gneiting2007jasa.pdf [high]
- Bracher et al. 2021 (PLoS Comput Biol; arXiv 2005.12881) App. A: QS_tau(F,y) = 2·{1(y <= q_tau) - tau}·(q_tau - y).
  https://ar5iv.labs.arxiv.org/html/2005.12881 [high]

### 6.2 CRPS [high]
- Gneiting & Raftery 2007 eq. (20) (positively oriented): CRPS(F,x) = - INT_{-inf}^{inf} (F(y) - 1{y >= x})^2 dy;
  "corresponds to the integral of the Brier scores for the associated binary probability forecasts at all real-valued thresholds".
- Kernel form eq. (21): CRPS(F,x) = (1/2)·E_F|X - X'| - E_F|X - x|  (X, X' iid ~ F). Negative orientation (as usually used):
  CRPS*(F,x) = E_F|X - x| - (1/2)·E_F|X - X'|. "It is typically used in negative orientation".
  "In negative orientation, the CRPS can be reported in the same unit as the observations, and it generalizes the absolute error".
- Gaussian closed form (unnumbered, after eq. 21; re-typeset): CRPS*(N(mu,sigma^2), x) = sigma·[ z(2Phi(z) - 1) + 2phi(z) - 1/sqrt(pi) ], z = (x - mu)/sigma. [med: typeset reconstructed from garbled extraction; the formula is standard]
- Sample/ensemble: "If the predictive distribution takes the form of a sample of size n, then the right side of (20) can be evaluated
  in terms of the respective order statistics in a total of O(n log n) operations (Hersbach 2000, sec. 4.b)."
- From quantiles: FPP3 §5.9 "we can average the quantile scores over all values of p to obtain the Continuous Ranked Probability
  Score or CRPS". Bracher et al. App. A: CRPS(F,y) = INT_0^1 QS_tau(F,y) d tau (with QS_tau containing the factor 2).
  Practical estimator from K quantile levels tau_1..tau_K (equally spaced): CRPS ≈ (1/K)·SUM_k QS_{tau_k}.
- Weighted interval score (Bracher et al. eq. 1–2) — CRPS approximation from a median + K central intervals:
  WIS(F,y) = (1/(K + 1/2))·( w_0·|y - m| + SUM_{k=1..K} w_k·IS_{alpha_k}(F,y) ),  w_0 = 1/2, w_k = alpha_k/2.
  Paper: "under this choice of weights" WIS ≈ CRPS (for large K, alpha_k spread over (0,1)).
  Their quantile set: median + K = 11 intervals, alpha = {0.02, 0.05, 0.1, 0.2, ..., 0.9} -> "the 23 available quantiles". [high]

### 6.3 Interval coverage, interval (Winkler) score [high]
- Gneiting & Raftery 2007 eq. (43), negatively oriented, central (1-alpha)x100% interval [l,u]:
  S_alpha^int(l,u;x) = (u - l) + (2/alpha)(l - x)·1{x < l} + (2/alpha)(x - u)·1{x > u};
  "can be traced back to Dunsmore (1968), Winkler (1972), and Winkler and Murphy (1979)".
- FPP3 §5.9 Winkler score identical; and W_{alpha,t} = (Q_{alpha/2,t} + Q_{1-alpha/2,t}) / alpha (with FPP3's factor-2 Q).
- Empirical coverage = (1/T)·SUM 1{l_t <= y_t <= u_t}; compare with nominal 1-alpha. Gneiting et al. 2007 (JRSS-B) note coverage
  "can be read off the PIT histogram, as the area under the 10 and 18 central bins" (50 % / 90 % with 20 bins).

### 6.4 PIT histogram [high]
Source: Gneiting, Balabdaoui & Raftery 2007, "Probabilistic forecasts, calibration and sharpness", JRSS-B 69:243-268,
https://sites.stat.washington.edu/raftery/Research/PDF/Gneiting2007jrssb.pdf
- Definition: "if Ft is the predictive distribution and xt materializes, the transform is defined as pt = Ft(xt)."
- Ideal forecasts: "If the forecasts are ideal and Ft is continuous, then pt has a uniform distribution." Uniformity is
  "a necessary but not a sufficient" condition.
- Bins: "10 or 20 histogram bins generally seem adequate."
- Shapes: "Hump-shaped histograms indicate overdispersed predictive distributions"; "U-shaped histograms often correspond to
  predictive distributions that are too narrow"; "Triangle-shaped histograms are seen when the predictive distributions are biased."
- k-step-ahead: "The PITs for ideal k-step-ahead forecasts are at most k - 1 dependent" -> check ACF of PIT and its moments.
- Paradigm: "maximizing the sharpness of the predictive distributions subject to calibration."
- Uniformity test (implementation choice, not from paper): chi-square over B bins, X^2 = SUM_b (n_b - T/B)^2/(T/B), df = B-1;
  or KS on PIT; treat as exploratory when PITs are autocorrelated (paper warns formal tests are "hindered by complex dependence").

### 6.5 Brier score & Murphy (1973) decomposition [med-high — two independent secondary sources agree; primary AMS PDF returned HTTP 403]
Sources opened: https://en.wikipedia.org/wiki/Brier_score (cites Murphy, A.H. 1973, J. Appl. Meteor. 12(4):595–600) and the WMO
WWRP/WGNE Joint Working Group on Forecast Verification Research page https://jwgfvr.github.io/forecastverification (formerly CAWCR),
which gives the same 3-term decomposition, BSS = 1 - BS/BS_ref ("not strictly proper", unstable for small samples), reliability diagram
(below diagonal = over-forecasting), RPS, CRPS(F,y) = INT (F(x) - 1{y <= x})^2 dx, and rank-histogram shapes (U = spread too small,
dome = too large, asymmetric = bias; "a flat rank histogram does not necessarily indicate a good forecast").
- BS = (1/N)·SUM_t (f_t - o_t)^2 ("the mean squared error of the forecast"), o_t ∈ {0,1}.
- BS = REL - RES + UNC = (1/N)·SUM_k n_k (f_k - obar_k)^2 - (1/N)·SUM_k n_k (obar_k - obar)^2 + obar(1 - obar),
  K forecast-probability bins, n_k forecasts in bin k, obar_k observed frequency in bin k, obar base rate.
  (Exact only when all forecasts within a bin share the same f_k; with binned continuous f, there are within-bin terms.)
- BSS = 1 - BS/BS_ref, BS_ref = climatology (1/N)·SUM (obar - o_t)^2.
- Crane use: binary event "component fails / exceeds discard criterion within horizon H" (e.g., 30 days) with predicted
  probability from the RUL distribution, F_RUL(H).
- Gneiting & Raftery 2007 Example 1: quadratic/Brier score is the Bregman-type proper score from G(p) = SUM p_i^2 - 1 [high].

## Q7 Interval calibration for time series

### 7.1 Rolling-origin CV [high]
FPP3 §5.10 https://otexts.com/fpp3/tscv.html: "there are a series of test sets, each consisting of a single observation";
"no future observations can be used in constructing the forecast"; "The forecast accuracy is computed by averaging over the
test sets"; "the earliest observations are not considered as test sets"; can be "modified to allow multi-step errors".
Example settings in FPP3 code: .init = 3, .step = 1 (example, not a rule).

### 7.2 Adaptive conformal inference (Gibbs & Candès 2021, NeurIPS; arXiv 2106.00170) [high]
Source: https://ar5iv.labs.arxiv.org/html/2106.00170
- Score: S_t(X_t, y); set C_t(alpha_t) = { y : S_t(X_t,y) <= Qhat_t(1 - alpha_t) }; Qhat(p) = inf{ s : (1/|D_cal|)·SUM 1{S_r <= s} >= p }.
- err_t = 1 if Y_t ∉ C_t(alpha_t), else 0.
- Update (eq. 2): alpha_{t+1} = alpha_t + gamma·(alpha - err_t), alpha_1 = alpha.
- Prop. 4.1 (any data distribution, w.p.1): | (1/T)·SUM_{t=1..T} err_t - alpha | <= (max{alpha_1, 1 - alpha_1} + gamma)/(T·gamma).
- gamma = 0.005: "chosen because it was found to give relatively stable trajectories for" alpha_t; larger gamma = more adaptive but more volatile.
- Experiments: Qhat_t from the 1250 most recent scores (volatility example); local coverage over a 500-point window;
  CQR score S = max{ qhat(X; alpha/2) - y, y - qhat(X; 1 - alpha/2) }.
- Edge cases (verbatim): quantile convention "such that Q^t(0)=−∞ and Q^t(1)=∞"; Lemma 4.1 "With probability one we have that
  ∀t∈ℕ, αt∈[−γ,1+γ]"; proof: "αt<0⟹Q^t(1−αt)=∞⟹errt=0". So alpha_t < 0 -> infinite interval; alpha_t > 1 -> empty set (err=1).
  Paper intro of the rule: "fixing a step size parameter γ>0 we consider the simple online update"; "In our experiments we will take γ=0.005."
- Coverage-error bound example (my arithmetic from Prop. 4.1): alpha=0.1, gamma=0.005, T=1000 -> |mean err - 0.1| <= (0.9+0.005)/5 = 0.181
  (loose for short series; T=10,000 -> 0.018). Use larger gamma (≈0.01–0.05) only if regime shifts are fast; this is a tuning choice.

### 7.3 Split conformal prediction (Angelopoulos & Bates, "A Gentle Introduction to Conformal Prediction", arXiv 2107.07511) [high]
Source: https://ar5iv.labs.arxiv.org/html/2107.07511
- Recipe: scores s_i = s(X_i, Y_i) on n calibration points; "Compute q̂ as the ⌈(n+1)(1−α)⌉/n quantile of the calibration scores";
  C(X_test) = { y : s(X_test, y) <= q̂ }.
- Guarantee (eq. 1, exchangeable/i.i.d.): 1 - alpha <= P(Y_test ∈ C(X_test)) <= 1 - alpha + 1/(n+1) (upper bound needs tie-breaking).
- CQR score (eq. 6): s(x,y) = max{ t̂_{α/2}(x) - y, y - t̂_{1-α/2}(x) }; interval (eq. 7): [ t̂_{α/2}(x) - q̂, t̂_{1-α/2}(x) + q̂ ].
- Calibration size: "Choosing a calibration set of size n=1000 is sufficient for most purposes."
  Conditional coverage ~ Beta(n+1-l, l), l = floor((n+1)·alpha); Table 1 (delta=0.1, alpha=0.1): n = 22 / 102 / 2491 / 9812 / 244390
  for coverage slack eps = 0.1 / 0.05 / 0.01 / 0.005 / 0.001.
- Time series / drift: weighted conformal; "it often suffices to pick either a rolling window of size K or a smooth decay";
  w_i^fixed = 1{i >= n-K}, w_i^decay = 0.99^(n-i+1); drift bound (Thm 4): coverage >= 1 - alpha - 2·SUM w̃_i·eps_i (eps_i = TV distance).

### 7.4 Recommended protocol for the crane forecaster (synthesis of 7.1–7.3; settings are sourced where marked)
1. Rolling origin over simulated fleet histories (FPP3 §5.10): origins every inspection; never use data after the origin.
2. For each origin and horizon h, store predictive quantiles at tau ∈ {0.01, 0.025, 0.05, 0.1, ..., 0.9, 0.95, 0.975, 0.99}
   (Bracher et al. 23-quantile set) -> pinball, WIS≈CRPS, coverage of 50 %/80 %/90 % intervals, PIT histogram (10–20 bins).
3. Calibrate intervals with split conformal on a held-out set of *units* (exchangeable across units), CQR score;
   then online ACI per unit with gamma = 0.005 (Gibbs & Candès) to track drift (season/regime).
4. Report RUL metrics (Saxena PH / alpha-lambda / RA / CRA / C_M) and the PHM-2012 asymmetric score per unit.
Unverified (Q7): Tashman (2000) IJF 16(4):437–450 recommendations (rolling-origin, multiple-origin updating, fixed vs rolling window) —
abstract not available on IDEAS (https://ideas.repec.org/a/eee/intfor/v16y2000i4p437-450.html: "No abstract is available"), ScienceDirect paywalled.
Hyndman blog (https://robjhyndman.com/hyndsight/tscv/) repeats FPP3 wording ("evaluation on a rolling forecasting origin") and gives
R usage `tsCV(dj, rwf, drift=TRUE, h=1)`; does not cite Tashman.

## Q8 Health index practice

### 8.1 Standards (scope only — full texts paywalled)
- ISO 17359:2018 "Condition monitoring and diagnostics of machines — General guidelines", Edition 3, status Published.
  Abstract (iso.org, https://www.iso.org/standard/71194.html): "gives guidelines for the general procedures to be considered when setting up
  a condition monitoring programme for machines"; "This document is applicable to all machines." [high for scope]
- ISO 13381-1:2015 "Prognostics — Part 1: General guidelines": "provides guidance for the development and application of prognosis processes";
  WITHDRAWN 02.09.2025, replaced by ISO 13381-1:2025 "Condition monitoring and diagnostics of machine systems — Prognostics — Part 1:
  General guidelines and requirements": "This document provides guidance and requirements for the development and application of prognosis
  processes." Sources: https://evs.ee/en/iso-13381-1-2015 and https://evs.ee/en/iso-13381-1-2025 (Estonian standards body). [high for scope/status]
  (iso.org page for 13381-1 was behind a bot challenge — not bypassed.)
  Unverified (search snippet only): the 2015 scope says other parts cover trending (-2), cyclic-driven life usage (-3), useful-life-remaining models (-4).
- Health index definition in prognostics metrics (Saxena et al. 2010, NTRS link in Q5): "h can be considered a normalized aggregate of
  health indicators (relevant features) and operational conditions." [high]

### 8.2 Condition grades -> actions (published examples)
(a) Crane wire rope, ISO 4309:2010 severity rating (Q2.2): "20% is 'slight', 80% 'very high' and 100% means that it is time to 'discard'";
    cumulative effect of several deterioration modes is combined. [med] — the one crane-specific 0–100 % scale found.
(b) Power-transformer health index (0–100 %): open-access article PMC11711404 ("Predecting [sic] power transformer health index and life
    expectation based on digital twins and multitask LSTM-GRU model", https://pmc.ncbi.nlm.nih.gov/articles/PMC11711404/):
    "when the state of the transformer's THI is "Very Good" (85% or higher), it is expected to have a lifetime of over 15 years";
    70–85 % "Good" -> more than 10 years; "Very Poor" (<30 %) -> taken out of service and replaced immediately. [med]
    UNVERIFIED — fuller 5-band table seen only in search-result snippets (no page opened; commonly attributed to Jahromi et al. 2009,
    IEEE Electrical Insulation Magazine 25(2)); listed so the bands can be checked later, not to be cited as sourced:
    | HI | Condition | Expected life | Action |
    |---|---|---|---|
    | 85–100 | Very good | > 15 y | normal maintenance |
    | 70–85 | Good | > 10 y | normal maintenance |
    | 50–70 | Fair | < 10 y | increase diagnostic testing / plan remedial work |
    | 30–50 | Poor | < 3 y | plan replacement/rebuild |
    | 0–30 | Very poor | end of life | assess risk; remove/replace immediately |
(c) Statistical alarm setting (Fluke Reliability vibration-standards presentation,
    https://images.info.fluke.com/Web/FlukeFDS/{1d996a95-5add-4731-9b45-0acfd1435fb4}_bpw_20230329_limitations-benefits-of-vibration-standard_luca-barraco.pdf):
    "By using average plus 2 sigma, you 'believe' there is a 95% probability that future readings should be below this limit";
    "the thresholds are 3 that indicate respectively pre-alert (green), alert (yellow), alarm (red)". [med — vendor slide deck,
    publicly hosted although its footer reads "Confidential Document"; use only as practice evidence]
(d) Asymmetric RUL penalty used in an official challenge (PHM 2012, Q3.2): late predictions penalised 4x harder (scale 5 vs 20). [high]

### 8.3 Proposed health-score construction for the crane sim (synthesis — choices flagged)
- Per component c with monotone damage D_c(t) and discard/failure limit L_c (ISO 4309 limits for rope; lining min thickness; wear limit):
  severity_c = 100·D_c/L_c (ISO 4309-style "percentage towards discard"); H_c = 100 - min(100, severity_c).
- Probabilistic variant (needs the RUL forecaster): H_c = 100·P(RUL_c > tau_action), tau_action = lead time to act (Saxena's alpha-band /
  EoUP idea) — directly scoreable with Brier/BSS (Q6.5).
- Bands for actions: transformer-HI cut points 85 / 70 / 30 are in the opened source (b); the 50 cut point comes only from the
  unverified 5-band table; ISO 4309 wording for rope (slight..discard);
  alarm thresholds from fleet statistics mean + 2 sigma (c). Cut points for non-rope crane components are an engineering choice, not sourced.
- Crane-level score: min over safety-critical components (rope, brakes, hook/structure) — series-system logic (NASA TP 3.4:
  "the system Weibull slope approximates that of the lowest lived component"); averaging only for non-critical items. [choice]

Unverified (Q8): ISO 20816-1/10816 vibration zones A–D wording (A new machine, B unrestricted long-term, C limited period, D damage);
DNV / ABS / LR crane or machinery CBM guidance content (ABS "Guidance Notes on Equipment Condition Monitoring Techniques", ABS 0224, 2016
exists per standards.ie listing in search results — content not opened); Ofgem DNO CNAIM health score 0.5–10 and HI1–HI5 bands;
CIGRE TB 761 transformer HI.

## Sources actually opened (for audit)
- Saxena et al. 2010 IJPHM: https://ntrs.nasa.gov/api/citations/20100039169/downloads/20100039169.pdf
- Gneiting & Raftery 2007 JASA: https://sites.stat.washington.edu/raftery/Research/PDF/Gneiting2007jasa.pdf
- Gneiting, Balabdaoui & Raftery 2007 JRSS-B: https://sites.stat.washington.edu/raftery/Research/PDF/Gneiting2007jrssb.pdf
- Bracher et al. 2021 (WIS): https://ar5iv.labs.arxiv.org/html/2005.12881
- FPP3 §5.9 / §5.10: https://otexts.com/fpp3/distaccuracy.html , https://otexts.com/fpp3/tscv.html ; https://robjhyndman.com/hyndsight/tscv/
- Gibbs & Candès 2021 (ACI): https://ar5iv.labs.arxiv.org/html/2106.00170
- Angelopoulos & Bates (conformal): https://ar5iv.labs.arxiv.org/html/2107.07511
- Brier/Murphy: https://en.wikipedia.org/wiki/Brier_score ; https://jwgfvr.github.io/forecastverification
- van Noortwijk 2009 abstract: https://ideas.repec.org/a/eee/reensy/v94y2009i1p2-21.html
- Strauss & Demissie REC 2012 (gamma process restatement): https://rec2012.fce.vutbr.cz/documents/papers/strauss.pdf
- Gamma-process Bayesian RUL (laser, train wheels): https://arxiv.org/abs/2212.02688 , https://ar5iv.labs.arxiv.org/html/2212.02688
- Inverse Gaussian / Brownian first passage: https://en.wikipedia.org/wiki/Inverse_Gaussian_distribution
- IEEE PHM 2012 / PRONOSTIA challenge doc: https://raw.githubusercontent.com/wkzs111/phm-ieee-2012-data-challenge-dataset/master/IEEEPHM2012-Challenge-Details.pdf
- NASA PCoE data repository (IMS, FEMTO entries): https://www.nasa.gov/intelligent-systems-division/discovery-and-systems-health/pcoe/pcoe-data-set-repository/
- MathWorks exponentialDegradationModel: https://www.mathworks.com/help/predmaint/ref/exponentialdegradationmodel.html
- Zaretsky NASA/TP-2013-215305: https://ntrs.nasa.gov/api/citations/20130011515/downloads/20130011515.pdf
- XJTU-SY Bayesian RUL article: https://pmc.ncbi.nlm.nih.gov/articles/PMC7795917/
- Barringer Weibull database (2016 snapshot): https://web.archive.org/web/2016id_/http://www.barringer1.com/wdbase.htm
- USBR FIST 3-1 insulation testing: https://www.usbr.gov/power/data/fist/fist3_1/vol3-1.pdf
- ISO 4309 discard tables (Certex): https://www.certex.dk/en/knowhow/articles-about-steel-wire-rope/discard-of-steel-wire-rope
- ISO 4309:2010 diameter/severity (Cranes Today): https://www.cranestodaymagazine.com/?p=4775 ; Carl Stahl: https://www.carlstahl.com/gb/en/services/wire-rope-diametric-reduction-calculator
- Feyrer OIPEEC abstract: https://www.oipeec.org/?p=15755
- ISO 17359:2018: https://www.iso.org/standard/71194.html ; ISO 13381-1: https://evs.ee/en/iso-13381-1-2015 , https://evs.ee/en/iso-13381-1-2025
- Transformer THI: https://pmc.ncbi.nlm.nih.gov/articles/PMC11711404/
- Vibration alarm practice (Fluke): https://images.info.fluke.com/Web/FlukeFDS/{1d996a95-5add-4731-9b45-0acfd1435fb4}_bpw_20230329_limitations-benefits-of-vibration-standard_luca-barraco.pdf
Blocked/not opened: van Noortwijk full text (paywall), Murphy 1973 (AMS 403), Nectoux PRONOSTIA HAL (bot wall), iso.org 13381 (bot challenge,
not bypassed), MDPI transformer-HI review (403), ResearchGate PDFs (403), Tashman 2000 (no abstract/paywall), Feyrer book (paywall).
