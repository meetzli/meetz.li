#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

PORT="${PORT:-8080}"
DX_PORT="${DX_PORT:-8082}"
LOG=target/prerender-dev.log

cargo build --release --manifest-path tools/prerender/Cargo.toml
mkdir -p target

tools/prerender/target/release/prerender serve \
    --port "$PORT" \
    --upstream "http://127.0.0.1:$DX_PORT" >"$LOG" 2>&1 &
proxy=$!
trap 'kill "$proxy" 2>/dev/null || true' EXIT

echo "og proxy on http://localhost:$PORT, dx serve on $DX_PORT (log: $LOG)"

dx serve --web --port "$DX_PORT" --hot-patch=false "$@"
