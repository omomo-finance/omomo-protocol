# login
# near login

# build & test
./build.sh && ./test.sh

# ============================
# REDEPLOY SCRIPT           ||
# ============================

near deploy controller.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm

near deploy weth.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
near deploy stnear.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
near deploy wbtc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
near deploy aurora.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
near deploy usdt.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
near deploy usdc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
near deploy dai.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
near deploy token.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm

near deploy dwnear.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy dweth.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy dstnear.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy dwbtc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy daurora.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy dusdt.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy dusdc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm

near deploy ddai.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm