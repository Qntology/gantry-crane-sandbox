# 사용자 현장 프리셋

`crane_demo site save` 가 바꾼 현장 프리셋을 이 폴더에 `<id>.json` 으로 저장한다 (`--presets DIR` 로 바꿀 수 있다).
내장 프리셋(`crane_demo site list` 의 '내장')은 코드에 있으며 같은 이름으로 덮어쓸 수 없다.

```bash
crane_demo site show incheon_steel_stockyard > my.json      # 내장 프리셋을 꺼내 고친 뒤
crane_demo site save my_yard --file my.json                  # 파일로 저장하거나
crane_demo site save my_yard --from incheon_steel_stockyard --set throughput.moves_per_hour=10 --set environment.corrosivity=C5
crane_demo operate --site my_yard --days 90 --asset YARD-01
```

경로는 점으로 잇는다: `throughput.*` (물량), `loads.*` (하중 스펙트럼 · 중량물 비율), `environment.*` (실내 · 부식성 · 분진 · 열원 · 레일),
`climate` (기상청 지점 id 또는 전체 기후 자료), `operations.*` (숙련 · 신호수 · 출입 통제 · 정비 · 점검 주기 · 조도), `alerts.*` (알림 문턱).
항목 설명은 [docs/site-operation.md](../docs/site-operation.md).
