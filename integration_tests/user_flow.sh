# mint some tokens, change the accountId to yours
near call weth.nearlend.testnet mint '{"account_id": "myuser.testnet", "amount": "250000"}' --accountId myuser.testnet

# view balance
near view weth.nearlend.testnet ft_balance_of '{"account_id": "myuser.testnet"}' --accountId myuser.testnet