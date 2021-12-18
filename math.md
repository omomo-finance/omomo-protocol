near delete math.nearlend.testnet nearlend.testnet
near create-account math.nearlend.testnet --masterAccount nearlend.testnet
near deploy math.nearlend.testnet --wasmFile ./target/wasm32-unknown-unknown/release/interest_rate_model.wasm