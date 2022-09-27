use near_sdk::json_types::U128;
use near_sdk::serde_json;
use near_sdk::serde_json::json;
use workspaces::network::Sandbox;
use workspaces::{Account, Worker};

const MARKET_WASM: &str = "../../res/dtoken.wasm";
const UNDERLYING_WASM: &str = "../../res/test_utoken.wasm";
const CONTROLLER_WASM: &str = "../../res/controller.wasm";

pub async fn deploy_underlying(
    owner: &Account,
    worker: &Worker<Sandbox>,
) -> Result<workspaces::Contract, workspaces::error::Error> {
    let wasm = std::fs::read(UNDERLYING_WASM);
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

pub async fn deploy_market(
    owner: &Account,
    worker: &Worker<Sandbox>,
    underlying_token: &Account,
    controller: &Account,
) -> Result<workspaces::Contract, workspaces::error::Error> {
    let wasm = std::fs::read(MARKET_WASM);
    let market = worker.dev_deploy(&wasm.unwrap()).await?;

    let _ = market
        .call("new_with_config")
        .args_json(
            json!({ "owner_id":  owner.id(), "underlying_token_id": underlying_token.id(),
                "controller_account_id": controller.id(),
                "initial_exchange_rate":"1000000000000000000000000",
                "interest_rate_model":{
                    "kink":"650000000000000000000000",
                    "multiplier_per_block":"3044140030441400",
                    "base_rate_per_block":"0",
                    "jump_multiplier_per_block":"38051750380517500",
                    "reserve_factor":"10000000000000000000000"
                }
            }),
        )
        .max_gas()
        .transact()
        .await?;

    let total_reserves: U128 = worker
        .view(
            market.id(),
            "view_total_reserves",
            serde_json::json!({}).to_string().into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(total_reserves, U128(0));

    Ok(market)
}

pub async fn deploy_controller(
    owner: &Account,
    worker: &Worker<Sandbox>,
) -> Result<workspaces::Contract, workspaces::error::Error> {
    let wasm = std::fs::read(CONTROLLER_WASM);
    let controller = worker.dev_deploy(&wasm.unwrap()).await?;
    let oracle = worker.dev_create_account().await?;

    let _ = controller
        .call("new_with_config")
        .args_json(json!({
        "owner_id": owner.id(),
        "oracle_account_id":oracle.id()
        }))
        .max_gas()
        .transact()
        .await?;

    Ok(controller)
}
