use near_sdk::serde_json::json;
use workspaces::network::Sandbox;
use workspaces::{Account, Worker};

const LEVERAGE_TRADING_WASM: &str = "./res/leverage_trading.wasm";
const TEST_UTULEN_WASM: &str = "./res/test_utoken.wasm";

pub async fn deploy_leverage_trading(
    owner: &Account,
    worker: &Worker<Sandbox>,
) -> Result<workspaces::Contract, workspaces::error::Error> {
    let wasm = std::fs::read(LEVERAGE_TRADING_WASM);
    let leverage_trading = worker.dev_deploy(&wasm.unwrap()).await?;

    let _ = leverage_trading
        .call("new_with_config")
        .args_json(json!({"owner_id": owner.id(), "oracle_account_id": owner.id()}))
        .max_gas()
        .transact()
        .await?;

    Ok(leverage_trading)
}

pub async fn deploy_test_utoken(
    owner: &Account,
    worker: &Worker<Sandbox>,
) -> Result<workspaces::Contract, workspaces::error::Error> {
    let wasm = std::fs::read(TEST_UTULEN_WASM);
    let underlying = worker.dev_deploy(&wasm.unwrap()).await?;

    let _ = underlying
        .call("new_default_meta")
        .args_json(json!({ "owner_id": owner.id(),
        "name": "Wrapped Ethereum",
        "symbol": "WETH",
        "total_supply": "1000000000000000000000000000"
                }))
        .max_gas()
        .transact()
        .await?;

    Ok(underlying)
}
