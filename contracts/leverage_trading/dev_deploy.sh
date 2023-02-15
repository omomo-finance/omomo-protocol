# login
#near login

# build & test
./build.sh && ./test.sh

near dev-deploy -f ./target/wasm32-unknown-unknown/release/leverage_trading.wasm
CONTRACT_ID="$(cat neardev/dev-account)"
# latest address version
NEAR_TOKEN=wnear.develop.v1.omomo-finance.testnet
NEAR_MARKET=wnear_market.develop.v1.omomo-finance.testnet
NEAR_TOKEN_DECIMALS=24

USDT_TOKEN=usdt.develop.v1.omomo-finance.testnet
USDT_MARKET=usdt_market.develop.v1.omomo-finance.testnet
USDT_TOKEN_DECIMALS=24
DEX_ACCOUNT=dclv2-dev.ref-dev.testnet
ORACLE_ACCOUNT=oracle.omomo-finance.testnet

# init contract
near call $CONTRACT_ID --accountId=$CONTRACT_ID new_with_config '{
       "owner_id":"'$CONTRACT_ID'",
       "oracle_account_id":"'$ORACLE_ACCOUNT'"
   }'

# register limit orders on tokens
near call $NEAR_TOKEN storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 0.25 &
near call $USDT_TOKEN storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 0.25 &
wait

# add supported pairs
near call $CONTRACT_ID add_pair '{
        "pair_data": {
            "sell_ticker_id": "USDT",
            "sell_token": "'$USDT_TOKEN'",
            "sell_token_decimals": '$USDT_TOKEN_DECIMALS',
            "sell_token_market": "'$USDT_MARKET'",
            "buy_ticker_id": "near",
            "buy_token": "'$NEAR_TOKEN'",
            "buy_token_decimals": '$NEAR_TOKEN_DECIMALS',
            "buy_token_market": "'$NEAR_MARKET'",
            "pool_id": "'$USDT_TOKEN'|'$NEAR_TOKEN'|2000",
            "max_leverage": "25000000000000000000000000",
            "swap_fee": "200000000000000000000"
        }
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_pair '{
        "pair_data": {
            "sell_ticker_id": "near",
            "sell_token": "'$NEAR_TOKEN'",
            "sell_token_decimals": '$NEAR_TOKEN_DECIMALS',
            "sell_token_market": "'$NEAR_MARKET'",
            "buy_ticker_id": "USDT",
            "buy_token": "'$USDT_TOKEN'",
            "buy_token_decimals": '$USDT_TOKEN_DECIMALS',
            "buy_token_market": "'$USDT_MARKET'",
            "pool_id": "'$USDT_TOKEN'|'$NEAR_TOKEN'|2000",
            "max_leverage": "25000000000000000000000000",
            "swap_fee": "200000000000000000000"
        }
    }' --accountId $CONTRACT_ID &

wait
near view $CONTRACT_ID view_supported_pairs '{}'

wait
near view $CONTRACT_ID view_pair_tokens_decimals '{
    "sell_token": "'$USDT_TOKEN'",
    "buy_token": "'$NEAR_TOKEN'"
}'

wait
near view $CONTRACT_ID view_pair_tokens_decimals '{
    "sell_token": "'$NEAR_TOKEN'",
    "buy_token": "'$USDT_TOKEN'"
}'

# add mock prices
near call $CONTRACT_ID update_or_insert_price '{
    "token_id":"'$USDT_TOKEN'",
    "price":{
        "ticker_id":"USDT",
        "value":"1010000000000000000000000"
    }
}' --accountId $CONTRACT_ID

near call $CONTRACT_ID update_or_insert_price '{
    "token_id":"'$NEAR_TOKEN'",
    "price":{
        "ticker_id":"near",
        "value":"2570000000000000000000000"
    }
}' --accountId $CONTRACT_ID &

wait
near view $CONTRACT_ID view_price '{"token_id":"'$USDT_TOKEN'"}'
near view $CONTRACT_ID view_price '{"token_id":"'$NEAR_TOKEN'"}'

# add mock orders
near call $CONTRACT_ID add_order_from_string '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$NEAR_TOKEN'\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"NEAR\",\"value\":\"4220000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930916,\"timestamp_ms\":86400000,\"lpt_id\":\"1\"}"
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_order_from_string '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$NEAR_TOKEN'\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"NEAR\",\"value\":\"3010000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930917,\"timestamp_ms\":86400000,\"lpt_id\":\"2\"}"
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_order_from_string '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$NEAR_TOKEN'\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"990000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"NEAR\",\"value\":\"3990000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930918,\"timestamp_ms\":86400000,\"lpt_id\":\"3\"}"
    }' --accountId $CONTRACT_ID &


wait

near view $CONTRACT_ID view_orders '{
    "account_id":"'$CONTRACT_ID'",
    "buy_token":"'$NEAR_TOKEN'",
    "sell_token":"'$USDT_TOKEN'",
    "borrow_rate_ratio": "1000"
}'

near view $DEX_ACCOUNT get_pool '{"pool_id": "'$USDT_TOKEN'|'$NEAR_TOKEN'|2000"}'

# mint 10000
near call $NEAR_TOKEN mint '{
        "account_id": "'$CONTRACT_ID'",
        "amount": "30000000000000000000000000000"
    }' --accountId $CONTRACT_ID &

# mint 10000
near call $USDT_TOKEN mint '{
        "account_id": "'$CONTRACT_ID'",
        "amount": "30000000000000000000000000000"
    }' --accountId $CONTRACT_ID


near call $CONTRACT_ID add_token_market '{"token_id": "'$USDT_TOKEN'", "market_id": "'$USDT_MARKET'"}' --accountId $CONTRACT_ID
near call $CONTRACT_ID add_token_market '{"token_id": "'$NEAR_TOKEN'", "market_id": "'$NEAR_MARKET'"}' --accountId $CONTRACT_ID
