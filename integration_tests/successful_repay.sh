# param is your account
# assumptions: we have borrowed 5 tokens so can repay including borrow rate
# see the debt and change the replayed amount ( mint some if necessary )

near call --depositYocto 1 --gas 300000000000000 weth_beta.nearlend.testnet ft_transfer_call '{"receiver_id": "dweth_beta.nearlend.testnet", "amount": "-amount-including-borrow-rate", "msg":"{\"action\": \"REPAY\"}"}' --accountId "$1"
