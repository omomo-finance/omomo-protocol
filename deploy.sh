# login
#near login

# clean up previuos deployment
near delete weth_beta.vlad_testing.testnet vlad_testing.testnet
near delete dweth_beta.vlad_testing.testnet vlad_testing.testnet
near delete controller_beta.vlad_testing.testnet vlad_testing.testnet
near delete oracle_beta.vlad_testing.testnet vlad_testing.testnet
near delete test.vlad_testing.testnet vlad_testing.testnet

# create corresponding accoutns
near create-account weth_beta.vlad_testing.testnet --masterAccount vlad_testing.testnet --initialBalance 5
near create-account dweth_beta.vlad_testing.testnet --masterAccount vlad_testing.testnet --initialBalance 5
near create-account controller_beta.vlad_testing.testnet --masterAccount vlad_testing.testnet --initialBalance 5
near create-account oracle_beta.vlad_testing.testnet --masterAccount vlad_testing.testnet --initialBalance 5
near create-account test.vlad_testing.testnet --masterAccount vlad_testing.testnet --initialBalance 5

# redeploy contracts
near deploy weth_beta.vlad_testing.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "vlad_testing.testnet", "name": "Wrapped Ethereum", "symbol": "WETH", "total_supply": "1000000000"}'

near deploy dweth_beta.vlad_testing.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "vlad_testing.testnet", "underlying_token_id": "weth_beta.vlad_testing.testnet", "controller_account_id": "controller_beta.vlad_testing.testnet", "initial_exchange_rate": "10000"}'

near deploy controller_beta.vlad_testing.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "vlad_testing.testnet", "oracle_account_id": "oracle_beta.vlad_testing.testnet"}'

# check states
near state weth_beta.vlad_testing.testnet
near view weth_beta.vlad_testing.testnet ft_metadata '{}' --accountId vlad_testing.testnet

near state dweth_beta.vlad_testing.testnet
near view dweth_beta.vlad_testing.testnet get_contract_config '{}' --accountId vlad_testing.testnet

near state controller_beta.vlad_testing.testnet
near view controller_beta.vlad_testing.testnet get_contract_config '{}' --accountId vlad_testing.testnet

###

near call weth_beta.vlad_testing.testnet mint '{"account_id": "dweth_beta.vlad_testing.testnet", "amount": "100000000000000000"}' --accountId vlad_testing.testnet

near state controller_beta.vlad_testing.testnet
near view controller_beta.vlad_testing.testnet get_contract_config '{}' --accountId vlad_testing.testnet

near call weth_beta.vlad_testing.testnet mint '{"account_id": "user1_beta.vlad_testing.testnet", "amount": "100000000000000000"}' --accountId vlad_testing.testnet

near call dweth_beta.vlad_testing.testnet increase_borrows '{"account": "user1_beta.vlad_testing.testnet", "token_amount": "10"}' --accountId vlad_testing.testnet

near call controller_beta.vlad_testing.testnet increase_borrows '{"account": "user1_beta.vlad_testing.testnet", "token_amount": "10", "token_address": "dweth_beta.vlad_testing.testnet"}' --accountId vlad_testing.testnet

near call weth_beta.vlad_testing.testnet ft_transfer_call '{"receiver_id": "dweth_beta.vlad_testing.testnet","amount":"10","msg":"\n    {\n        \"action\": \"SUPPLY\"\n    }"}' --accountId user1_beta.vlad_testing.testnet --depositYocto 1 --gas=300000000000000

near call weth_beta.vlad_testing.testnet ft_transfer_call '{"receiver_id": "dweth_beta.vlad_testing.testnet","amount":"10","msg":"\n    {\n        \"action\": \"LIQUIDATION\",\n        \"memo\": {\n            \"borrower\": \"user1_beta.vlad_testing.testnet\",\n            \"borrowing_dtoken\": \"dweth_beta.vlad_testing.testnet\",\n            \"liquidator\": \"test.vlad_testing.testnet\",\n            \"collateral_dtoken\": \"dweth_beta.vlad_testing.testnet\",\n            \"liquidation_amount\": \"10\"\n        }\n    }","sender_id":"weth_beta.vlad_testing.testnet"}' --accountId vlad_testing.testnet --depositYocto 1 --gas=300000000000000
