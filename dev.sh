#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

PORT="${PORT:-8080}"
YAWN_PORT="${YAWN_PORT:-3001}"
LOG=target/prerender-dev.log

cargo build --release --manifest-path tools/prerender/Cargo.toml
mkdir -p target

tools/prerender/target/release/prerender serve \
    --port "$PORT" \
    --upstream "http://127.0.0.1:$YAWN_PORT" >"$LOG" 2>&1 &
proxy=$!
trap 'kill "$proxy" 2>/dev/null || true' EXIT

echo "og proxy on http://localhost:$PORT, yawn dev on $YAWN_PORT (log: $LOG)"

yawn dev --port "$YAWN_PORT" "$@"
