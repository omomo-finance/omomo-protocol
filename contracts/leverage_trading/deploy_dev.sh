# login
#near login

# build & test
./build.sh && ./test.sh

ROOT_ACCOUNT=develop.v1.omomo-finance.testnet
CONTRACT_ID=leverage.develop.v1.omomo-finance.testnet
# latest address version
USDT_TOKEN=usdt.develop.v1.omomo-finance.testnet
USDT_MARKET=usdt_market.develop.v1.omomo-finance.testnet
WNEAR_TOKEN=wnear.develop.v1.omomo-finance.testnet
WNEAR_MARKET=wnear_market.develop.v1.omomo-finance.testnet
ORACLE_ID=oracle.omomo-finance.testnet
DEX_ACCOUNT=dclv2-dev.ref-dev.testnet

# clean up previuos deployment
echo 'y' | near delete ${CONTRACT_ID} $ROOT_ACCOUNT

# create corresponding accoutns
near create-account ${CONTRACT_ID} --masterAccount $ROOT_ACCOUNT --initialBalance 10


# init contract
near deploy ${CONTRACT_ID} \
  --wasmFile  ./target/wasm32-unknown-unknown/release/leverage_trading.wasm \
  --initFunction 'new_with_config' \
  --initArgs '{
        "owner_id":"'${CONTRACT_ID}'",
        "oracle_account_id":"'$ORACLE_ID'"
    }'

# register limit orders on tokens
near call $WNEAR_TOKEN storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 0.25 &
near call $USDT_TOKEN storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 0.25 &
wait

near call $WNEAR_TOKEN storage_deposit '{"account_id": "'$DEX_ACCOUNT'"}' --accountId $CONTRACT_ID --amount 0.25 &
near call $USDT_TOKEN storage_deposit '{"account_id": "'$DEX_ACCOUNT'"}' --accountId $CONTRACT_ID --amount 0.25 &
wait

# add supported pairs
near call $CONTRACT_ID add_pair '{
        "pair_data": {
            "sell_ticker_id": "USDt",
            "sell_token": "'$USDT_TOKEN'",
            "sell_token_market": "'$USDT_MARKET'",
            "buy_ticker_id": "near",
            "buy_token": "'$WNEAR_TOKEN'",
            "pool_id": "'$USDT_TOKEN'|'$WNEAR_TOKEN'|2000",
            "max_leverage": "25000000000000000000000000",
            "swap_fee": "300000000000000000000"
        }
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_pair '{
        "pair_data": {
            "sell_ticker_id": "near",
            "sell_token": "'$WNEAR_TOKEN'",
            "sell_token_market": "'$WNEAR_MARKET'",
            "buy_ticker_id": "USDt",
            "buy_token": "'$USDT_TOKEN'",
            "pool_id": "'$USDT_TOKEN'|'$WNEAR_TOKEN'|2000",
            "max_leverage": "25000000000000000000000000",
            "swap_fee": "300000000000000000000"
        }
    }' --accountId $CONTRACT_ID &

wait
near view $CONTRACT_ID view_supported_pairs '{}'

# add mock prices
near call $CONTRACT_ID update_or_insert_price '{
    "token_id":"'$USDT_TOKEN'",
    "price":{
        "ticker_id":"USDt",
        "value":"1.01"
    }
}' --accountId $CONTRACT_ID &

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
        "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$WNEAR_TOKEN'\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDt\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}"
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_order '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$WNEAR_TOKEN'\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDt\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.01\"},\"block\":103930917,\"lpt_id\":\"2\"}"
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_order '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$WNEAR_TOKEN'\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDt\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"block\":103930918,\"lpt_id\":\"3\"}"
    }' --accountId $CONTRACT_ID &


wait

near view $CONTRACT_ID view_orders '{
    "account_id":"'$CONTRACT_ID'",
    "buy_token":"'$WNEAR_TOKEN'",
    "sell_token":"'$USDT_TOKEN'",
    "borrow_rate_ratio": "1000"
}'

near view dclv2-dev.ref-dev.testnet get_pool '{"pool_id": "'$USDT_TOKEN'|'$WNEAR_TOKEN'|2000"}'

# mint 30000
near call $WNEAR_TOKEN mint '{
        "account_id": "'$CONTRACT_ID'",
        "amount": "30000000000000000000000000000"
    }' --accountId $CONTRACT_ID &

# mint 30000
near call $USDT_TOKEN mint '{
        "account_id": "'$CONTRACT_ID'",
        "amount": "30000000000000000000000000000"
    }' --accountId $CONTRACT_ID


near call $CONTRACT_ID add_token_market '{"token_id": "'$USDT_TOKEN'", "market_id": "'$USDT_MARKET'"}' --accountId $CONTRACT_ID
near call $CONTRACT_ID add_token_market '{"token_id": "'$WNEAR_TOKEN'", "market_id": "'$WNEAR_MARKET'"}' --accountId $CONTRACT_ID

near call $USDT_MARKET set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ID}'" }' --accountId shared_admin.testnet
near view $USDT_MARKET get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ID}'" }'

near call controller.$ROOT_ACCOUNT set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ID}'" }' --accountId controller.$ROOT_ACCOUNT
near view controller.$ROOT_ACCOUNT get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ID}'" }'
