near delete ctrl.nearlend.testnet nearlend.testnet
near create-account ctrl.nearlend.testnet --masterAccount nearlend.testnet
near deploy --force ctrl.nearlend.testnet --wasmFile ./target/wasm32-unknown-unknown/release/controller.wasm


near call ctrl.nearlend.testnet add_market '{ "underlying": "weth.nearlend.testnet", "dtoken_address": "dweth.nearlend.testnet" }' --account_id nearlend.testnet
near call ctrl.nearlend.testnet add_market '{ "underlying": "wnear.nearlend.testnet", "dtoken_address": "dwnear.nearlend.testnet" }' --account_id nearlend.testnet
near view ctrl.nearlend.testnet get_markets '{}' --account_id nearlend.testnet


near call ctrl.nearlend.testnet set_interest_rate_model '{ "dtoken_address": "dweth.nearlend.testnet", "interest_rate_model_address": "math.nearlend.testnet" }' --account_id nearlend.testnet
near call ctrl.nearlend.testnet get_interest_rate '{ "dtoken_address": "dweth.nearlend.testnet", "underlying_balance":100, "total_borrows":100, "total_reserve":0 }' --account_id nearlend.testnet


near call ctrl.nearlend.testnet set_interest_rate_model '{ "dtoken_address": "dwnear.nearlend.testnet", "interest_rate_model_address": "math.nearlend.testnet" }' --account_id nearlend.testnet
near call ctrl.nearlend.testnet get_interest_rate '{ "dtoken_address": "dwnear.nearlend.testnet", "underlying_balance":100, "total_borrows":100, "total_reserve":0 }' --account_id nearlend.testnet


# 4026.59$ WETH price with 8 decimals 
near call ctrl.nearlend.testnet set_price '{ "dtoken_address": "dweth.nearlend.testnet", "price": "402659000000" }' --account_id nearlend.testnet
near view ctrl.nearlend.testnet get_price '{ "dtoken_address": "dweth.nearlend.testnet" }' --account_id nearlend.testnet


# 9,76$ WNEAR price with 8 decimals 
near call ctrl.nearlend.testnet set_price '{ "dtoken_address": "dwnear.nearlend.testnet", "price": "976000000" }' --account_id nearlend.testnet
near view ctrl.nearlend.testnet get_price '{ "dtoken_address": "dwnear.nearlend.testnet" }' --account_id nearlend.testnet