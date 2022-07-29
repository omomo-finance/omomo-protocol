# login
# near login

# build & test
./build.sh && ./test.sh


# clean up previous deployment
near delete weth.qa.nearlend.testnet qa.nearlend.testnet 
near delete dweth.qa.nearlend.testnet qa.nearlend.testnet 

near delete wnear.qa.nearlend.testnet qa.nearlend.testnet 
near delete dwnear.qa.nearlend.testnet qa.nearlend.testnet 

near delete usdt.qa.nearlend.testnet qa.nearlend.testnet 
near delete dusdt.qa.nearlend.testnet qa.nearlend.testnet 

near delete usdc.qa.nearlend.testnet qa.nearlend.testnet 
near delete dusdc.qa.nearlend.testnet qa.nearlend.testnet 

near delete controller.qa.nearlend.testnet qa.nearlend.testnet 


# create underlying tokens and markets
near create-account weth.qa.nearlend.testnet --masterAccount qa.nearlend.testnet --initialBalance 3 
near create-account dweth.qa.nearlend.testnet --masterAccount qa.nearlend.testnet --initialBalance 7 

near create-account wnear.qa.nearlend.testnet --masterAccount qa.nearlend.testnet --initialBalance 3 
near create-account dwnear.qa.nearlend.testnet --masterAccount qa.nearlend.testnet --initialBalance 7 

near create-account usdt.qa.nearlend.testnet --masterAccount qa.nearlend.testnet --initialBalance 3 
near create-account dusdt.qa.nearlend.testnet --masterAccount qa.nearlend.testnet --initialBalance 7 

near create-account usdc.qa.nearlend.testnet --masterAccount qa.nearlend.testnet --initialBalance 3 
near create-account dusdc.qa.nearlend.testnet --masterAccount qa.nearlend.testnet --initialBalance 7 


# create controller
near create-account controller.qa.nearlend.testnet --masterAccount qa.nearlend.testnet --initialBalance 10 


# deploy underlyings
near deploy weth.qa.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm \
    --initFunction 'new_default_meta' \
    --initArgs '{"owner_id": "qa.nearlend.testnet", "name": "Wrapped Ethereum", "symbol": "WETH", "total_supply": "1000000000000000000000000000"}'
near deploy wnear.qa.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm \
    --initFunction 'new_default_meta' \
    --initArgs '{"owner_id": "qa.nearlend.testnet", "name": "Wrapped Near", "symbol": "WNEAR", "total_supply": "1000000000000000000000000000"}'
near deploy usdt.qa.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm \
    --initFunction 'new_default_meta' \
    --initArgs '{"owner_id": "qa.nearlend.testnet", "name": "Tether", "symbol": "USDT", "total_supply": "1000000000000000000000000000"}'
near deploy usdc.qa.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm \
    --initFunction 'new_default_meta' \
    --initArgs '{"owner_id": "qa.nearlend.testnet", "name": "USD Coin", "symbol": "USDC", "total_supply": "1000000000000000000000000000"}'


# deploy markets
near deploy dweth.qa.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
        "owner_id":"qa.nearlend.testnet",
        "underlying_token_id":"weth.qa.nearlend.testnet",
        "controller_account_id":"controller.qa.nearlend.testnet",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
            "kink":"650000000000000000000000",
            "multiplier_per_block":"3044140030441400",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"38051750380517500",
            "reserve_factor":"10000000000000000000000"
        }
    }'
near deploy dwnear.qa.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
        "owner_id":"qa.nearlend.testnet",
        "underlying_token_id":"wnear.qa.nearlend.testnet",
        "controller_account_id":"controller.qa.nearlend.testnet",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
            "kink":"650000000000000000000000",
            "multiplier_per_block":"3044140030441400",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"38051750380517500",
            "reserve_factor":"10000000000000000000000"
        }
    }'
near deploy dusdt.qa.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
       "owner_id":"qa.nearlend.testnet",
       "underlying_token_id":"usdt.qa.nearlend.testnet",
       "controller_account_id":"controller.qa.nearlend.testnet",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
           "kink":"800000000000000000000000",
           "multiplier_per_block":"1522070015220700",
           "base_rate_per_block":"0",
           "jump_multiplier_per_block":"28538812785388100",
           "reserve_factor":"10000000000000000000000"
        }
    }'
near deploy dusdc.qa.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
       "owner_id":"qa.nearlend.testnet",
       "underlying_token_id":"usdc.qa.nearlend.testnet",
       "controller_account_id":"controller.qa.nearlend.testnet",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
           "kink":"800000000000000000000000",
           "multiplier_per_block":"1522070015220700",
           "base_rate_per_block":"0",
           "jump_multiplier_per_block":"28538812785388100",
           "reserve_factor":"10000000000000000000000"
        }
    }'

# deploy controller
near deploy controller.qa.nearlend.testnet \
    --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{
        "owner_id":"qa.nearlend.testnet",
        "oracle_account_id":"oracle.qa.nearlend.testnet"
    }'


# fund dweth.qa.nearlend.testnet
near call weth.qa.nearlend.testnet storage_deposit '{"account_id": "dweth.qa.nearlend.testnet"}' --accountId qa.nearlend.testnet --amount 0.25
near call wnear.qa.nearlend.testnet storage_deposit '{"account_id": "dwnear.qa.nearlend.testnet"}' --accountId qa.nearlend.testnet --amount 0.25
near call usdt.qa.nearlend.testnet storage_deposit '{"account_id": "dusdt.qa.nearlend.testnet"}' --accountId qa.nearlend.testnet --amount 0.25
near call usdc.qa.nearlend.testnet storage_deposit '{"account_id": "dusdc.qa.nearlend.testnet"}' --accountId qa.nearlend.testnet --amount 0.25

# near call weth.qa.nearlend.testnet mint '{"account_id": "dweth.qa.nearlend.testnet", "amount": "1"}' --accountId qa.nearlend.testnet 
# near call wnear.qa.nearlend.testnet mint '{"account_id": "dwnear.qa.nearlend.testnet", "amount": "1"}' --accountId qa.nearlend.testnet 
# near call usdt.qa.nearlend.testnet mint '{"account_id": "dusdt.qa.nearlend.testnet", "amount": "1"}' --accountId qa.nearlend.testnet 
# near call usdc.qa.nearlend.testnet mint '{"account_id": "dusdc.qa.nearlend.testnet", "amount": "1"}' --accountId qa.nearlend.testnet 

# register market
near call controller.qa.nearlend.testnet add_market '{"asset_id": "weth.qa.nearlend.testnet", "dtoken": "dweth.qa.nearlend.testnet", "ticker_id": "weth"}' --accountId qa.nearlend.testnet 
near call controller.qa.nearlend.testnet add_market '{"asset_id": "wnear.qa.nearlend.testnet", "dtoken": "dwnear.qa.nearlend.testnet", "ticker_id": "wnear"}' --accountId qa.nearlend.testnet 
near call controller.qa.nearlend.testnet add_market '{"asset_id": "usdt.qa.nearlend.testnet", "dtoken": "dusdt.qa.nearlend.testnet", "ticker_id": "usdt"}' --accountId qa.nearlend.testnet 
near call controller.qa.nearlend.testnet add_market '{"asset_id": "usdc.qa.nearlend.testnet", "dtoken": "dusdc.qa.nearlend.testnet", "ticker_id": "usdc"}' --accountId qa.nearlend.testnet 

near view controller.qa.nearlend.testnet view_markets '{}' --accountId controller.qa.nearlend.testnet

near view controller.qa.nearlend.testnet view_prices '{ "dtokens": ["dwnear.qa.nearlend.testnet", "dweth.qa.nearlend.testnet", "dusdt.qa.nearlend.testnet", "dusdc.qa.nearlend.testnet"] }' --accountId controller.qa.nearlend.testnet 


near call weth.qa.nearlend.testnet mint '{"account_id": "qa.nearlend.testnet", "amount": "1000000000000000000000000000"}' --accountId qa.nearlend.testnet
near call wnear.qa.nearlend.testnet mint '{"account_id": "qa.nearlend.testnet", "amount": "1000000000000000000000000000"}' --accountId qa.nearlend.testnet
near call usdt.qa.nearlend.testnet mint '{"account_id": "qa.nearlend.testnet", "amount": "1000000000000000000000000000"}' --accountId qa.nearlend.testnet
near call usdc.qa.nearlend.testnet mint '{"account_id": "qa.nearlend.testnet", "amount": "1000000000000000000000000000"}' --accountId qa.nearlend.testnet

near call weth.qa.nearlend.testnet ft_transfer_call '{"receiver_id": "dweth.qa.nearlend.testnet", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId qa.nearlend.testnet
near call wnear.qa.nearlend.testnet ft_transfer_call '{"receiver_id": "dwnear.qa.nearlend.testnet", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId qa.nearlend.testnet
near call usdt.qa.nearlend.testnet ft_transfer_call '{"receiver_id": "dusdt.qa.nearlend.testnet", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId qa.nearlend.testnet
near call usdc.qa.nearlend.testnet ft_transfer_call '{"receiver_id": "dusdc.qa.nearlend.testnet", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId qa.nearlend.testnet

near view weth.qa.nearlend.testnet ft_balance_of '{"account_id": "dweth.qa.nearlend.testnet"}'
near view wnear.qa.nearlend.testnet ft_balance_of '{"account_id": "dwnear.qa.nearlend.testnet"}'
near view usdt.qa.nearlend.testnet ft_balance_of '{"account_id": "dusdt.qa.nearlend.testnet"}'
near view usdc.qa.nearlend.testnet ft_balance_of '{"account_id": "dusdc.qa.nearlend.testnet"}'

# set shared admin as admin for dtokens
near call dweth.qa.nearlend.testnet set_admin '{"account": "shared_admin.testnet"}' --gas 300000000000000 --accountId qa.nearlend.testnet
near call dwnear.qa.nearlend.testnet set_admin '{"account": "shared_admin.testnet"}' --gas 300000000000000 --accountId qa.nearlend.testnet
near call dusdt.qa.nearlend.testnet set_admin '{"account": "shared_admin.testnet"}' --gas 300000000000000 --accountId qa.nearlend.testnet
near call dusdc.qa.nearlend.testnet set_admin '{"account": "shared_admin.testnet"}' --gas 300000000000000 --accountId qa.nearlend.testnet