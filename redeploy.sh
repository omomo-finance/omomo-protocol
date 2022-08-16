# login
# near login

# build & test
./build.sh && ./test.sh

# deploy markets
echo 'y' | near deploy weth_market.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy wnear_market.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy usdt_market.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy usdc_market.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
# deploy controller
echo 'y' | near deploy controller.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm


near view controller.nearlend.testnet view_markets '{}' --accountId controller.nearlend.testnet
near view controller.nearlend.testnet view_prices '{ "dtokens": ["wnear_market.nearlend.testnet", "weth_market.nearlend.testnet", "usdt_market.nearlend.testnet", "usdc_market.nearlend.testnet"] }' --accountId controller.nearlend.testnet 

near view weth.nearlend.testnet ft_balance_of '{"account_id": "weth_market.nearlend.testnet"}'
near view wnear.nearlend.testnet ft_balance_of '{"account_id": "wnear_market.nearlend.testnet"}'
near view usdt.nearlend.testnet ft_balance_of '{"account_id": "usdt_market.nearlend.testnet"}'
near view usdc.nearlend.testnet ft_balance_of '{"account_id": "usdc_market.nearlend.testnet"}'