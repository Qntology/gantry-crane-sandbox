# 1.6단계 리서치 메모 (2026-10-20)

[기획 · 체크리스트](../../plan-stage2.md) 의 P1 리서치 결과. 값마다 열어 본 URL · 짧은 인용 · 신뢰도(high · med · low)가 붙어 있고,
확인하지 못한 값은 각 파일의 'Unverified' 절에 따로 있다. 모델에 넣을 때는 'Unverified' 값을 출처 있는 값처럼 쓰지 않는다.

| 파일 | 내용 | 상태 |
| --- | --- | --- |
| [R1_marine_corrosion.md](R1_marine_corrosion.md) | ISO 9223 용량–반응식 · 범주 경계 · 염분/SO₂/젖음 등급, ISO 9224 지수, 해안 거리별 비래염분(국내 27개 지역 · 6개 지점 거리 감쇠), 바람 · 파도 영향, 차폐 · 세척, 조해 습도 | 끝남 — Q4 실측 첫해 부식 표 33행 (fieldcal 부식 벤치마크 12곳의 출처) |
| [R2_component_marine.md](R2_component_marine.md) | ISO 12944 내구성 · 도장 계통, 용융아연 수명 · 이중 계통 배수, 해안 로프, 모터 절연저항 · 히터 · 고장 비율, ISA-71.04, 수분과 베어링 수명, 태풍 매미 · 계류 지침, 항만 크레인 방식 사양 | 끝남 |
| [R3_cargo.md](R3_cargo.md) | 컨테이너 무게 분포 · 오신고, 부산항 생산성, 코일 · 프리캐스트 · 블록 무게, 풍압 면적 허용 풍속식, 위험물 · 중량물 기준 | 끝남 — 특수 화물 비율은 Unverified (가정 1 %) |
| [R4_stats_gaps.md](R4_stats_gaps.md) | 국내 크레인 사망 · 대수(노출 기반 사고율), 해외 통계 · TT Club, KDS 41 12 00 기본풍속, 부산 HKT 크레인 가용률 · MTBF · MTTR, 시정 기준, 태풍 매미, 조도 | 끝남 |
| [R5_degradation_scoring.md](R5_degradation_scoring.md) | 감마 과정 열화, 로프 단선 증가, 베어링 열화, 마모 산포 · 와이블 형상, Saxena 예측 지표, CRPS · PIT · Brier, 시간 순 교차검증 · 등각 예측, 건강 지수 관행 | 끝남 |
| [prefetch_weather.py](prefetch_weather.py) | Open-Meteo 무료 한도(분당 600 · 시간당 5,000 · 하루 10,000, 변수 10개 · 2주 초과 가중)에 맞춰 ERA5 · CAMS · 파고를 받아 `out/weather_cache` 에 저장 | 이어 받기 가능 (있는 파일은 건너뜀) |
| [prefetch_2026-10-20.log](prefetch_2026-10-20.log) | 수집 기록 — 첫 날 기상청 6개 지점 일별 1991–2020 (18개 파일, 가중 4,696), 이어서 기상청 시간별 2022–2023 · 항만 14곳 일별 2014–2023 · 시간별 2023 · CAMS SO₂ · 파고 (가중 5,034) | 끝남 → `crane_demo weather build` → `data/climate/ports.json` |

이어 받기:

```bash
python -I docs/research/stage2/prefetch_weather.py data/climate/port_sites.json out/weather_cache out/prefetch.log
```
