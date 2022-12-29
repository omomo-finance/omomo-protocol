#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --manifest-path ./contracts/leverage_trading/Cargo.toml --target wasm32-unknown-unknown --release

cp ./contracts/leverage_trading/target/wasm32-unknown-unknown/release/*.wasm ./contracts/leverage_trading/res/

