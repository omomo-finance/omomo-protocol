# used previously created account with no supply another_account <- param is your account
near call dweth_beta.nearlend.testnet withdraw '{"dtoken_amount": "1"}' --accountId "$1" --gas 300000000000000
