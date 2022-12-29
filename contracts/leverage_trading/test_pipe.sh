#!/bin/bash
set -e

cargo test --manifest-path ./contracts/leverage_trading/Cargo.toml -- --nocapture
