# we have 201 tokens supplied so can do borrow; param is your account
near call dweth_beta.nearlend.testnet borrow '{"token_amount": "5"}' --accountId "$1" --gas 300000000000000
