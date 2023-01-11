# login
#near login

# build & test
mkdir -p res && ./build.sh && ./test.sh

ROOT_ACCOUNT=v1.omomo-finance.testnet
CONTROLLER_ACCOUNT=controller
ORACLE_ACCOUNT=oracle.omomo-finance.testnet
ETH_TOKEN=eth.fakes.testnet
NEAR_TOKEN=wrap.testnet
USDT_TOKEN=usdt.fakes.testnet
USDC_TOKEN=usdc.fakes.testnet
CONTRACT_ADDRESS=leverage.$ROOT_ACCOUNT

# clean up previuos deployment
echo 'y' | near delete ${CONTRACT_ADDRESS} $ROOT_ACCOUNT

# create corresponding accoutns
near create-account ${CONTRACT_ADDRESS} --masterAccount $ROOT_ACCOUNT --initialBalance 10

# redeploy contracts
# --wasmFile ./res/limit_orders.wasm
near deploy ${CONTRACT_ADDRESS} \
  --wasmFile ./res/limit_orders.wasm \
  --initFunction 'new_with_config' \
  --initArgs '{
        "owner_id":"'${CONTRACT_ADDRESS}'",
        "oracle_account_id":"limit_orders_oracle.'$ROOT_ACCOUNT'"
    }'

# register limit orders on tokens
near call $NEAR_TOKEN storage_deposit '{"account_id": "'${CONTRACT_ADDRESS}'"}' --accountId ${CONTRACT_ADDRESS} --amount 0.25 &
near call $USDT_TOKEN storage_deposit '{"account_id": "'${CONTRACT_ADDRESS}'"}' --accountId ${CONTRACT_ADDRESS} --amount 0.25 &
wait

# add supported pairs
near call ${CONTRACT_ADDRESS} add_pair '{
        "pair_data": {
            "sell_ticker_id": "USDt",
            "sell_token": "'$USDT_TOKEN'",
            "sell_token_market": "usdt_market.'$ROOT_ACCOUNT'",
            "buy_ticker_id": "near",
            "buy_token": "'$NEAR_TOKEN'",
            "pool_id": "'$USDT_TOKEN'|'$NEAR_TOKEN'|2000",
            "max_leverage": "25000000000000000000000000"
        }
    }' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} add_pair '{
        "pair_data": {
            "sell_ticker_id": "near",
            "sell_token": "'$NEAR_TOKEN'",
            "sell_token_market": "wnear_market.'$ROOT_ACCOUNT'",
            "buy_ticker_id": "USDt",
            "buy_token": "'$USDT_TOKEN'",
            "pool_id": "'$USDT_TOKEN'|'$NEAR_TOKEN'|2000",
            "max_leverage": "25000000000000000000000000"
        }
    }' --accountId ${CONTRACT_ADDRESS} &


near call ${CONTRACT_ADDRESS} set_max_order_amount '{
    "value": "10000000000000000000000000000"
}' --accountId ${CONTRACT_ADDRESS} &

wait
# near view ${CONTRACT_ADDRESS} view_supported_pairs '{}'

# add mock prices
near call ${CONTRACT_ADDRESS} update_or_insert_price '{
    "token_id":"'$USDT_TOKEN'",
    "price":{
        "ticker_id":"USDt",
        "value":"1.01"
    }
}' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} update_or_insert_price '{
    "token_id":"'$NEAR_TOKEN'",
    "price":{
        "ticker_id":"near",
        "value":"1.83"
    }
}' --accountId ${CONTRACT_ADDRESS} &

wait

near view ${CONTRACT_ADDRESS} view_price '{"token_id":"'$USDT_TOKEN'"}'
near view ${CONTRACT_ADDRESS} view_price '{"token_id":"'$NEAR_TOKEN'"}'

wait

# setup pool
near call dcl.ref-dev.testnet storage_deposit '{"account_id": "'${CONTRACT_ADDRESS}'"}' --accountId nearlend.testnet --amount 1 &

near call ${CONTRACT_ADDRESS} add_token_market '{"token_id": "'$NEAR_TOKEN'", "market_id": "wnear_market.'$ROOT_ACCOUNT'"}' --account_id ${CONTRACT_ADDRESS} &
near call ${CONTRACT_ADDRESS} add_token_market '{"token_id": "'$USDT_TOKEN'", "market_id": "usdt_market.'$ROOT_ACCOUNT'"}' --account_id ${CONTRACT_ADDRESS} &

near call usdt_market.$ROOT_ACCOUNT set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }' --accountId shared_admin.testnet
near view usdt_market.$ROOT_ACCOUNT get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }'

near call controller.$ROOT_ACCOUNT set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }' --accountId controller.$ROOT_ACCOUNT
near view controller.$ROOT_ACCOUNT get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }'

wait
