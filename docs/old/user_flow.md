near call dweth.nearlend.testnet supply '{"amount": 1000}' --account_id nearlend.testnet --gas 300000000000000
near view dweth.nearlend.testnet get_supplies '{"account": "nearlend.testnet"}' --account_id nearlend.testnet
near view dweth.nearlend.testnet get_total_supplies '{}' --account_id nearlend.testnet
near view dweth.nearlend.testnet ft_balance_of '{ "account_id": "nearlend.testnet" }' --accountId nearlend.testnet
near view weth.nearlend.testnet ft_balance_of '{ "account_id": "nearlend.testnet" }' --accountId nearlend.testnet
near view weth.nearlend.testnet ft_balance_of '{ "account_id": "dweth.nearlend.testnet" }' --accountId nearlend.testnet

near call dweth.nearlend.testnet borrow '{"amount": 100}' --account_id nearlend.testnet --gas 300000000000000
near view dweth.nearlend.testnet get_borrows '{"account": "nearlend.testnet"}' --account_id nearlend.testnet
near view dweth.nearlend.testnet get_total_borrows '{}' --account_id nearlend.testnet
near view dweth.nearlend.testnet ft_balance_of '{ "account_id": "nearlend.testnet" }' --accountId nearlend.testnet
near view weth.nearlend.testnet ft_balance_of '{ "account_id": "nearlend.testnet" }' --accountId nearlend.testnet
near view weth.nearlend.testnet ft_balance_of '{ "account_id": "dweth.nearlend.testnet" }' --accountId nearlend.testnet


near call dweth.nearlend.testnet repay '{}' --account_id nearlend.testnet --gas 300000000000000
near view dweth.nearlend.testnet get_borrows '{"account": "nearlend.testnet"}' --account_id nearlend.testnet
near view dweth.nearlend.testnet get_total_borrows '{}' --account_id nearlend.testnet
near view dweth.nearlend.testnet ft_balance_of '{ "account_id": "nearlend.testnet" }' --accountId nearlend.testnet
near view weth.nearlend.testnet ft_balance_of '{ "account_id": "nearlend.testnet" }' --accountId nearlend.testnet
near view weth.nearlend.testnet ft_balance_of '{ "account_id": "dweth.nearlend.testnet" }' --accountId nearlend.testnet


near call dweth.nearlend.testnet withdraw '{"amount": 1000}' --account_id nearlend.testnet --gas 300000000000000
near view dweth.nearlend.testnet get_supplies '{"account": "nearlend.testnet"}' --account_id nearlend.testnet
near view dweth.nearlend.testnet get_total_supplies '{}' --account_id nearlend.testnet