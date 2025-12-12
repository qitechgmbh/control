
#!/usr/bin/env bash
set -euo pipefail

BUILD_DIR="${BUILD_DIR:-target}"
CSV_FILE="${CSV_FILE:-compile_metrics.csv}"

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <build command...>"
  echo "Example: $0 cargo build --release"
  exit 1
fi

# Time the build
compile_start_ns=$(date +%s%N)
"$@"
compile_end_ns=$(date +%s%N)
compile_ms=$(( (compile_end_ns - compile_start_ns) / 1000000 ))

# Count LLVM IR lines (will be 0 unless you emit .ll files)
if [ -d "$BUILD_DIR" ]; then
  llvm_ir_lines=$(
    find "$BUILD_DIR" -type f -name '*.ll' -print0 2>/dev/null \
      | xargs -0 cat 2>/dev/null \
      | tr -cd '\n' \
      | wc -c
  )
else
  llvm_ir_lines=0
fi

# Auto-detect binary path
if [ -n "${BINARY_PATH:-}" ]; then
  bin_path="$BINARY_PATH"
elif [ -f "./target/release/server" ]; then
  bin_path="./target/release/server"
elif [ -f "./target/debug/server" ]; then
  bin_path="./target/debug/server"
else
  bin_path=""
fi

# Binary size in bytes (0 if not found)
if [ -n "$bin_path" ] && [ -f "$bin_path" ]; then
  binary_size_bytes=$(stat -c '%s' "$bin_path")
else
  binary_size_bytes=0
fi

finished_timestamp_ms=$(($(date +%s%N) / 1000000))

if [ ! -f "$CSV_FILE" ]; then
  echo "finished_timestamp_ms,compile_ms,llvm_ir_lines,binary_size_bytes" > "$CSV_FILE"
fi

echo "${finished_timestamp_ms},${compile_ms},${llvm_ir_lines},${binary_size_bytes}" \
  >> "$CSV_FILE"
