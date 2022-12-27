# login
#near login

# build & test
mkdir -p res && ./build.sh && ./test.sh

CONTRACT_ADDRESS=leverage.dev.v1.omomo-finance.testnet

# clean up previuos deployment
echo 'y' | near delete ${CONTRACT_ADDRESS} dev.v1.omomo-finance.testnet

# create corresponding accoutns
near create-account ${CONTRACT_ADDRESS} --masterAccount dev.v1.omomo-finance.testnet --initialBalance 10

# redeploy contracts
# --wasmFile ./res/limit_orders.wasm
near deploy ${CONTRACT_ADDRESS} \
  --wasmFile ./res/limit_orders.wasm \
  --initFunction 'new_with_config' \
  --initArgs '{
        "owner_id":"'${CONTRACT_ADDRESS}'",
        "oracle_account_id":"limit_orders_oracle.dev.v1.omomo-finance.testnet"
    }'

# register limit orders on tokens
near call wnear.dev.v1.omomo-finance.testnet storage_deposit '{"account_id": "'${CONTRACT_ADDRESS}'"}' --accountId ${CONTRACT_ADDRESS} --amount 0.25 &
near call usdt.dev.v1.omomo-finance.testnet storage_deposit '{"account_id": "'${CONTRACT_ADDRESS}'"}' --accountId ${CONTRACT_ADDRESS} --amount 0.25 &
wait

# add supported pairs
near call ${CONTRACT_ADDRESS} add_pair '{
        "pair_data": {
            "sell_ticker_id": "USDt",
            "sell_token": "usdt.dev.v1.omomo-finance.testnet",
            "sell_token_market": "usdt_market.dev.v1.omomo-finance.testnet",
            "buy_ticker_id": "near",
            "buy_token": "wnear.dev.v1.omomo-finance.testnet",
            "pool_id": "usdt.dev.v1.omomo-finance.testnet|wnear.dev.v1.omomo-finance.testnet|2000",
            "max_leverage": "25"
        }
    }' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} add_pair '{
        "pair_data": {
            "sell_ticker_id": "near",
            "sell_token": "wnear.dev.v1.omomo-finance.testnet",
            "sell_token_market": "wnear_market.dev.v1.omomo-finance.testnet",
            "buy_ticker_id": "USDt",
            "buy_token": "usdt.dev.v1.omomo-finance.testnet",
            "pool_id": "usdt.dev.v1.omomo-finance.testnet|wnear.dev.v1.omomo-finance.testnet|2000",
            "max_leverage": "25"
        }
    }' --accountId ${CONTRACT_ADDRESS} &


near call ${CONTRACT_ADDRESS} set_max_order_amount '{
    "value": "10000000000000000000000000000"
}' --accountId ${CONTRACT_ADDRESS} &

wait
# near view ${CONTRACT_ADDRESS} view_supported_pairs '{}'

# add mock prices
near call ${CONTRACT_ADDRESS} update_or_insert_price '{
    "token_id":"usdt.dev.v1.omomo-finance.testnet",
    "price":{
        "ticker_id":"USDt",
        "value":"1.01"
    }
}' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} update_or_insert_price '{
    "token_id":"wnear.dev.v1.omomo-finance.testnet",
    "price":{
        "ticker_id":"near",
        "value":"1.83"
    }
}' --accountId ${CONTRACT_ADDRESS} &

wait

near view ${CONTRACT_ADDRESS} view_price '{"token_id":"usdt.dev.v1.omomo-finance.testnet"}'
near view ${CONTRACT_ADDRESS} view_price '{"token_id":"wnear.dev.v1.omomo-finance.testnet"}'

# add mock orders
near call ${CONTRACT_ADDRESS} add_order '{
        "account_id":"tommylinks.testnet",
        "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"usdt.dev.v1.omomo-finance.testnet\",\"buy_token\":\"wnear.dev.v1.omomo-finance.testnet\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}"
    }' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} add_order '{
        "account_id":"tommylinks.testnet",
        "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"usdt.dev.v1.omomo-finance.testnet\",\"buy_token\":\"wnear.dev.v1.omomo-finance.testnet\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.01\"},\"block\":103930917,\"lpt_id\":\"2\"}"
    }' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} add_order '{
        "account_id":"tommylinks.testnet",
        "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"usdt.dev.v1.omomo-finance.testnet\",\"buy_token\":\"wnear.dev.v1.omomo-finance.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"block\":103930918,\"lpt_id\":\"3\"}"
    }' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} add_order '{
        "account_id":"nearlend.testnet",
        "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"usdt.dev.v1.omomo-finance.testnet\",\"buy_token\":\"wnear.dev.v1.omomo-finance.testnet\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}"
    }' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} add_order '{
        "account_id":"nearlend.testnet",
        "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"usdt.dev.v1.omomo-finance.testnet\",\"buy_token\":\"wnear.dev.v1.omomo-finance.testnet\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.01\"},\"block\":103930917,\"lpt_id\":\"2\"}"
    }' --accountId ${CONTRACT_ADDRESS} &


near call ${CONTRACT_ADDRESS} add_order '{
        "account_id":"nearlend.testnet",
        "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"usdt.dev.v1.omomo-finance.testnet\",\"buy_token\":\"wnear.dev.v1.omomo-finance.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"block\":103930918,\"lpt_id\":\"3\"}"
    }' --accountId ${CONTRACT_ADDRESS} &

wait

# setup pool
near call dcl.ref-dev.testnet storage_deposit '{"account_id": "'${CONTRACT_ADDRESS}'"}' --accountId nearlend.testnet --amount 1 &

near call ${CONTRACT_ADDRESS} add_token_market '{"token_id": "wnear.dev.v1.omomo-finance.testnet", "market_id": "wnear_market.dev.v1.omomo-finance.testnet"}' --account_id ${CONTRACT_ADDRESS} &
near call ${CONTRACT_ADDRESS} add_token_market '{"token_id": "usdt.dev.v1.omomo-finance.testnet", "market_id": "usdt_market.dev.v1.omomo-finance.testnet"}' --account_id ${CONTRACT_ADDRESS} &

near call usdt_market.dev.v1.omomo-finance.testnet set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }' --accountId shared_admin.testnet
near view usdt_market.dev.v1.omomo-finance.testnet get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }'

near call controller.dev.v1.omomo-finance.testnet set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }' --accountId controller.dev.v1.omomo-finance.testnet
near view controller.dev.v1.omomo-finance.testnet get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }'

wait
