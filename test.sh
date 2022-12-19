#!/bin/bash
set -e

cargo test --manifest-path ./contracts/Cargo.toml -- --nocapture
cargo test --manifest-path ./contracts/leverage_trading/Cargo.toml -- --nocapture

