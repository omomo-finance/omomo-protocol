# login
#near login

# build & test
./build.sh && ./test.sh

ROOT_ACCOUNT=develop.v1.omomo-finance.testnet
CONTRACT_ID=leverage.develop.v1.omomo-finance.testnet

# latest address version
ETH_TOKEN=weth.develop.v1.omomo-finance.testnet
ETH_MARKET=weth_market.develop.v1.omomo-finance.testnet
ETH_TOKEN_DECIMALS=18

NEAR_TOKEN=wnear.develop.v1.omomo-finance.testnet
NEAR_MARKET=wnear_market.develop.v1.omomo-finance.testnet
NEAR_TOKEN_DECIMALS=24

USDT_TOKEN=usdt.develop.v1.omomo-finance.testnet
USDT_MARKET=usdt_market.develop.v1.omomo-finance.testnet
USDT_TOKEN_DECIMALS=24

USDC_TOKEN=usdc.develop.v1.omomo-finance.testnet
USDC_MARKET=usdc_market.develop.v1.omomo-finance.testnet
USDC_TOKEN_DECIMALS=6

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
near call $NEAR_TOKEN storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 0.25 &
near call $USDT_TOKEN storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 0.25 &
wait

near call $NEAR_TOKEN storage_deposit '{"account_id": "'$DEX_ACCOUNT'"}' --accountId $CONTRACT_ID --amount 0.25 &
near call $USDT_TOKEN storage_deposit '{"account_id": "'$DEX_ACCOUNT'"}' --accountId $CONTRACT_ID --amount 0.25 &
wait

# add supported pairs
near call $CONTRACT_ID add_pair '{
        "pair_data": {
            "sell_ticker_id": "USDt",
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
            "sell_ticker_id": "USDt",
            "sell_token": "'$USDT_TOKEN'",
            "sell_token_decimals": '$USDT_TOKEN_DECIMALS',
            "sell_token_market": "'$USDT_MARKET'",
            "buy_ticker_id": "nWETH",
            "buy_token": "'$ETH_TOKEN'",
            "buy_token_decimals": '$ETH_TOKEN_DECIMALS',
            "buy_token_market": "'$ETH_MARKET'",
            "pool_id": "'$USDT_TOKEN'|'$ETH_TOKEN'|2000",
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
    "sell_token": "'$USDT_TOKEN'",
    "buy_token": "'$ETH_TOKEN'"
}'

# add mock prices
near call $CONTRACT_ID update_or_insert_price '{
    "token_id":"'$USDT_TOKEN'",
    "price":{
        "ticker_id":"USDt",
        "value":"1010000000000000000000000"
    }
}' --accountId $CONTRACT_ID &

near call $CONTRACT_ID update_or_insert_price '{
    "token_id":"'$NEAR_TOKEN'",
    "price":{
        "ticker_id":"near",
        "value":"2570000000000000000000000"
    }
}' --accountId $CONTRACT_ID &


near call $CONTRACT_ID update_or_insert_price '{
    "token_id":"'$ETH_TOKEN'",
    "price":{
        "ticker_id":"nWETH",
        "value":"1623670000000000000000000000"
    }
}' --accountId $CONTRACT_ID &

wait
near view $CONTRACT_ID view_price '{"token_id":"'$USDT_TOKEN'"}'
near view $CONTRACT_ID view_price '{"token_id":"'$NEAR_TOKEN'"}'
near view $CONTRACT_ID view_price '{"token_id":"'$ETH_TOKEN'"}'

# add mock orders
near call $CONTRACT_ID add_order_from_string '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$NEAR_TOKEN'\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDt\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"open_or_close_price\":\"2.5\",\"block\":103930916,\"timestamp_ms\":86400000,\"lpt_id\":\"1\"}"
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_order_from_string '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$NEAR_TOKEN'\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDt\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.01\"},\"open_or_close_price\":\"2.5\",\"block\":103930917,\"timestamp_ms\":86400000,\"lpt_id\":\"2\"}"
    }' --accountId $CONTRACT_ID &

near call $CONTRACT_ID add_order_from_string '{
        "account_id":"'$CONTRACT_ID'",
        "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"'$USDT_TOKEN'\",\"buy_token\":\"'$NEAR_TOKEN'\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDt\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"open_or_close_price\":\"2.5\",\"block\":103930918,\"timestamp_ms\":86400000,\"lpt_id\":\"3\"}"
    }' --accountId $CONTRACT_ID &


wait

near view $CONTRACT_ID view_orders '{
    "account_id":"'$CONTRACT_ID'",
    "buy_token":"'$NEAR_TOKEN'",
    "sell_token":"'$USDT_TOKEN'",
    "borrow_rate_ratio": "1000"
}'

near view dclv2-dev.ref-dev.testnet get_pool '{"pool_id": "'$USDT_TOKEN'|'$NEAR_TOKEN'|2000"}'

# mint 30000
near call $NEAR_TOKEN mint '{
        "account_id": "'$CONTRACT_ID'",
        "amount": "30000000000000000000000000000"
    }' --accountId $CONTRACT_ID &

# mint 30000
near call $USDT_TOKEN mint '{
        "account_id": "'$CONTRACT_ID'",
        "amount": "30000000000000000000000000000"
    }' --accountId $CONTRACT_ID


near call $CONTRACT_ID add_token_market '{"token_id": "'$USDT_TOKEN'", "market_id": "'$USDT_MARKET'"}' --accountId $CONTRACT_ID
near call $CONTRACT_ID add_token_market '{"token_id": "'$NEAR_TOKEN'", "market_id": "'$NEAR_MARKET'"}' --accountId $CONTRACT_ID

near call $USDT_MARKET set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ID}'" }' --accountId shared_admin.testnet
near view $USDT_MARKET get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ID}'" }'

near call controller.$ROOT_ACCOUNT set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ID}'" }' --accountId controller.$ROOT_ACCOUNT
near view controller.$ROOT_ACCOUNT get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ID}'" }'
