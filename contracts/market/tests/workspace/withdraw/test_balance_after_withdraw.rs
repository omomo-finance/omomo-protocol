use crate::utils::*;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use workspaces::network::Sandbox;
use workspaces::{Account, Worker};

const DECIMALS: u8 = 6;

async fn withdraw_fixture(
    owner: &Account,
    user: &Account,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<
    (
        workspaces::Contract,
        workspaces::Contract,
        workspaces::Contract,
    ),
    anyhow::Error,
> {
    ////////////////////////////////////////////////////////////////////////////
    // Stage 1: Deploy contracts such as underlying, controller, and markets
    ////////////////////////////////////////////////////////////////////////////

    let underlying = deploy_underlying(owner, worker, DECIMALS).await?;
    let controller = deploy_controller(owner, worker).await?;
    let market = deploy_market(
        owner,
        worker,
        underlying.as_account(),
        DECIMALS,
        controller.as_account(),
    )
    .await?;

    let contract_ft_balance_of: U128 = worker
        .view(
            market.id(),
            "ft_balance_of",
            json!({
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
            json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_balance_field, contract_ft_balance_of,
        "Balances should match"
    );

    ////////////////////////////////////////////////////////////////////////////////////////////
    // Stage 2: Deposit the storage for contract, mint and fund with reserve underlying contract
    ////////////////////////////////////////////////////////////////////////////////////////////

    let _ = underlying
        .call("storage_deposit")
        .args_json(json!({
            "account_id": market.id()
        }))
        .max_gas()
        .deposit(25 * 10u128.pow(23))
        .transact()
        .await?;

    let _ = underlying
        .call("mint")
        .args_json(json!({
            "account_id": user.id(),
            "amount": U128::from(2000 * 10_u128.pow(6))
        }))
        .max_gas()
        .transact()
        .await?;

    let _ = underlying
        .call("mint")
        .args_json(json!({
            "account_id": owner.id(),
            "amount": U128::from(2000 * 10_u128.pow(6))
        }))
        .max_gas()
        .transact()
        .await?;

    let _ = owner
        .call(underlying.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": market.id(),
            "amount": U128::from(1000 * 10_u128.pow(6)),
            "msg": "\"Reserve\""
        }))
        .max_gas()
        .deposit(1)
        .transact()
        .await?;

    ////////////////////////////////////////////////////////////////////////////////////////////
    // Stage 3: Check corresponding field after reserve
    ////////////////////////////////////////////////////////////////////////////////////////////

    let total_reserves_after_reserve: U128 = worker
        .view(
            market.id(),
            "view_total_reserves",
            json!({}).to_string().into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        total_reserves_after_reserve,
        U128::from(1000 * 10_u128.pow(24))
    );

    let contract_underlying_ft_balance_of_after_reserve: U128 = worker
        .view(
            underlying.id(),
            "ft_balance_of",
            json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_underlying_ft_balance_of_after_reserve,
        U128(1000 * 10_u128.pow(6))
    );

    let contract_market_balance_field_after_reserve: U128 = worker
        .view(
            market.id(),
            "view_contract_balance",
            json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_market_balance_field_after_reserve,
        U128(1000 * 10_u128.pow(24))
    );

    assert_ne!(
        contract_balance_field,
        contract_market_balance_field_after_reserve
    );

    ////////////////////////////////////////////////////////////////////////////////////////////
    // Stage 3: Register market on controller
    ////////////////////////////////////////////////////////////////////////////////////////////

    let _ = controller
        .call("add_market")
        .args_json(json!({
            "asset_id": underlying.id(),
            "dtoken": market.id(),
            "ticker_id": "weth",
            "ltv": "0.4",
            "lth": "0.8"
        }))
        .max_gas()
        .transact()
        .await?;

    let _ = controller
        .call("upsert_price")
        .args_json(json!({
            "dtoken_id": market.id(),
            "price": {
            "ticker_id": "weth".to_string(),
            "value": U128(2000),
            "volatility": U128(100),
            "fraction_digits": 4,
        }}))
        .max_gas()
        .transact()
        .await?;

    Ok((underlying, market, controller))
}

#[tokio::test]
async fn test_successful_withdraw() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account()?;
    let user = worker.dev_create_account().await?;
    let (underlying, market, _) = withdraw_fixture(&owner, &user, &worker).await?;

    let _a = user
        .call(underlying.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": market.id(),
            "amount": U128::from(1000 * 10_u128.pow(6)),
            "msg": "\"Supply\"",
        }))
        .max_gas()
        .deposit(1)
        .transact()
        .await?;

    let contract_underlying_ft_balance_of_after_supply: U128 = worker
        .view(
            underlying.id(),
            "ft_balance_of",
            json!({
                "account_id": user.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_underlying_ft_balance_of_after_supply,
        U128(1000 * 10_u128.pow(6))
    );

    let contract_market_ft_balance_of_after_supply: U128 = worker
        .view(
            market.id(),
            "ft_balance_of",
            json!({
                "account_id": user.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_market_ft_balance_of_after_supply,
        U128(1000 * 10_u128.pow(24))
    );

    let contract_ft_balance_of_before_withdraw: U128 = worker
        .view(
            underlying.id(),
            "ft_balance_of",
            json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_ft_balance_of_before_withdraw,
        U128(2000 * 10_u128.pow(6))
    );

    let contract_balance_field_before_withdraw: U128 = worker
        .view(
            market.id(),
            "view_contract_balance",
            json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_balance_field_before_withdraw,
        U128(2000 * 10_u128.pow(24))
    );

    let _ = user
        .call(market.id(), "withdraw")
        .args_json(json!({
            "amount": U128::from(500 * 10_u128.pow(24)),
        }))
        .max_gas()
        .transact()
        .await?;

    let contract_ft_balance_of_market_after_withdraw: U128 = worker
        .view(
            underlying.id(),
            "ft_balance_of",
            json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_ft_balance_of_market_after_withdraw,
        U128(1500 * 10_u128.pow(6))
    );

    let contract_ft_balance_of_user_after_withdraw: U128 = worker
        .view(
            underlying.id(),
            "ft_balance_of",
            json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_ft_balance_of_user_after_withdraw,
        U128(1500 * 10_u128.pow(6))
    );

    let contract_balance_field_after_withdraw: U128 = worker
        .view(
            market.id(),
            "view_contract_balance",
            json!({
                "account_id": market.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_balance_field_after_withdraw,
        U128(1500 * 10_u128.pow(24))
    );

    let contract_market_ft_balance_of_after_withdraw: U128 = worker
        .view(
            market.id(),
            "ft_balance_of",
            json!({
                "account_id": user.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_market_ft_balance_of_after_withdraw,
        U128(500 * 10_u128.pow(24))
    );
    Ok(())
}
