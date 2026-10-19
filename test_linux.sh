#!/usr/bin/env bash
# 라이브러리 · 통합 테스트, 물리 엔진만 빌드(--no-default-features), 정밀 로프 해석기, CLI 데모.
# 사용: ./test_linux.sh [cpu|vulkan|rocm]
set -euo pipefail
cd "$(dirname "$0")"
backend="${1:-cpu}"
stamp="$(date +%Y%m%d_%H%M%S)"
log="test_logs/${stamp}_${backend}"
mkdir -p "$log"
run() { local name="$1"; shift; echo "== $name: $*"; if "$@" > "$log/$name.log" 2>&1; then echo "   ok"; else echo "   FAILED (see $log/$name.log)"; tail -20 "$log/$name.log"; exit 1; fi; }
if [ "$backend" = cpu ]; then
  run 01_tests cargo test
else
  export CRANE_DEVICE="$backend"
  run 01_tests cargo test --no-default-features --features "$backend store"
fi
run 02_physics_only cargo test --no-default-features --lib
run 03_mbd bash -c "cd crates/crane_mbd && cargo test --release"
run 04_demo cargo run --release --bin crane_demo -- demo
run 05_pipeline cargo run --release --bin crane_demo -- pipeline scenarios/gantry_10t_demo.json --compare --out "$log/out"
grep -h "test result" "$log"/01_tests.log "$log"/02_physics_only.log | sed 's/^/   /'
echo "logs: $log"
