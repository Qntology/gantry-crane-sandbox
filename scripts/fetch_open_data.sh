#!/usr/bin/env bash
# 공개 저장소에 있는 실데이터 · 모델을 data/raw 로 받는다. 라이선스는 docs/data-sources.md 참고.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p data/raw
clone() { [ -d "data/raw/$2" ] && { echo "있음: data/raw/$2"; return; }; git clone --depth 1 "$1" "data/raw/$2"; }

# INTECO 3D 크레인 식별 모델 (GPL-3.0) — 마찰 · 흔들림 감쇠
clone https://github.com/jrgvicente/Crane3DSim Crane3DSim
# XJTU-SY 베어링 수명 데이터 안내 저장소
clone https://github.com/WangBiaoXJTU/xjtu-sy-bearing-datasets xjtu-sy

# Kaggle: Paderborn Electric Motor Temperature (CC BY-SA 4.0) — Kaggle API 토큰이 필요하다
if command -v kaggle >/dev/null 2>&1; then
  kaggle datasets download -d wkirgsn/electric-motor-temperature -p data/raw/electric-motor-temperature --unzip
else
  echo "kaggle CLI 가 없습니다: pip install kaggle 후 ~/.kaggle/kaggle.json 을 두고 다시 실행하세요"
  echo "  kaggle datasets download -d wkirgsn/electric-motor-temperature -p data/raw/electric-motor-temperature --unzip"
fi

cat <<'MSG'

직접 받아야 하는 자료:
  - Aalto "Driving Smart Crane With Various Loads" (IEEE DataPort, 구독)
      https://ieee-dataport.org/documents/driving-smart-crane-various-loads
  - Mendeley 타워 크레인 궤적 (라이선스 확인 후)
      https://data.mendeley.com/datasets/x69hyyv3zw/2
  - Paderborn KAt 베어링 (CC BY-NC 4.0)
      https://mb.uni-paderborn.de/en/kat/research/bearing-datacenter/data-sets-and-download
  - Felixstowe RTG 10 Hz 로그 (저자에게 요청)
      https://e-space.mmu.ac.uk/620719/

보정 예:
  cargo run --release --bin crane_demo -- calibrate kaggle data/raw/electric-motor-temperature/measures_v2.csv --profile 4
MSG
