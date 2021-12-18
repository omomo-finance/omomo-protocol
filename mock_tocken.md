## WETH
near delete weth.nearlend.testnet nearlend.testnet

near create-account weth.nearlend.testnet --masterAccount nearlend.testnet

near deploy weth.nearlend.testnet --wasmFile ./target/wasm32-unknown-unknown/release/token_mock.wasm

near call weth.nearlend.testnet new_default_meta '{"symbol": "WETH", "ft_name": "WETH", "total_supply": "1000000000000000000000000000000"}' --accountId weth.nearlend.testnet

near call weth.nearlend.testnet give_tokens_to '{ "receiver_id": "nearlend.testnet", "amount": "1000000000000000000000000000" }' --accountId nearlend.testnet

near call weth.nearlend.testnet ft_total_supply '{}' --accountId nearlend.testnet

near call weth.nearlend.testnet ft_balance_of '{ "account_id": "nearlend.testnet" }' --accountId nearlend.testnet


## WNEAR
near delete wnear.nearlend.testnet nearlend.testnet

near create-account wnear.nearlend.testnet --masterAccount nearlend.testnet

near deploy wnear.nearlend.testnet --wasmFile ./target/wasm32-unknown-unknown/release/token_mock.wasm

near call wnear.nearlend.testnet new_default_meta '{"symbol": "WNEAR", "ft_name": "WNEAR", "total_supply": "1000000000000000000000000000000"}' --accountId wnear.nearlend.testnet

near call wnear.nearlend.testnet give_tokens_to '{ "receiver_id": "nearlend.testnet", "amount": "1000000000000000000000000000" }' --accountId nearlend.testnet

near call wnear.nearlend.testnet ft_balance_of '{ "account_id": "nearlend.testnet" }' --accountId nearlend.testnet




###
near call wnear.nearlend.testnet ft_transfer '{"receiver_id": "nearlend.testnet", "amount": "1000000000000000000000000000" }'  --amount 0.000000000000000000000001 --accountId wnear.nearlend.testnet
