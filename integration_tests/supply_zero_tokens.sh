# supply zero tokens -> ExecutionError
near call --depositYocto 1 --gas 300000000000000 weth.nearlend.testnet ft_transfer_call '{"receiver_id": "weth_market.nearlend.testnet", "amount": "0", "msg":"{\"action\": \"SUPPLY\"}"}' --accountId "$1"
