# login
# near login

# build & test
./build.sh && ./test.sh

# ============================
# REDEPLOY SCRIPT           ||
# ============================
echo 'y' | near deploy controller.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm

echo 'y' | near deploy weth.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
echo 'y' | near deploy stnear.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
echo 'y' | near deploy wbtc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
echo 'y' | near deploy aurora.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
echo 'y' | near deploy usdt.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
echo 'y' | near deploy usdc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
echo 'y' | near deploy dai.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
echo 'y' | near deploy token.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm

echo 'y' | near deploy wnear_market.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy weth_market.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy dstnear.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy dwbtc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy daurora.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy usdt_market.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy usdc_market.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm

echo 'y' | near deploy ddai.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm