# login
# near login

# build & test
build_and_test() {
    ./build.sh && ./test.sh
}

# clean up previous deployment
clean_up_previous_deployment () {
    echo 'y' | near delete weth_market.$1 $1 & 
    echo 'y' | near delete wnear_market.$1 $1 & 
    echo 'y' | near delete usdt_market.$1 $1 & 
    echo 'y' | near delete usdc_market.$1 $1 &

    # TODO unify naming
    echo 'y' | near delete $CONTROLLER_ACCOUNT.$1 $1 &
    wait
}

# create underlying tokens and markets
create_markets() {
    near create-account weth_market.$1 --masterAccount $1 --initialBalance 7 &
    near create-account wnear_market.$1 --masterAccount $1 --initialBalance 7 &
    near create-account usdt_market.$1 --masterAccount $1 --initialBalance 7 &
    near create-account usdc_market.$1 --masterAccount $1 --initialBalance 7 &
    wait
}

# create controller
create_controller() {
    near create-account $CONTROLLER_ACCOUNT.$1 --masterAccount $1 --initialBalance 10 &
    wait
}

# deploy markets
deploy_markets(){
    near deploy weth_market.$1 \
        --wasmFile  ./contracts/target/wasm32-unknown-unknown/release/market.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
            "owner_id":"'$1'",
            "underlying_token_id":"'$ETH_TOKEN'",
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
        --wasmFile  ./contracts/target/wasm32-unknown-unknown/release/market.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
            "owner_id":"'$1'",
            "underlying_token_id":"'$NEAR_TOKEN'",
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
        --wasmFile  ./contracts/target/wasm32-unknown-unknown/release/market.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
        "owner_id":"'$1'",
        "underlying_token_id":"'$USDT_TOKEN'",
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
        --wasmFile  ./contracts/target/wasm32-unknown-unknown/release/market.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
        "owner_id":"'$1'",
        "underlying_token_id":"'$USDC_TOKEN'",
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

redeploy_markets(){
    echo 'y' | near deploy weth_market.$1 \
        --wasmFile  ./contracts/target/wasm32-unknown-unknown/release/market.wasm \
        --initFunction 'migrate' \
        --initArgs '{}' &
    echo 'y' | near deploy wnear_market.$1 \
        --wasmFile  ./contracts/target/wasm32-unknown-unknown/release/market.wasm \
        --initFunction 'migrate' \
        --initArgs '{}' &
    echo 'y' | near deploy usdt_market.$1 \
        --wasmFile  ./contracts/target/wasm32-unknown-unknown/release/market.wasm \
        --initFunction 'migrate' \
        --initArgs '{}' &
    echo 'y' | near deploy usdc_market.$1 \
        --wasmFile  ./contracts/target/wasm32-unknown-unknown/release/market.wasm \
        --initFunction 'migrate' \
        --initArgs '{}' &
    
    wait
}

# deploy controller
deploy_controller(){
    near deploy $CONTROLLER_ACCOUNT.$1 \
        --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm \
        --initFunction 'new_with_config' \
        --initArgs '{
            "owner_id":"'$1'",
            "oracle_account_id":"'$ORACLE_ACCOUNT'"
        }' &

    wait
}

redeploy_controller(){
    echo 'y' | near deploy $CONTROLLER_ACCOUNT.$1 \
        --wasmFile  ./contracts/target/wasm32-unknown-unknown/release/controller.wasm \
        --initFunction 'migrate' \
        --initArgs '{}' &

    wait
}

# create account on underlyings for dtokens
create_account_on_underlyings_for_dtokens(){
    near call $ETH_TOKEN storage_deposit '{"account_id": "weth_market.'$1'"}' --accountId $1 --amount 0.25 &
    near call $NEAR_TOKEN storage_deposit '{"account_id": "wnear_market.'$1'"}' --accountId $1 --amount 0.25 &
    near call $USDT_TOKEN storage_deposit '{"account_id": "usdt_market.'$1'"}' --accountId $1 --amount 0.25 &
    near call $USDC_TOKEN storage_deposit '{"account_id": "usdc_market.'$1'"}' --accountId $1 --amount 0.25 &

    wait
}

# register markets
register_markets_on_controller(){
    near call $CONTROLLER_ACCOUNT.$1 add_market '{
            "asset_id": "'$ETH_TOKEN'",
            "dtoken": "weth_market.'$1'",
            "ticker_id": "nWETH",
            "ltv": "0.4",
            "lth": "0.8"
        }' --accountId $1 &
    near call $CONTROLLER_ACCOUNT.$1 add_market '{
            "asset_id": "'$NEAR_TOKEN'",
            "dtoken": "wnear_market.'$1'",
            "ticker_id": "near",
            "ltv": "0.5",
            "lth": "0.8"
        }' --accountId $1 &
    near call $CONTROLLER_ACCOUNT.$1 add_market '{
            "asset_id": "'$USDT_TOKEN'",
            "dtoken": "usdt_market.'$1'",
            "ticker_id": "USDt",
            "ltv": "0.8",
            "lth": "0.9"
        }' --accountId $1 &
    near call $CONTROLLER_ACCOUNT.$1 add_market '{
            "asset_id": "'$USDC_TOKEN'",
            "dtoken": "usdc_market.'$1'",
            "ticker_id": "nUSDC",
            "ltv": "0.8",
            "lth": "0.9"
        }' --accountId $1 &

    wait
}

setup_reserves(){
    # mint reserves
    near call $ETH_TOKEN mint '{
        "account_id": "'$1'",
        "amount": "1000000000000000000000000000"
    }' --accountId $1 &
    near call $NEAR_TOKEN mint '{
        "account_id": "'$1'",
        "amount": "1000000000000000000000000000"
    }' --accountId $1 &
    near call $USDT_TOKEN mint '{
        "account_id": "'$1'",
        "amount": "1000000000000000000000000000"
    }' --accountId $1 &
    near call $USDC_TOKEN mint '{
        "account_id": "'$1'",
        "amount": "1000000000000000000000000000"
    }' --accountId $1 &

    wait

    # transfer reserves
    near call $ETH_TOKEN ft_transfer_call '{
        "receiver_id": "weth_market.'$1'",
        "amount": "1000000000000000000000000000",
        "msg": "\"Reserve\""
    }' --depositYocto 1 --gas 300000000000000 --accountId $1 &
    near call $NEAR_TOKEN ft_transfer_call '{
        "receiver_id": "wnear_market.'$1'",
        "amount": "1000000000000000000000000000",
        "msg": "\"Reserve\""
    }' --depositYocto 1 --gas 300000000000000 --accountId $1 &
    near call $USDT_TOKEN ft_transfer_call '{
        "receiver_id": "usdt_market.'$1'",
        "amount": "1000000000000000000000000000",
        "msg": "\"Reserve\""
    }' --depositYocto 1 --gas 300000000000000 --accountId $1 &
    near call $USDC_TOKEN ft_transfer_call '{
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

ROOT_ACCOUNT=v1.omomo-finance.testnet
CONTROLLER_ACCOUNT=controller
ORACLE_ACCOUNT=oracle.omomo-finance.testnet
ETH_TOKEN=eth.fakes.testnet
NEAR_TOKEN=wrap.testnet
USDT_TOKEN=usdt.fakes.testnet
USDC_TOKEN=usdc.fakes.testnet

build_and_test
# clean_up_previous_deployment $ROOT_ACCOUNT
# create_markets $ROOT_ACCOUNT &
# create_controller $ROOT_ACCOUNT &
wait

# deploy_markets $ROOT_ACCOUNT &
redeploy_markets $ROOT_ACCOUNT &
# deploy_controller $ROOT_ACCOUNT &
redeploy_controller $ROOT_ACCOUNT &
wait

# create_account_on_underlyings_for_dtokens $ROOT_ACCOUNT
# register_markets_on_controller $ROOT_ACCOUNT &
# setup_reserves $ROOT_ACCOUNT &
wait

# configure_acl $ROOT_ACCOUNT &
wait

# view status
near view $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT view_markets '{}' --accountId $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT
near view $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT view_prices '{ "dtokens": ["wnear_market.'$ROOT_ACCOUNT'", "weth_market.'$ROOT_ACCOUNT'", "usdt_market.'$ROOT_ACCOUNT'", "usdc_market.'$ROOT_ACCOUNT'"] }' --accountId $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT

# view balances
near view $ETH_TOKEN ft_balance_of '{"account_id": "weth_market.'$ROOT_ACCOUNT'"}'
near view $NEAR_TOKEN ft_balance_of '{"account_id": "wnear_market.'$ROOT_ACCOUNT'"}'
near view $USDT_TOKEN ft_balance_of '{"account_id": "usdt_market.'$ROOT_ACCOUNT'"}'
near view $USDC_TOKEN ft_balance_of '{"account_id": "usdc_market.'$ROOT_ACCOUNT'"}'
