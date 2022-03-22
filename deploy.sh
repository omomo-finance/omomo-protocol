# login
near login

# clean up previuos deployment
near delete weth_beta.nearlend.testnet nearlend.testnet
near delete dweth_beta.nearlend.testnet nearlend.testnet
near delete controller_beta.nearlend.testnet nearlend.testnet
near delete oracle_beta.nearlend.testnet nearlend.testnet

# create corresponding accoutns
near create-account weth_beta.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 5
near create-account dweth_beta.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 5
near create-account controller_beta.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 5
near create-account oracle_beta.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 5

# redeploy contracts
near deploy weth_beta.nearlend.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm --initFunction 'new_default_meta' --initArgs '{"owner_id": "nearlend.testnet", "name": "Wrapped Ethereum", "symbol": "WETH", "total_supply": "1000000000"}'

near deploy dweth_beta.nearlend.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "nearlend.testnet", "underlying_token_id": "weth_beta.nearlend.testnet", "controller_account_id": "controller_beta.nearlend.testnet", "initial_exchange_rate": "10000"}'

near deploy controller_beta.nearlend.testnet --wasmFile ./contracts/target/wasm32-unknown-unknown/release/controller.wasm --initFunction 'new_with_config' --initArgs '{"owner_id": "nearlend.testnet", "oracle_account_id": "oracle_beta.nearlend.testnet"}'

# check states
near state weth_beta.nearlend.testnet
near view weth_beta.nearlend.testnet ft_metadata '{}' --accountId nearlend.testnet

near state dweth_beta.nearlend.testnet
near view dweth_beta.nearlend.testnet get_contract_config '{}' --accountId nearlend.testnet

near state controller_beta.nearlend.testnet
near view controller_beta.nearlend.testnet get_contract_config '{}' --accountId nearlend.testnet