# data

- `sample/` — 보정 명령의 **형식 확인용 합성 데이터**다 (실측이 아님). `cargo run --release --example make_samples` 로 다시 만든다.
  - `pendulum_log.csv` — 셰이핑 없이 이동한 뒤의 자유 흔들림 (등가 로프 7.0 m, 감쇠비 0.01)
  - `coastdown_log.csv` — 구동 없이 굴러가는 주행 속도 (구름 저항 0.006, 점성 200 N·s/m, 31.2 t)
  - `winding_log.csv` — 11 kW 모터 S3 40% 10분 주기의 전류 · 속도 · 권선 온도
  - `kaggle_format_sample.csv` — Kaggle "Electric Motor Temperature" 와 같은 열 이름 · 2 Hz 형식
- `raw/` — `scripts/fetch_open_data.sh` 가 받는 공개 데이터 (깃에는 넣지 않는다)

실데이터 목록과 라이선스는 [../docs/data-sources.md](../docs/data-sources.md).
