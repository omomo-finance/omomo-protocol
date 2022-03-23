near call weth_beta.nearlend.testnet mint '{"account_id": "nearlend.testnet", "amount": "1000"}' --accountId nearlend.testnet
near view weth_beta.nearlend.testnet ft_balance_of '{"account_id": "nearlend.testnet"}' --accountId nearlend.testnet

near call --depositYocto 1 --gas 300000000000000 weth_beta.nearlend.testnet ft_transfer_call '{"receiver_id": "dweth_beta.nearlend.testnet", "amount": "100", "msg":"{\"action\": \"SUPPLY\"}"}' --accountId nearlend.testnet