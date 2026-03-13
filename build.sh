#!/bin/sh
set -e
echo "ox∅ build"

# Use rustup-managed toolchain (not Homebrew cargo)
CARGO="$HOME/.cargo/bin/cargo"
WASM_BINDGEN="$HOME/.cargo/bin/wasm-bindgen"

$CARGO build --target wasm32-unknown-unknown --release
$WASM_BINDGEN --target web --out-dir pkg target/wasm32-unknown-unknown/release/oxvoid.wasm

echo "done → pkg/"
