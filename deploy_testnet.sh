# login
# near login

# build & test
./build.sh && ./test.sh

# clean up previous deployment
near delete weth.omomo-finance.testnet omomo-finance.testnet 
near delete dweth.omomo-finance.testnet omomo-finance.testnet 

# near delete wnear.omomo-finance.testnet omomo-finance.testnet 
near delete dwnear.omomo-finance.testnet omomo-finance.testnet 

near delete usdt.omomo-finance.testnet omomo-finance.testnet 
near delete dusdt.omomo-finance.testnet omomo-finance.testnet 

near delete usdc.omomo-finance.testnet omomo-finance.testnet 
near delete dusdc.omomo-finance.testnet omomo-finance.testnet 

near delete controller.omomo-finance.testnet omomo-finance.testnet 


# create underlying tokens and markets
near create-account weth.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 3 
near create-account dweth.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 5 

# near create-account wnear.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 3 
near create-account dwnear.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 5 

near create-account usdt.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 3 
near create-account dusdt.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 5 

near create-account usdc.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 3 
near create-account dusdc.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 5 

# create controller
near create-account controller.omomo-finance.testnet --masterAccount omomo-finance.testnet --initialBalance 10 


# deploy underlyings
near deploy weth.omomo-finance.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm \
    --initFunction 'new_default_meta' \
    --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Wrapped Ethereum", "symbol": "WETH", "total_supply": "1000000000000000000000000000"}'
# near deploy wnear.omomo-finance.testnet \
#     --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm \
#     --initFunction 'new_default_meta' \
#     --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Wrapped Near", "symbol": "WNEAR", "total_supply": "1000000000000000000000000000"}'
near deploy usdt.omomo-finance.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm \
    --initFunction 'new_default_meta' \
    --initArgs '{"owner_id": "omomo-finance.testnet", "name": "Tether", "symbol": "USDT", "total_supply": "1000000000000000000000000000"}'
near deploy usdc.omomo-finance.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm \
    --initFunction 'new_default_meta' \
    --initArgs '{"owner_id": "omomo-finance.testnet", "name": "USD Coin", "symbol": "USDC", "total_supply": "1000000000000000000000000000"}'


# deploy markets
near deploy dweth.omomo-finance.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
        "owner_id":"omomo-finance.testnet",
        "underlying_token_id":"weth.omomo-finance.testnet",
        "controller_account_id":"controller.omomo-finance.testnet",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
            "kink":"650000000000000000000000",
            "multiplier_per_block":"62800000000000000",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"76100000000000000",
            "reserve_factor":"10000000000000000000000"
        }
    }'
near deploy dwnear.omomo-finance.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
        "owner_id":"omomo-finance.testnet",
        "underlying_token_id":"wrap.testnet",
        "controller_account_id":"controller.omomo-finance.testnet",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
            "kink":"650000000000000000000000",
            "multiplier_per_block":"62800000000000000",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"76100000000000000",
            "reserve_factor":"10000000000000000000000"
        }
    }'
near deploy dusdt.omomo-finance.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
       "owner_id":"omomo-finance.testnet",
       "underlying_token_id":"usdt.omomo-finance.testnet",
       "controller_account_id":"controller.omomo-finance.testnet",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
           "kink":"800000000000000000000000",
           "multiplier_per_block":"68500000000000000",
           "base_rate_per_block":"0",
           "jump_multiplier_per_block":"66600000000000000",
           "reserve_factor":"10000000000000000000000"
        }
    }'
near deploy dusdc.omomo-finance.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
       "owner_id":"omomo-finance.testnet",
       "underlying_token_id":"usdc.omomo-finance.testnet",
       "controller_account_id":"controller.omomo-finance.testnet",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
           "kink":"800000000000000000000000",
           "multiplier_per_block":"68500000000000000",
           "base_rate_per_block":"0",
           "jump_multiplier_per_block":"66600000000000000",
           "reserve_factor":"10000000000000000000000"
        }
    }'

# deploy controller
near deploy controller.omomo-finance.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
        "owner_id":"omomo-finance.testnet",
        "oracle_account_id":"oracle.omomo-finance.testnet"
    }'


# fund dweth.omomo-finance.testnet
near call weth.omomo-finance.testnet storage_deposit '{"account_id": "dweth.omomo-finance.testnet"}' --accountId omomo-finance.testnet --amount 0.25
near call wrap.testnet storage_deposit '{"account_id": "dwnear.omomo-finance.testnet"}' --accountId omomo-finance.testnet --amount 0.25
near call usdt.omomo-finance.testnet storage_deposit '{"account_id": "dusdt.omomo-finance.testnet"}' --accountId omomo-finance.testnet --amount 0.25
near call usdc.omomo-finance.testnet storage_deposit '{"account_id": "dusdc.omomo-finance.testnet"}' --accountId omomo-finance.testnet --amount 0.25

# near call weth.omomo-finance.testnet mint '{"account_id": "dweth.omomo-finance.testnet", "amount": "1"}' --accountId omomo-finance.testnet 
# near call wnear.omomo-finance.testnet mint '{"account_id": "dwnear.omomo-finance.testnet", "amount": "1"}' --accountId omomo-finance.testnet 
# near call usdt.omomo-finance.testnet mint '{"account_id": "dusdt.omomo-finance.testnet", "amount": "1"}' --accountId omomo-finance.testnet 
# near call usdc.omomo-finance.testnet mint '{"account_id": "dusdc.omomo-finance.testnet", "amount": "1"}' --accountId omomo-finance.testnet 

# register market
near call controller.omomo-finance.testnet add_market '{"asset_id": "weth.omomo-finance.testnet", "dtoken": "dweth.omomo-finance.testnet", "ticker_id": "weth"}' --accountId omomo-finance.testnet 
near call controller.omomo-finance.testnet add_market '{"asset_id": "wrap.testnet", "dtoken": "dwnear.omomo-finance.testnet", "ticker_id": "wnear"}' --accountId omomo-finance.testnet 
near call controller.omomo-finance.testnet add_market '{"asset_id": "usdt.omomo-finance.testnet", "dtoken": "dusdt.omomo-finance.testnet", "ticker_id": "usdt"}' --accountId omomo-finance.testnet 
near call controller.omomo-finance.testnet add_market '{"asset_id": "usdc.omomo-finance.testnet", "dtoken": "dusdc.omomo-finance.testnet", "ticker_id": "usdc"}' --accountId omomo-finance.testnet 

near view controller.omomo-finance.testnet view_markets '{}' --accountId controller.omomo-finance.testnet

near view controller.omomo-finance.testnet view_prices '{ "dtokens": ["dwnear.omomo-finance.testnet", "dweth.omomo-finance.testnet", "dusdt.omomo-finance.testnet", "dusdc.omomo-finance.testnet"] }' --accountId controller.omomo-finance.testnet 


near call weth.omomo-finance.testnet mint '{"account_id": "omomo-finance.testnet", "amount": "1000000000000000000000000000"}' --accountId omomo-finance.testnet
# near call wnear.omomo-finance.testnet mint '{"account_id": "omomo-finance.testnet", "amount": "1000000000000000000000000000"}' --accountId omomo-finance.testnet
near call usdt.omomo-finance.testnet mint '{"account_id": "omomo-finance.testnet", "amount": "1000000000000000000000000000"}' --accountId omomo-finance.testnet
near call usdc.omomo-finance.testnet mint '{"account_id": "omomo-finance.testnet", "amount": "1000000000000000000000000000"}' --accountId omomo-finance.testnet

near call weth.omomo-finance.testnet ft_transfer_call '{"receiver_id": "dweth.omomo-finance.testnet", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId omomo-finance.testnet
# near call wnear.omomo-finance.testnet ft_transfer_call '{"receiver_id": "dwnear.omomo-finance.testnet", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId omomo-finance.testnet
near call usdt.omomo-finance.testnet ft_transfer_call '{"receiver_id": "dusdt.omomo-finance.testnet", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId omomo-finance.testnet
near call usdc.omomo-finance.testnet ft_transfer_call '{"receiver_id": "dusdc.omomo-finance.testnet", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId omomo-finance.testnet

near view weth.omomo-finance.testnet ft_balance_of '{"account_id": "dweth.omomo-finance.testnet"}'
near view wrap.testnet ft_balance_of '{"account_id": "dwnear.omomo-finance.testnet"}'
near view usdt.omomo-finance.testnet ft_balance_of '{"account_id": "dusdt.omomo-finance.testnet"}'
near view usdc.omomo-finance.testnet ft_balance_of '{"account_id": "dusdc.omomo-finance.testnet"}'