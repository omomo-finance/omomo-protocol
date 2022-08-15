# suppose we have 1000 tokens so we can perform all the operations or see how to do it it user_flow.sh
# change myuser.testnet to your account to test

#  successful supply
near call --depositYocto 1 --gas 300000000000000 weth.nearlend.testnet ft_transfer_call '{"receiver_id": "weth_market.nearlend.testnet", "amount": "100", "msg":"{\"action\": \"SUPPLY\"}"}' --accountId "$1"
