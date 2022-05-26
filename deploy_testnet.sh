# login
# near login

# build & test
./build.sh && ./test.sh

# Clean up previous deployment
near delete *.omomo-finance.testnet omomo-finance.testnet


# create corresponding accounts
## The oracle contract account
near create-account oracle.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 5
## The controller contract account
near create-account controller.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1

## Token contract accounts, create all except wnear, we assume to use "wrap.testnet"
near create-account weth.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account stnear.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account wbtc.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account aurora.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account usdt.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account usdc.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account dai.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1

near create-account token.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1

## Dtoken contract accounts
near create-account dwnear.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account dweth.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account dstnear.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account dwbtc.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account daurora.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account dusdt.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account dusdc.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1
near create-account ddai.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 1

near deploy controller.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "omomo-finance.testnet", "oracle_account_id": "oracle.omomo-finance.testnet"}'

near deploy weth.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Wrapped Ethereum", "symbol": "WETH", "total_supply": "1000000000"}'
near deploy stnear.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Wrapped staked NEAR", "symbol": "stNEAR", "total_supply": "1000000000"}'
near deploy wbtc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Wrapped Bitcoin", "symbol": "WBTC", "total_supply": "1000000000"}'
near deploy aurora.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Wrapped Aurora", "symbol": "AURORA", "total_supply": "1000000000"}'
near deploy usdt.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Wrapped USDT", "symbol": "USDT", "total_supply": "1000000000"}'
near deploy usdc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Wrapped USDC", "symbol": "USDC", "total_supply": "1000000000"}'
near deploy dai.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Wrapped DAI", "symbol": "DAI", "total_supply": "1000000000"}'
near deploy token.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "omomo-finance.testnet", "name": "OMOMO token", "symbol": "OMOMO", "total_supply": "1000000000"}'

near deploy dwnear.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "omomo-finance.testnet", "underlying_token_id": "wrap.testnet", "controller_account_id": "controller.omomo-finance.testnet, "initial_exchange_rate": "10000000000", "interest_rate_model": {"kink": "8000000000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100000000"}}'
near deploy dweth.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "omomo-finance.testnet", "underlying_token_id": "weth.omomo-finance.testnet", "controller_account_id": "controller.omomo-finance.testnet, "initial_exchange_rate": "10000000000", "interest_rate_model": {"kink": "8000000000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100000000"}}'
near deploy dstnear.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "omomo-finance.testnet", "underlying_token_id": "stnear.omomo-finance.testnet", "controller_account_id": "controller.omomo-finance.testnet, "initial_exchange_rate": "10000000000", "interest_rate_model": {"kink": "8000000000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100000000"}}'
near deploy dwbtc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "omomo-finance.testnet", "underlying_token_id": "wbtc.omomo-finance.testnet", "controller_account_id": "controller.omomo-finance.testnet, "initial_exchange_rate": "10000000000", "interest_rate_model": {"kink": "8000000000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100000000"}}'
near deploy daurora.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "omomo-finance.testnet", "underlying_token_id": "aurora.omomo-finance.testnet", "controller_account_id": "controller.omomo-finance.testnet, "initial_exchange_rate": "10000000000", "interest_rate_model": {"kink": "8000000000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100000000"}}'
near deploy dusdt.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "omomo-finance.testnet", "underlying_token_id": "usdt.omomo-finance.testnet", "controller_account_id": "controller.omomo-finance.testnet, "initial_exchange_rate": "10000000000", "interest_rate_model": {"kink": "8000000000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100000000"}}'
near deploy dusdc.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "omomo-finance.testnet", "underlying_token_id": "usdc.omomo-finance.testnet", "controller_account_id": "controller.omomo-finance.testnet, "initial_exchange_rate": "10000000000", "interest_rate_model": {"kink": "8000000000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100000000"}}'
near deploy ddai.omomo-finance.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "omomo-finance.testnet", "underlying_token_id": "dai.omomo-finance.testnet", "controller_account_id": "controller.omomo-finance.testnet, "initial_exchange_rate": "10000000000", "interest_rate_model": {"kink": "8000000000", "multiplier_per_block": "1", "base_rate_per_block": "1", "jump_multiplier_per_block": "2", "reserve_factor": "100000000"}}'

near call controller.omomo-finance.testnet add_market '{"asset_id": "wrap.testnet", "dtoken": "dwnear.omomo-finance.testnet", "ticker_id": "wnear"}' --accountId omomo.near
near call controller.omomo-finance.testnet add_market '{"asset_id": "weth.omomo-finance.testnet", "dtoken": "dweth.omomo-finance.testnet", "ticker_id": "weth"}' --accountId omomo.near
near call controller.omomo-finance.testnet add_market '{"asset_id": "stnear.omomo-finance.testnet", "dtoken": "dstnear.omomo-finance.testnet", "ticker_id": "stnear"}' --accountId omomo.near
near call controller.omomo-finance.testnet add_market '{"asset_id": "wbtc.omomo-finance.testnet", "dtoken": "dwbtc.omomo-finance.testnet", "ticker_id": "wbtc"}' --accountId omomo.near
near call controller.omomo-finance.testnet add_market '{"asset_id": "daurora.omomo-finance.testnet", "dtoken": "aurora.omomo-finance.testnet", "ticker_id": "aurora"}' --accountId omomo.near
near call controller.omomo-finance.testnet add_market '{"asset_id": "dusdt.omomo-finance.testnet", "dtoken": "usdt.omomo-finance.testnet", "ticker_id": "usdt"}' --accountId omomo.near
near call controller.omomo-finance.testnet add_market '{"asset_id": "dusdc.omomo-finance.testnet", "dtoken": "usdc.omomo-finance.testnet", "ticker_id": "usdc"}' --accountId omomo.near
near call controller.omomo-finance.testnet add_market '{"asset_id": "ddai.omomo-finance.testnet", "dtoken": "dai.omomo-finance.testnet", "ticker_id": "dai"}' --accountId omomo.near



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

