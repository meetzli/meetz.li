#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

PORT="${PORT:-8080}"
PUBLIC=dist/public

cargo build --release --manifest-path tools/prerender/Cargo.toml

rm -rf dist
yawn build --static
tools/prerender/target/release/prerender "$PUBLIC"
cp cloudflare/_routes.json "$PUBLIC/_routes.json"

[ -d tools/github-share/pkg ] || npm run build:wasm

echo "release static build on http://localhost:$PORT"

exec npx wrangler pages dev "$PUBLIC" --port "$PORT" --ip 0.0.0.0
