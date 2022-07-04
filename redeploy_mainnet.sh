# login
# near login

# build & test
./build.sh && ./test.sh

# ============================
# REDEPLOY SCRIPT           ||
# ============================
near deploy controller.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm

near deploy wnear.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy weth.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy stnear.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy wbtc.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy aurora.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy usdt.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy usdc.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy dai.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
near deploy omomo.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm