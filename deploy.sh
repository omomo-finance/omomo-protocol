# login
# near login

# build & test
./build.sh && ./test.sh

# clean up previuos deployment
near delete weth_beta.nearlend.testnet nearlend.testnet
near delete dweth_beta.nearlend.testnet nearlend.testnet
near delete wnear_beta.nearlend.testnet nearlend.testnet
near delete dwnear_beta.nearlend.testnet nearlend.testnet
near delete controller_beta.nearlend.testnet nearlend.testnet

# create corresponding accoutns
near create-account weth_beta.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 10
near create-account dweth_beta.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 10

near create-account wnear_beta.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 10
near create-account dwnear_beta.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 10

near create-account controller_beta.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 10

# redeploy contracts
near deploy weth_beta.nearlend.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '
{"owner_id": "nearlend.testnet", "name": "Wrapped Ethereum", "symbol": "WETH", "total_supply": "1000000000"}'
near deploy wnear_beta.nearlend.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "nearlend.testnet", "name": "Wrapped Near", "symbol": "WNEAR", "total_supply": "1000000000"}'

near deploy dweth_beta.nearlend.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "nearlend.testnet", "underlying_token_id": "weth_beta.nearlend.testnet", "controller_account_id": "controller_beta.nearlend.testnet", "initial_exchange_rate": "10000", "interest_rate_model": {"kink": "8000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100"}}'
near deploy dwnear_beta.nearlend.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "nearlend.testnet", "underlying_token_id": "wnear_beta.nearlend.testnet", "controller_account_id": "controller_beta.nearlend.testnet", "initial_exchange_rate": "10000", "interest_rate_model": {"kink": "8000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100"}}'

near deploy controller_beta.nearlend.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "nearlend.testnet", "oracle_account_id": "oracle_beta.nearlend.testnet"}'

# fund dweth_beta.nearlend.testnet
near call weth_beta.nearlend.testnet mint '{"account_id": "dweth_beta.nearlend.testnet", "amount": "1000000000"}' --accountId nearlend.testnet
# fund dwnear_beta.nearlend.testnet
near call wnear_beta.nearlend.testnet mint '{"account_id": "dwnear_beta.nearlend.testnet", "amount": "1000000000"}' --accountId nearlend.testnet


# register market
near call controller_beta.nearlend.testnet add_market '{"asset_id": "weth_beta.nearlend.testnet", "dtoken": "dweth_beta.nearlend.testnet", "ticker_id": "weth"}' --accountId nearlend.testnet
near call controller_beta.nearlend.testnet add_market '{"asset_id": "wnear_beta.nearlend.testnet", "dtoken": "dwnear_beta.nearlend.testnet", "ticker_id": "wnear"}' --accountId nearlend.testnet
near view controller_beta.nearlend.testnet view_markets '{}' --accountId controller_beta.nearlend.testnet
