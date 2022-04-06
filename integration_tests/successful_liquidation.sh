# other account to liquidate other_account.myuser.testnet

# near call --depositYocto 1 --gas 300000000000000 weth_beta.nearlend.testnet ft_transfer_call  '{"receiver_id": "dweth_beta.nearlend.testnet","amount":"100","msg":"\n    {\n        \"Liquidate\": {\n            \"borrower\": \"myuser.testnet\",\n            \"borrowing_dtoken\": \"dweth_beta.nearlend.testnet\",\n            \"liquidator\": \"other_account.myuser.testnet\",\n            \"collateral_dtoken\": \"dweth_beta.nearlend.testnet\",\n            \"liquidation_amount\": \"1000\"\n        }\n    }","sender_id":"dweth_beta.nearlend.testnet"}'  --accountId myuser.testnet


