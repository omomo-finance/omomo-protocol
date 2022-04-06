# supply zero tokens -> ExecutionError
near call --depositYocto 1 --gas 300000000000000 weth_beta.nearlend.testnet ft_transfer_call '{"receiver_id": "dweth_beta.nearlend.testnet", "amount": "0", "msg":"{\"action\": \"SUPPLY\"}"}' --accountId "$1"
