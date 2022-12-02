# login
# near login

# build & test
build_and_test() {
    ./build.sh && ./test.sh
}

# clean up previous deployment
clean_up_previous_deployment () {
    echo 'y' | near delete weth.$1 $1 &
    echo 'y' | near delete weth_market.$1 $1 & 

    echo 'y' | near delete wnear.$1 $1 &
    echo 'y' | near delete wnear_market.$1 $1 & 

    echo 'y' | near delete usdt.$1 $1 &
    echo 'y' | near delete usdt_market.$1 $1 & 

    echo 'y' | near delete usdc.$1 $1 &
    echo 'y' | near delete usdc_market.$1 $1 &

    # TODO unify naming
    echo 'y' | near delete $CONTROLLER_ACCOUNT.$1 $1 &
    wait
}

# create underlying tokens and markets
create_underlying_tokens_and_markets() {
    near create-account weth.$1 --masterAccount $1 --initialBalance 3 &
    near create-account weth_market.$1 --masterAccount $1 --initialBalance 7 &

    near create-account wnear.$1 --masterAccount $1 --initialBalance 3 &
    near create-account wnear_market.$1 --masterAccount $1 --initialBalance 7 &

    near create-account usdt.$1 --masterAccount $1 --initialBalance 3 &
    near create-account usdt_market.$1 --masterAccount $1 --initialBalance 7 &

    near create-account usdc.$1 --masterAccount $1 --initialBalance 3 &
    near create-account usdc_market.$1 --masterAccount $1 --initialBalance 7 &
    wait
}

# create controller
create_controller() {
    near create-account $CONTROLLER_ACCOUNT.$1 --masterAccount $1 --initialBalance 10 &
    wait
}

# deploy underlyings
deploy_underlyings() {
    near deploy weth.$1 \
        --wasmFile ./res/test_utoken.wasm \
        --initFunction 'new_default_meta' \
        --initArgs '{
            "owner_id": "'$1'",
            "name": "Wrapped Ethereum",
            "symbol": "WETH",
            "total_supply": "1000000000000000000000000000"
        }' &
    near deploy wnear.$1 \
        --wasmFile ./res/test_utoken.wasm \
        --initFunction 'new_default_meta' \
        --initArgs '{
            "owner_id": "'$1'",
            "name": "Wrapped Near",
            "symbol": "WNEAR",
            "total_supply": "1000000000000000000000000000"
        }' &
    near deploy usdt.$1 \
        --wasmFile ./res/test_utoken.wasm \
        --initFunction 'new_default_meta' \
        --initArgs '{
            "owner_id": "'$1'",
            "name": "Tether",
            "symbol": "USDT",
            "total_supply": "1000000000000000000000000000"
        }' &
    near deploy usdc.$1 \
        --wasmFile ./res/test_utoken.wasm \
        --initFunction 'new_default_meta' \
        --initArgs '{
            "owner_id": "'$1'",
            "name": "USD Coin",
            "symbol": "USDC",
            "total_supply": "1000000000000000000000000000"
        }' &

    wait
}

# deploy markets
deploy_markets(){
    near deploy weth_market.$1 \
        --wasmFile ./res/dtoken.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
            "owner_id":"'$1'",
            "underlying_token_id":"weth.'$1'",
            "controller_account_id":"'$CONTROLLER_ACCOUNT'.'$1'",
            "initial_exchange_rate":"1000000000000000000000000",
            "interest_rate_model":{
                "kink":"650000000000000000000000",
                "multiplier_per_block":"3044140030441400",
                "base_rate_per_block":"0",
                "jump_multiplier_per_block":"38051750380517500",
                "reserve_factor":"10000000000000000000000"
            }
        }' &
    near deploy wnear_market.$1 \
        --wasmFile ./res/dtoken.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
            "owner_id":"'$1'",
            "underlying_token_id":"wnear.'$1'",
            "controller_account_id":"'$CONTROLLER_ACCOUNT'.'$1'",
            "initial_exchange_rate":"1000000000000000000000000",
            "interest_rate_model":{
                "kink":"650000000000000000000000",
                "multiplier_per_block":"3044140030441400",
                "base_rate_per_block":"0",
                "jump_multiplier_per_block":"38051750380517500",
                "reserve_factor":"10000000000000000000000"
            }
        }' &
    near deploy usdt_market.$1 \
        --wasmFile ./res/dtoken.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
        "owner_id":"'$1'",
        "underlying_token_id":"usdt.'$1'",
        "controller_account_id":"'$CONTROLLER_ACCOUNT'.'$1'",
            "initial_exchange_rate":"1000000000000000000000000",
            "interest_rate_model":{
            "kink":"800000000000000000000000",
            "multiplier_per_block":"1522070015220700",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"28538812785388100",
            "reserve_factor":"10000000000000000000000"
            }
        }' &
    near deploy usdc_market.$1 \
        --wasmFile ./res/dtoken.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
        "owner_id":"'$1'",
        "underlying_token_id":"usdc.'$1'",
        "controller_account_id":"'$CONTROLLER_ACCOUNT'.'$1'",
            "initial_exchange_rate":"1000000000000000000000000",
            "interest_rate_model":{
            "kink":"800000000000000000000000",
            "multiplier_per_block":"1522070015220700",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"28538812785388100",
            "reserve_factor":"10000000000000000000000"
            }
        }' &
    
    wait
}

# deploy controller
deploy_controller(){
    near deploy $CONTROLLER_ACCOUNT.$1 \
        --wasmFile ./res/controller.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
            "owner_id":"'$1'",
            "oracle_account_id":"oracle.'$1'"
        }' &

    wait
}

# create account on underlyings for dtokens
create_account_on_underlyings_for_dtokens(){
    near call weth.$1 storage_deposit '{"account_id": "weth_market.'$1'"}' --accountId $1 --amount 0.25 &
    near call wnear.$1 storage_deposit '{"account_id": "wnear_market.'$1'"}' --accountId $1 --amount 0.25 &
    near call usdt.$1 storage_deposit '{"account_id": "usdt_market.'$1'"}' --accountId $1 --amount 0.25 &
    near call usdc.$1 storage_deposit '{"account_id": "usdc_market.'$1'"}' --accountId $1 --amount 0.25 &

    wait
}

# register markets
register_markets_on_controller(){
    near call $CONTROLLER_ACCOUNT.$1 add_market '{
            "asset_id": "weth.'$1'",
            "dtoken": "weth_market.'$1'",
            "ticker_id": "nWETH",
            "ltv": "0.4",
            "lth": "0.8"
        }' --accountId $1 &
    near call $CONTROLLER_ACCOUNT.$1 add_market '{
            "asset_id": "wnear.'$1'",
            "dtoken": "wnear_market.'$1'",
            "ticker_id": "near",
            "ltv": "0.5",
            "lth": "0.8"
        }' --accountId $1 &
    near call $CONTROLLER_ACCOUNT.$1 add_market '{
            "asset_id": "usdt.'$1'",
            "dtoken": "usdt_market.'$1'",
            "ticker_id": "USDt",
            "ltv": "0.8",
            "lth": "0.9"
        }' --accountId $1 &
    near call $CONTROLLER_ACCOUNT.$1 add_market '{
            "asset_id": "usdc.'$1'",
            "dtoken": "usdc_market.'$1'",
            "ticker_id": "nUSDC",
            "ltv": "0.8",
            "lth": "0.9"
        }' --accountId $1 &

    wait
}

setup_reserves(){
    # mint reserves
    near call weth.$1 mint '{
        "account_id": "'$1'",
        "amount": "1000000000000000000000000000"
    }' --accountId $1 &
    near call wnear.$1 mint '{
        "account_id": "'$1'",
        "amount": "1000000000000000000000000000"
    }' --accountId $1 &
    near call usdt.$1 mint '{
        "account_id": "'$1'",
        "amount": "1000000000000000000000000000"
    }' --accountId $1 &
    near call usdc.$1 mint '{
        "account_id": "'$1'",
        "amount": "1000000000000000000000000000"
    }' --accountId $1 &

    wait

    # transfer reserves
    near call weth.$1 ft_transfer_call '{
        "receiver_id": "weth_market.'$1'",
        "amount": "1000000000000000000000000000",
        "msg": "\"Reserve\""
    }' --depositYocto 1 --gas 300000000000000 --accountId $1 &
    near call wnear.$1 ft_transfer_call '{
        "receiver_id": "wnear_market.'$1'",
        "amount": "1000000000000000000000000000",
        "msg": "\"Reserve\""
    }' --depositYocto 1 --gas 300000000000000 --accountId $1 &
    near call usdt.$1 ft_transfer_call '{
        "receiver_id": "usdt_market.'$1'",
        "amount": "1000000000000000000000000000",
        "msg": "\"Reserve\""
    }' --depositYocto 1 --gas 300000000000000 --accountId $1 &
    near call usdc.$1 ft_transfer_call '{
        "receiver_id": "usdc_market.'$1'",
        "amount": "1000000000000000000000000000",
        "msg": "\"Reserve\""
    }' --depositYocto 1 --gas 300000000000000 --accountId $1 &

    wait
}

configure_acl() {
    # set shared admin as admin for dtokens
    near call weth_market.$1 set_admin '{"account": "shared_admin.testnet"}' --gas 300000000000000 --accountId $1 &
    near call wnear_market.$1 set_admin '{"account": "shared_admin.testnet"}' --gas 300000000000000 --accountId $1 &
    near call usdt_market.$1 set_admin '{"account": "shared_admin.testnet"}' --gas 300000000000000 --accountId $1 &
    near call usdc_market.$1 set_admin '{"account": "shared_admin.testnet"}' --gas 300000000000000 --accountId $1 &

    wait
}

ROOT_ACCOUNT=nearlend.testnet
CONTROLLER_ACCOUNT=controller_beta

build_and_test
clean_up_previous_deployment $ROOT_ACCOUNT
create_underlying_tokens_and_markets $ROOT_ACCOUNT &
create_controller $ROOT_ACCOUNT &
wait

deploy_underlyings $ROOT_ACCOUNT &
deploy_markets $ROOT_ACCOUNT &
deploy_controller $ROOT_ACCOUNT &
wait

create_account_on_underlyings_for_dtokens $ROOT_ACCOUNT
register_markets_on_controller $ROOT_ACCOUNT &
setup_reserves $ROOT_ACCOUNT &
wait

# configure_acl $ROOT_ACCOUNT &
# wait

# view status
near view $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT view_markets '{}' --accountId $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT
near view $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT view_prices '{ "dtokens": ["wnear_market.'$ROOT_ACCOUNT'", "weth_market.'$ROOT_ACCOUNT'", "usdt_market.'$ROOT_ACCOUNT'", "usdc_market.'$ROOT_ACCOUNT'"] }' --accountId $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT

# view balances
near view weth.$ROOT_ACCOUNT ft_balance_of '{"account_id": "weth_market.'$ROOT_ACCOUNT'"}'
near view wnear.$ROOT_ACCOUNT ft_balance_of '{"account_id": "wnear_market.'$ROOT_ACCOUNT'"}'
near view usdt.$ROOT_ACCOUNT ft_balance_of '{"account_id": "usdt_market.'$ROOT_ACCOUNT'"}'
near view usdc.$ROOT_ACCOUNT ft_balance_of '{"account_id": "usdc_market.'$ROOT_ACCOUNT'"}'
