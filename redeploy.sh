# login
# near login

# build & test
./build.sh && ./test.sh

# deploy markets
echo 'y' | near deploy dweth_beta.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy dwnear_beta.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy dusdt_beta.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
echo 'y' | near deploy dusdc_beta.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
# deploy controller
echo 'y' | near deploy controller_beta.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm


near view controller_beta.nearlend.testnet view_markets '{}' --accountId controller_beta.nearlend.testnet
near view controller_beta.nearlend.testnet view_prices '{ "dtokens": ["dwnear_beta.nearlend.testnet", "dweth_beta.nearlend.testnet", "dusdt_beta.nearlend.testnet", "dusdc_beta.nearlend.testnet"] }' --accountId controller_beta.nearlend.testnet 

near view weth_beta.nearlend.testnet ft_balance_of '{"account_id": "dweth_beta.nearlend.testnet"}'
near view wnear_beta.nearlend.testnet ft_balance_of '{"account_id": "dwnear_beta.nearlend.testnet"}'
near view usdt_beta.nearlend.testnet ft_balance_of '{"account_id": "dusdt_beta.nearlend.testnet"}'
near view usdc_beta.nearlend.testnet ft_balance_of '{"account_id": "dusdc_beta.nearlend.testnet"}'