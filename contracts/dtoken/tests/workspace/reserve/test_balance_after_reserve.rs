use near_sdk::json_types::U128;
use near_sdk::serde_json;
use near_sdk::serde_json::json;
use workspaces::network::Sandbox;
use workspaces::{Account, Worker};

const MARKET_WASM: &str = "../../res/dtoken.wasm";

async fn create_custom_market(
    owner: &Account,
    worker: &Worker<Sandbox>,
) -> Result<workspaces::Contract, workspaces::error::Error> {
    let underlying_token = worker.dev_create_account().await?;
    let controller = worker.dev_create_account().await?;

    let wasm = std::fs::read(MARKET_WASM);
    let market = worker.dev_deploy(&wasm.unwrap()).await?;

    let _ = market
        .call("new_with_config")
        .args_json(
            json!({ "owner_id":  owner.id(), "underlying_token_id": underlying_token.id(),
                "controller_account_id":controller.id(),
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

#[tokio::test]
async fn test_balance_after_reserve() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account()?;

    ///////////////////////////////////////////////////////////////////////////
    // Stage 1: Deploy relevant contracts such as Market
    ///////////////////////////////////////////////////////////////////////////

    let market = create_custom_market(&owner, &worker).await?;

    let _contract_ft_balance_of: U128 = worker
        .view(
            market.id(),
            "ft_balance_of",
            serde_json::json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let contract_balance_field: U128 = worker
        .view(
            market.id(),
            "view_contract_balance",
            serde_json::json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    dbg!(contract_balance_field);

    Ok(())
}
