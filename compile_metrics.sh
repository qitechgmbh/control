#!/usr/bin/env bash
set -euo pipefail

BUILD_DIR="${BUILD_DIR:-target}"
CSV_FILE="${CSV_FILE:-compile_metrics.csv}"
DEFAULT_PKG="server"

if [ "$#" -eq 0 ]; then
  set -- cargo build
fi

if [ "${1:-}" = "cargo" ] && [ "${2:-}" = "build" ]; then
  cargo clean
  BUILD_CMD=(cargo rustc -p "$DEFAULT_PKG")
  shift 2
  BUILD_CMD+=("$@")
  BUILD_CMD+=(-- --emit=llvm-ir -C debuginfo=0)
else
  BUILD_CMD=("$@")
fi

compile_start_ns=$(date +%s%N)
"${BUILD_CMD[@]}"
compile_end_ns=$(date +%s%N)
compile_ms=$(( (compile_end_ns - compile_start_ns) / 1000000 ))

if [ -d "$BUILD_DIR" ]; then
  llvm_ir_lines=$(
    find "$BUILD_DIR" -type f -name '*.ll' -print0 \
      | xargs -0 cat \
      | wc -l
  )
else
  llvm_ir_lines=0
fi

bin_path=""

if [ -n "${BINARY_PATH:-}" ] && [ -f "$BINARY_PATH" ]; then
  bin_path="$BINARY_PATH"
elif [ -f "./target/release/server" ]; then
  bin_path="./target/release/server"
elif [ -f "./target/debug/server" ]; then
  bin_path="./target/debug/server"
elif [ -f "./server/target/release/server" ]; then
  bin_path="./server/target/release/server"
elif [ -f "./server/target/debug/server" ]; then
  bin_path="./server/target/debug/server"
fi

if [ -n "$bin_path" ]; then
  binary_size_bytes=$(stat -c '%s' "$bin_path")
else
  binary_size_bytes=0
fi

finished_timestamp_ms=$(($(date +%s%N) / 1000000))
finished_datetime=$(date -Iseconds)

if [ ! -f "$CSV_FILE" ]; then
  echo "datetime,finished_timestamp_ms,compile_ms,llvm_ir_lines,binary_size_bytes" > "$CSV_FILE"
fi

echo "${finished_datetime},${finished_timestamp_ms},${compile_ms},${llvm_ir_lines},${binary_size_bytes}" \
  >> "$CSV_FILE"

echo "Metrics recorded in $CSV_FILE"
