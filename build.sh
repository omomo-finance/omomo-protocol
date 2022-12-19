#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --manifest-path ./contracts/Cargo.toml --target wasm32-unknown-unknown --release
RUSTFLAGS='-C link-arg=-s' cargo build --manifest-path ./contracts/leverage_trading/Cargo.toml --target wasm32-unknown-unknown --release

cp ./**/target/wasm32-unknown-unknown/release/*.wasm ./res/
cp ./**/**/target/wasm32-unknown-unknown/release/*.wasm ./res/
cp ./res/test_utoken.wasm ./**/**/res
cp ./res/limit_orders.wasm ./**/**/res

