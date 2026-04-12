#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_PID=""
FRONTEND_PID=""

cleanup() {
  if [[ -n "${FRONTEND_PID}" ]] && kill -0 "${FRONTEND_PID}" 2>/dev/null; then
    kill "${FRONTEND_PID}" 2>/dev/null || true
  fi

  if [[ -n "${BACKEND_PID}" ]] && kill -0 "${BACKEND_PID}" 2>/dev/null; then
    kill "${BACKEND_PID}" 2>/dev/null || true
  fi
}

trap cleanup EXIT INT TERM

echo "[mock-runner] Starting backend in mock-machine mode..."
(
  cd "${ROOT_DIR}"
  ./cargo_run_linux.sh mock-machine
) &
BACKEND_PID=$!

sleep 2

echo "[mock-runner] Starting Electron frontend..."
(
  cd "${ROOT_DIR}/electron"
  npm start
) &
FRONTEND_PID=$!

wait "${FRONTEND_PID}"
