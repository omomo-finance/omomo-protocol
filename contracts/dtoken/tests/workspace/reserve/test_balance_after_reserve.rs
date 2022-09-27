use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use crate::utils::*;

#[tokio::test]
async fn test_balance_after_reserve() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account()?;

    ////////////////////////////////////////////////////////////////////////////
    // Stage 1: Deploy contracts such as underlying, controller, and markets
    ////////////////////////////////////////////////////////////////////////////


    let underlying = deploy_underlying(&owner, &worker).await?;
    let controller = deploy_controller(&owner, &worker).await?;
    let market = deploy_market(&owner, &worker, underlying.as_account(), controller.as_account()).await?;

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

    assert_eq!(contract_balance_field, contract_ft_balance_of, "Balances should match");

    ////////////////////////////////////////////////////////////////////////////////////////////
    // Stage 2: Deposit the storage for contract, mint and fund with reserve underlying contract
    ////////////////////////////////////////////////////////////////////////////////////////////


    let _ = underlying.call("storage_deposit").args_json(json!({
    "account_id": market.id()
})).max_gas()
        .deposit(25 * 10u128.pow(23))
        .transact()
        .await?;


    let _ = underlying.call("mint").args_json(json!({
    "account_id": owner.id(),
    "amount": U128::from(1000000000000000000000000000)
})).max_gas()

        .transact()
        .await?;


    let _ = owner.call(underlying.id(), "ft_transfer_call")
        .args_json(
            json!({
        "receiver_id": market.id(),
        "amount": U128::from(1000000000000000000000000000),
        "msg": "\"Reserve\""
    }))
        .max_gas()
        .deposit(1)
        .transact()
        .await?;

    ////////////////////////////////////////////////////////////////////////////////////////////
    // Stage 3: Check corresponding field afterwards
    ////////////////////////////////////////////////////////////////////////////////////////////


    let total_reserves_after_reserve: U128 = worker
        .view(
            market.id(),
            "view_total_reserves",
            json!({}).to_string().into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(total_reserves_after_reserve, U128::from(1000000000000000000000000000));

    let contract_ft_balance_of_after_reserve: U128 = worker
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

    let contract_balance_field_after_reserve: U128 = worker
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

    assert_eq!(contract_balance_field_after_reserve, contract_ft_balance_of_after_reserve);
    assert_ne!(contract_ft_balance_of, contract_ft_balance_of_after_reserve);
    assert_ne!(contract_balance_field, contract_balance_field_after_reserve);

    Ok(())
}
