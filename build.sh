#!/bin/sh
set -e
echo "ox∅ build"
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir dist target/wasm32-unknown-unknown/release/oxvoid.wasm
echo "done → dist/"
