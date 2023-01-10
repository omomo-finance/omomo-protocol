CONTRACT_ID="$(cat neardev/dev-account)"
# latest address version
USDT_TOKEN=usdt.dev.v1.omomo-finance.testnet
USDT_MARKET=usdt_market.dev.v1.omomo-finance.testnet
WNEAR_TOKEN=wnear.dev.v1.omomo-finance.testnet
WNEAR_MARKET=wnear_market.dev.v1.omomo-finance.testnet
ORACLE_ID=limit_orders_oracle.v1.nearlend.testnet

# User account for work with leverage trading
USER_ACCOUNT=...

near call dcl.ref-dev.testnet storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 1

near call $USDT_TOKEN mint '{
    "account_id": "'$USER_ACCOUNT'",
    "amount": "4000000000000000000000000000000000"
}' --accountId $USER_ACCOUNT

near call $WNEAR_TOKEN mint '{
    "account_id": "'$USER_ACCOUNT'",
    "amount": "4000000000000000000000000000000000"
}' --accountId $USER_ACCOUNT

# Deposit
near call $USDT_TOKEN ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "1000000000000000000000000000", "msg": "{\"Deposit\": {\"token\": \"'$USDT_TOKEN'\"}}"}' --accountId $USER_ACCOUNT --depositYocto 1 --gas 300000000000000
near view $CONTRACT_ID balance_of '{"account_id": "'$USER_ACCOUNT'", "token": "'$USDT_TOKEN'" }'

# Create order
# amount = 1000.0
# leverage = 1.0
near call $CONTRACT_ID create_order '{
    "order_type": "Buy",
    "amount": "1000000000000000000000000000",
    "sell_token": "'$USDT_TOKEN'",
    "buy_token": "'$WNEAR_TOKEN'",
    "leverage": "10000000000000000000000000"
}' --accountId $USER_ACCOUNT --gas 300000000000000 --depositYocto 100000000000000

# View new order
near view $CONTRACT_ID view_orders '{
    "account_id":"'$USER_ACCOUNT'",
    "buy_token":"'$WNEAR_TOKEN'",
    "sell_token":"'$USDT_TOKEN'",
    "borrow_rate_ratio": "1000"
}'

# Cancel order
near call $CONTRACT_ID cancel_order '{"order_id": "4", "swap_fee": "1", "price_impact": "1"}' --accountId $USER_ACCOUNT --gas 160000000000000
# Deposit balance should be returned after cancel order
near view $CONTRACT_ID balance_of '{"account_id": "'$USER_ACCOUNT'", "token": "'$USDT_TOKEN'" }'



