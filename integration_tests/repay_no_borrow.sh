# trying to repay from another account that dont have borrowed assets -> change to yours account

near call --depositYocto 1 --gas 300000000000000 weth.nearlend.testnet ft_transfer_call '{"receiver_id": "weth_market.nearlend.testnet", "amount": "123", "msg":"\"Repay\""}' --accountId "$1"
