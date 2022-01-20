near delete dweth.nearlend.testnet nearlend.testnet
near create-account dweth.nearlend.testnet --masterAccount nearlend.testnet
near deploy --force dweth.nearlend.testnet --wasmFile ./target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new' --initArgs '{"underlying_token": "weth.nearlend.testnet"}'

near view dweth.nearlend.testnet ft_balance_of '{"account_id": "nearlend.testnet"}' --account_id nearlend.testnet
near call dweth.nearlend.testnet get_exchange_rate '{}' --account_id nearlend.testnet --gas 300000000000000
near view dweth.nearlend.testnet get_supplies '{"account": "nearlend.testnet"}' --account_id nearlend.testnet
near view dweth.nearlend.testnet get_borrows '{"account": "nearlend.testnet"}' --account_id nearlend.testnet
near view dweth.nearlend.testnet get_total_reserve '{}' --account_id nearlend.testnet
near view dweth.nearlend.testnet get_total_supplies '{}' --account_id nearlend.testnet
near view dweth.nearlend.testnet get_total_borrows '{}' --account_id nearlend.testnet

near delete dwnear.nearlend.testnet nearlend.testnet
near create-account dwnear.nearlend.testnet --masterAccount nearlend.testnet
near deploy --force dwnear.nearlend.testnet --wasmFile ./target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new' --initArgs '{"underlying_token": "wnear.nearlend.testnet"}'

near view dwnear.nearlend.testnet ft_balance_of '{"account_id": "nearlend.testnet"}' --account_id nearlend.testnet
near call dwnear.nearlend.testnet get_exchange_rate '{}' --account_id nearlend.testnet --gas 300000000000000
near view dwnear.nearlend.testnet get_supplies '{"account": "nearlend.testnet"}' --account_id nearlend.testnet
near view dwnear.nearlend.testnet get_borrows '{"account": "nearlend.testnet"}' --account_id nearlend.testnet
near view dwnear.nearlend.testnet get_total_reserve '{}' --account_id nearlend.testnet
near view dwnear.nearlend.testnet get_total_supplies '{}' --account_id nearlend.testnet
near view dwnear.nearlend.testnet get_total_borrows '{}' --account_id nearlend.testnet

## should fail
near call dweth.nearlend.testnet borrow_callback '{"amount": 100}' --account_id nearlend.testnet

## logic
near call dweth.nearlend.testnet supply '{"amount": 1000}' --account_id nearlend.testnet --gas 300000000000000
near call dweth.nearlend.testnet withdraw '{"amount": 1000}' --account_id nearlend.testnet --gas 300000000000000
near call dweth.nearlend.testnet borrow '{"amount": 10}' --account_id nearlend.testnet --gas 300000000000000