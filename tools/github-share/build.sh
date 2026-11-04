#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo build --locked --release --target wasm32-unknown-unknown --manifest-path tools/github-share/Cargo.toml
wasm-bindgen tools/github-share/target/wasm32-unknown-unknown/release/github_share.wasm \
    --target web --out-dir tools/github-share/pkg
