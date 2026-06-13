#!/usr/bin/sh

echo "Building the wasm module..."
cd crates/rvemu-wasm
cargo build --release -p rvemu-wasm --target=wasm32v1-none
cd ../..
echo "Applying wasm transformation..."
cargo run --release -p rvemu-wasm-gen -- ./target/wasm32v1-none/release/rvemu_wasm.wasm rvemu-web/public/rvemu.wasm