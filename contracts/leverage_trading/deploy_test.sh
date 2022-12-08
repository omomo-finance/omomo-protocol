# login
#near login

# build & test
mkdir -p res && ./build.sh && ./test.sh

# clean up previuos deployment
echo 'y' | near delete order.mtrading_cl.testnet mtrading_cl.testnet

# create corresponding accoutns
near create-account order.mtrading_cl.testnet --masterAccount mtrading_cl.testnet --initialBalance 10

# redeploy contracts
near deploy order.mtrading_cl.testnet \
  --wasmFile ./res/limit_orders.wasm \
  --initFunction 'new_with_config' \
  --initArgs '{
        "owner_id":"order.mtrading_cl.testnet",
        "oracle_account_id":"limit_orders_oracle.v1.nearlend.testnet"
    }'

# register limit orders on tokens
near call wnear.qa.v1.nearlend.testnet storage_deposit '{"account_id": "order.mtrading_cl.testnet"}' --accountId order.mtrading_cl.testnet --amount 0.25 &
near call usdt.qa.v1.nearlend.testnet storage_deposit '{"account_id": "order.mtrading_cl.testnet"}' --accountId order.mtrading_cl.testnet --amount 0.25 &
wait

# add supported pairs
near call order.mtrading_cl.testnet add_pair '{
        "pair_data": {
            "sell_ticker_id": "USDT",
            "sell_token": "usdt.qa.v1.nearlend.testnet",
            "sell_token_market": "usdt_market.qa.v1.nearlend.testnet",
            "buy_ticker_id": "near",
            "buy_token": "wnear.qa.v1.nearlend.testnet",
            "pool_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000"
        }
    }' --accountId order.mtrading_cl.testnet &

near call order.mtrading_cl.testnet add_pair '{
        "pair_data": {
            "sell_ticker_id": "near",
            "sell_token": "wnear.qa.v1.nearlend.testnet",
            "sell_token_market": "wnear_market.qa.v1.nearlend.testnet",
            "buy_ticker_id": "USDT",
            "buy_token": "usdt.qa.v1.nearlend.testnet",
            "pool_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000"
        }
    }' --accountId order.mtrading_cl.testnet &

wait
near view order.mtrading_cl.testnet view_supported_pairs '{}'

# add mock prices
near call order.mtrading_cl.testnet update_or_insert_price '{
    "token_id":"usdt.qa.v1.nearlend.testnet",
    "price":{
        "ticker_id":"USDT",
        "value":"1.01"
    }
}' --accountId order.mtrading_cl.testnet

near call order.mtrading_cl.testnet update_or_insert_price '{
    "token_id":"wnear.qa.v1.nearlend.testnet",
    "price":{
        "ticker_id":"near",
        "value":"3.07"
    }
}' --accountId order.mtrading_cl.testnet &

wait
near view order.mtrading_cl.testnet view_price '{"token_id":"usdt.qa.v1.nearlend.testnet"}'
near view order.mtrading_cl.testnet view_price '{"token_id":"wnear.qa.v1.nearlend.testnet"}'

# add mock orders
near call order.mtrading_cl.testnet add_order '{
        "account_id":"mtrading_cl.testnet",
        "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}"
    }' --accountId order.mtrading_cl.testnet &

near call order.mtrading_cl.testnet add_order '{
        "account_id":"mtrading_cl.testnet",
        "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.01\"},\"block\":103930917,\"lpt_id\":\"2\"}"
    }' --accountId order.mtrading_cl.testnet &

near call order.mtrading_cl.testnet add_order '{
        "account_id":"mtrading_cl.testnet",
        "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"block\":103930918,\"lpt_id\":\"3\"}"
    }' --accountId order.mtrading_cl.testnet &


wait

near view order.mtrading_cl.testnet view_orders '{
    "account_id":"mtrading_cl.testnet",
    "buy_token":"wnear.qa.v1.nearlend.testnet",
    "sell_token":"usdt.qa.v1.nearlend.testnet"
}'

near view dcl.ref-dev.testnet get_pool '{"pool_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000"}'

near call order.mtrading_cl.testnet set_pool_id '{"pool_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000"}' --accountId order.mtrading_cl.testnet

 near call wnear.qa.v1.nearlend.testnet mint '{
        "account_id": "mtrading_cl.testnet",
        "amount": "10000000000000000000000000000"
    }' --accountId mtrading_cl.testnet &

near call usdt.qa.v1.nearlend.testnet mint '{
            "account_id": "mtrading_cl.testnet",
            "amount": "10000000000000000000000000000"
        }' --accountId mtrading_cl.testnet

near call usdt.qa.v1.nearlend.testnet mint '{
            "account_id": "order.mtrading_cl.testnet",
            "amount": "10000000000000000000000000000"
        }' --accountId mtrading_cl.testnet

 near call wnear.qa.v1.nearlend.testnet mint '{
        "account_id": "order.mtrading_cl.testnet",
        "amount": "10000000000000000000000000000"
    }' --accountId mtrading_cl.testnet

## Create order

near call usdt.qa.v1.nearlend.testnet ft_transfer_call '{"receiver_id": "order.mtrading_cl.testnet", "amount": "2000000000000000000000000000", "msg": "{\"Deposit\": {\"token\": \"usdt.qa.v1.nearlend.testnet\"}}"}' --accountId mtrading_cl.testnet --depositYocto 1 --gas 300000000000000

near view order.mtrading_cl.testnet balance_of '{"account_id": "mtrading_cl.testnet", "token": "usdt.qa.v1.nearlend.testnet" }'

near call order.mtrading_cl.testnet add_token_market '{"token_id": "usdt.qa.v1.nearlend.testnet", "market_id": "usdt_market.qa.v1.nearlend.testnet"}' --accountId order.mtrading_cl.testnet

# amount = 1000.0
# leverage = 1.0
near call order.mtrading_cl.testnet create_order '{
    "order_type": "Buy",
    "amount": "1000000000000000000000000000",
    "sell_token": "usdt.qa.v1.nearlend.testnet",
    "buy_token": "wnear.qa.v1.nearlend.testnet",
    "leverage": "1000000000000000000000000"
}' --accountId mtrading_cl.testnet --gas 300000000000000

# make sure lpt id is valid
near view order.mtrading_cl.testnet view_orders '{    "account_id":"mtrading_cl.testnet",
                                                          "buy_token":"wnear.qa.v1.nearlend.testnet",
                                                          "sell_token":"usdt.qa.v1.nearlend.testnet"}'


## Cancel order
near call order.mtrading_cl.testnet cancel_order '{"order_id": "4", "swap_fee": "1", "price_impact": "1"}' --accountId mtrading_cl.testnet --gas 300000000000000


