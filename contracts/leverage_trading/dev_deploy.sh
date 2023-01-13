# login
#near login

# build & test
./build.sh && ./test.sh

near dev-deploy -f  ./target/wasm32-unknown-unknown/release/leverage_trading.wasm
CONTRACT_ID="$(cat neardev/dev-account)"
# latest address version
USDT_TOKEN=usdt.dev.v1.omomo-finance.testnet
USDT_MARKET=usdt_market.dev.v1.omomo-finance.testnet
WNEAR_TOKEN=wnear.dev.v1.omomo-finance.testnet
WNEAR_MARKET=wnear_market.dev.v1.omomo-finance.testnet
ORACLE_ID=oracle.omomo-finance.testnet

# init contract
near call $CONTRACT_ID --accountId=$CONTRACT_ID new_with_config '{
       "owner_id":"'$CONTRACT_ID'",
       "oracle_account_id":"'$ORACLE_ID'"
   }'

# register limit orders on tokens
near call $WNEAR_TOKEN storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 0.25 &
near call $USDT_TOKEN storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 0.25 &
wait

# add supported pairs
near call $CONTRACT_ID add_pair '{
        "pair_data": {
            "sell_ticker_id": "USDT",
            "sell_token": "'$USDT_TOKEN'",
            "sell_token_market": "'$USDT_MARKET'",
            "buy_ticker_id": "near",
            "buy_token": "'$WNEAR_TOKEN'",
            "pool_id": "'$USDT_TOKEN'|'$WNEAR_TOKEN'|2000",
            "max_leverage": "25000000000000000000000000",
            "swap_fee": "100000000000000000000"
        }
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_pair '{
        "pair_data": {
            "sell_ticker_id": "near",
            "sell_token": "'$WNEAR_TOKEN'",
            "sell_token_market": "'$WNEAR_MARKET'",
            "buy_ticker_id": "USDT",
            "buy_token": "'$USDT_TOKEN'",
            "pool_id": "'$USDT_TOKEN'|'$WNEAR_TOKEN'|2000",
            "max_leverage": "25000000000000000000000000",
            "swap_fee": "100000000000000000000"
        }
    }' --accountId $CONTRACT_ID &

wait
near view $CONTRACT_ID view_supported_pairs '{}'

# add mock prices
near call $CONTRACT_ID update_or_insert_price '{
    "token_id":"'$USDT_TOKEN'",
    "price":{
        "ticker_id":"USDT",
        "value":"1.01"
    }
}' --accountId $CONTRACT_ID

near call $CONTRACT_ID update_or_insert_price '{
    "token_id":"'$WNEAR_TOKEN'",
    "price":{
        "ticker_id":"near",
        "value":"3.07"
    }
}' --accountId $CONTRACT_ID &

wait
near view $CONTRACT_ID view_price '{"token_id":"'$USDT_TOKEN'"}'
near view $CONTRACT_ID view_price '{"token_id":"'$WNEAR_TOKEN'"}'

# add mock orders
near call $CONTRACT_ID add_order '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$WNEAR_TOKEN'\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}"
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_order '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$WNEAR_TOKEN'\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.01\"},\"block\":103930917,\"lpt_id\":\"2\"}"
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_order '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$WNEAR_TOKEN'\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"block\":103930918,\"lpt_id\":\"3\"}"
    }' --accountId $CONTRACT_ID &


wait

near view $CONTRACT_ID view_orders '{
    "account_id":"'$CONTRACT_ID'",
    "buy_token":"'$WNEAR_TOKEN'",
    "sell_token":"'$USDT_TOKEN'",
    "borrow_rate_ratio": "1000"
}'

near view dcl.ref-dev.testnet get_pool '{"pool_id": "'$USDT_TOKEN'|'$WNEAR_TOKEN'|2000"}'

# mint 10000
near call $WNEAR_TOKEN mint '{
        "account_id": "'$CONTRACT_ID'",
        "amount": "30000000000000000000000000000"
    }' --accountId $CONTRACT_ID &

# mint 10000
near call $USDT_TOKEN mint '{
        "account_id": "'$CONTRACT_ID'",
        "amount": "30000000000000000000000000000"
    }' --accountId $CONTRACT_ID


near call $CONTRACT_ID add_token_market '{"token_id": "'$USDT_TOKEN'", "market_id": "'$USDT_MARKET'"}' --accountId $CONTRACT_ID
near call $CONTRACT_ID add_token_market '{"token_id": "'$WNEAR_TOKEN'", "market_id": "'$WNEAR_MARKET'"}' --accountId $CONTRACT_ID
