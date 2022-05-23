# login
# near login

# build & test
./build.sh && ./test.sh

# ============================
# LAUNCH DEPLOY SCRIPT      ||
# ============================

# Create corresponding accounts
## The oracle contract account
near create-account oracle.omomo.near --masterAccount omomo.near --initialBalance 5 

## The controller contract account
near create-account controller.omomo.near --masterAccount omomo.near --initialBalance 1 

## The OMOMO token account
near create-account token.omomo.near --masterAccount omomo.near --initialBalance 1 

## The market accounts list
near create-account wnear.omomo.near --masterAccount omomo.near --initialBalance 1 
near create-account weth.omomo.near --masterAccount omomo.near --initialBalance 1 
near create-account stnear.omomo.near --masterAccount omomo.near --initialBalance 1 
near create-account wbtc.omomo.near --masterAccount omomo.near --initialBalance 1 
near create-account aurora.omomo.near --masterAccount omomo.near --initialBalance 1 
near create-account usdt.omomo.near --masterAccount omomo.near --initialBalance 1 
near create-account usdc.omomo.near --masterAccount omomo.near --initialBalance 1 
near create-account dai.omomo.near --masterAccount omomo.near --initialBalance 1
near create-account omomo.omomo.near --masterAccount omomo.near --initialBalance 1

# Deploy Controller contract
near deploy controller.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm
  --initFunction 'new_with_config'
  --initArgs '{"owner_id": "omomo.near", "oracle_account_id": "oracle.omomo.near"}' 

# Deploy OMOMO token
near deploy token.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/token.wasm
  --initFunction 'new_default_meta'
  --initArgs '{
    "owner_id": "omomo.near",
    "name": "OMOMO token",
    "symbol": "OMOMO",
    "total_supply": "1000000000000000000000000000000000"}}' 

# Deploy Dtoken contracts
near deploy wnear.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
  --initFunction 'new_with_config'
  --initArgs '{
    "owner_id": "omomo.near",
    "underlying_token_id": "wrap.near",
    "controller_account_id": "controller.omomo.near",
    "initial_exchange_rate": "10000",
    "interest_rate_model": {"kink": "XYZ", "multiplier_per_block": "XYZ", "base_rate_per_block": "XYZ", "jump_multiplier_per_block": "XYZ", "reserve_factor": "XYZ"}}' 

near deploy weth.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
  --initFunction 'new_with_config'
  --initArgs '{
    "owner_id": "omomo.near",
    "underlying_token_id": "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2.factory.bridge.near",
    "controller_account_id": "controller.omomo.near",
    "initial_exchange_rate": "10000",
    "interest_rate_model": {"kink": "XYZ", "multiplier_per_block": "XYZ", "base_rate_per_block": "XYZ", "jump_multiplier_per_block": "XYZ", "reserve_factor": "XYZ"}}' 

near deploy stnear.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
  --initFunction 'new_with_config'
  --initArgs '{
    "owner_id": "omomo.near",
    "underlying_token_id": "meta-pool.near",
    "controller_account_id": "controller.omomo.near",
    "initial_exchange_rate": "10000",
    "interest_rate_model": {"kink": "XYZ", "multiplier_per_block": "XYZ", "base_rate_per_block": "XYZ", "jump_multiplier_per_block": "XYZ", "reserve_factor": "XYZ"}}' 


near deploy wbtc.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
  --initFunction 'new_with_config'
  --initArgs '{
    "owner_id": "omomo.near",
    "underlying_token_id": "2260fac5e5542a773aa44fbcfedf7c193bc2c599.factory.bridge.near",
    "controller_account_id": "controller.omomo.near",
    "initial_exchange_rate": "10000",
    "interest_rate_model": {"kink": "XYZ", "multiplier_per_block": "XYZ", "base_rate_per_block": "XYZ", "jump_multiplier_per_block": "XYZ", "reserve_factor": "XYZ"}}' 

near deploy aurora.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
  --initFunction 'new_with_config'
  --initArgs '{
    "owner_id": "omomo.near",
    "underlying_token_id": "aaaaaa20d9e0e2461697782ef11675f668207961.factory.bridge.near",
    "controller_account_id": "controller.omomo.near",
    "initial_exchange_rate": "10000",
    "interest_rate_model": {"kink": "XYZ", "multiplier_per_block": "XYZ", "base_rate_per_block": "XYZ", "jump_multiplier_per_block": "XYZ", "reserve_factor": "XYZ"}}' 

near deploy usdt.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
  --initFunction 'new_with_config'
  --initArgs '{
    "owner_id": "omomo.near",
    "underlying_token_id": "dac17f958d2ee523a2206206994597c13d831ec7.factory.bridge.near",
    "controller_account_id": "controller.omomo.near",
    "initial_exchange_rate": "10000",
    "interest_rate_model": {"kink": "XYZ", "multiplier_per_block": "XYZ", "base_rate_per_block": "XYZ", "jump_multiplier_per_block": "XYZ", "reserve_factor": "XYZ"}}' 

near deploy usdc.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
  --initFunction 'new_with_config'
  --initArgs '{
    "owner_id": "omomo.near",
    "underlying_token_id": "a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.factory.bridge.near",
    "controller_account_id": "controller.omomo.near",
    "initial_exchange_rate": "10000",
    "interest_rate_model": {"kink": "XYZ", "multiplier_per_block": "XYZ", "base_rate_per_block": "XYZ", "jump_multiplier_per_block": "XYZ", "reserve_factor": "XYZ"}}' 

near deploy dai.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
  --initFunction 'new_with_config'
  --initArgs '{
    "owner_id": "omomo.near",
    "underlying_token_id": "6b175474e89094c44da98b954eedeac495271d0f.factory.bridge.near",
    "controller_account_id": "controller.omomo.near",
    "initial_exchange_rate": "10000",
    "interest_rate_model": {"kink": "XYZ", "multiplier_per_block": "XYZ", "base_rate_per_block": "XYZ", "jump_multiplier_per_block": "XYZ", "reserve_factor": "XYZ"}}' 

near deploy omomo.omomo.near --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
  --initFunction 'new_with_config'
  --initArgs '{
    "owner_id": "omomo.near",
    "underlying_token_id": "token.omomo.near",
    "controller_account_id": "controller.omomo.near",
    "initial_exchange_rate": "10000",
    "interest_rate_model": {"kink": "XYZ", "multiplier_per_block": "XYZ", "base_rate_per_block": "XYZ", "jump_multiplier_per_block": "XYZ", "reserve_factor": "XYZ"}}'


near call controller.omomo.near add_market '{"asset_id": "wrap.near", "dtoken": "wnear.omomo.near", "ticker_id": "weth"}' --accountId omomo.near
near call controller.omomo.near add_market '{"asset_id": "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2.factory.bridge.near", "dtoken": "weth.omomo.near", "ticker_id": "weth"}' --accountId omomo.near
near call controller.omomo.near add_market '{"asset_id": "meta-pool.near", "dtoken": "stnear.omomo.near", "ticker_id": "weth"}' --accountId omomo.near
near call controller.omomo.near add_market '{"asset_id": "2260fac5e5542a773aa44fbcfedf7c193bc2c599.factory.bridge.near", "dtoken": "wbtc.omomo.near", "ticker_id": "weth"}' --accountId omomo.near
near call controller.omomo.near add_market '{"asset_id": "aaaaaa20d9e0e2461697782ef11675f668207961.factory.bridge.near", "dtoken": "aurora.omomo.near", "ticker_id": "weth"}' --accountId omomo.near
near call controller.omomo.near add_market '{"asset_id": "dac17f958d2ee523a2206206994597c13d831ec7.factory.bridge.near", "dtoken": "usdt.omomo.near", "ticker_id": "weth"}' --accountId omomo.near
near call controller.omomo.near add_market '{"asset_id": "a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.factory.bridge.near", "dtoken": "usdc.omomo.near", "ticker_id": "weth"}' --accountId omomo.near
near call controller.omomo.near add_market '{"asset_id": "6b175474e89094c44da98b954eedeac495271d0f.factory.bridge.near", "dtoken": "dai.omomo.near", "ticker_id": "weth"}' --accountId omomo.near
near call controller.omomo.near add_market '{"asset_id": "token.omomo.near", "dtoken": "omomo.omomo.near", "ticker_id": "weth"}' --accountId omomo.near




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