CONTRACT_ID="$(cat neardev/dev-account)"
# latest address version
NEAR_TOKEN=wnear.develop.v1.omomo-finance.testnet
NEAR_MARKET=wnear_market.develop.v1.omomo-finance.testnet
NEAR_TOKEN_DECIMALS=24

USDT_TOKEN=usdt.develop.v1.omomo-finance.testnet
USDT_MARKET=usdt_market.develop.v1.omomo-finance.testnet
USDT_TOKEN_DECIMALS=24
DEX_ACCOUNT=dclv2-dev.ref-dev.testnet

# User account for work with leverage trading
USER_ACCOUNT=...

near call $DEX_ACCOUNT storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $CONTRACT_ID --amount 1

near call $USDT_TOKEN mint '{
    "account_id": "'$USER_ACCOUNT'",
    "amount": "4000000000000000000000000000000000"
}' --accountId $USER_ACCOUNT

near call $NEAR_TOKEN mint '{
    "account_id": "'$USER_ACCOUNT'",
    "amount": "4000000000000000000000000000000000"
}' --accountId $USER_ACCOUNT

near call $USDT_TOKEN mint '{
    "account_id": "'$CONTRACT_ID'",
    "amount": "4000000000000000000000000000000000"
}' --accountId $USER_ACCOUNT

near call $NEAR_TOKEN mint '{
    "account_id": "'$CONTRACT_ID'",
    "amount": "4000000000000000000000000000000000"
}' --accountId $USER_ACCOUNT

# Deposit
near call $USDT_TOKEN ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "1000000000000000000000000000", "msg": "{\"Deposit\": {\"token\": \"'$USDT_TOKEN'\"}}"}' --accountId $USER_ACCOUNT --depositYocto 1 --gas 300000000000000
near view $CONTRACT_ID balance_of '{"account_id": "'$USER_ACCOUNT'", "token": "'$USDT_TOKEN'" }'

# Create order
# amount = 1000.0
# leverage = 1.0
# open_price = 1.0
# current_point = -7000
near call $CONTRACT_ID create_order '{
    "order_type": "Buy",
    "left_point": -6960,
    "right_point": -6920,
    "amount": "1000000000000000000000000000",
    "sell_token": "'$USDT_TOKEN'",
    "buy_token": "'$NEAR_TOKEN'",
    "leverage": "1000000000000000000000000",
    "open_price": "1000000000000000000000000"
}' --accountId $USER_ACCOUNT --gas 300000000000000 --depositYocto 100000000000000

# View new order
near view $CONTRACT_ID view_orders '{
    "account_id":"'$USER_ACCOUNT'",
    "buy_token":"'$NEAR_TOKEN'",
    "sell_token":"'$USDT_TOKEN'",
    "borrow_rate_ratio": "1000"
}'

# Cancel order
near call $CONTRACT_ID cancel_order '{"order_id": "4", "price_impact": "1"}' --accountId $USER_ACCOUNT --gas 160000000000000
# Deposit balance should be returned after cancel order
near view $CONTRACT_ID balance_of '{"account_id": "'$USER_ACCOUNT'", "token": "'$USDT_TOKEN'" }'

# View orders
near view $CONTRACT_ID view_orders '{
    "account_id":"'$USER_ACCOUNT'",
    "buy_token":"'$NEAR_TOKEN'",
    "sell_token":"'$USDT_TOKEN'",
    "borrow_rate_ratio": "1000"
}'