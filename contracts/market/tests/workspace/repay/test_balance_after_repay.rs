use crate::utils::*;
use market::RepayInfo;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{require, Balance};
use workspaces::network::Sandbox;
use workspaces::{Account, Worker};

// 10 Near
const BORROW_AMOUNT: Balance = 10000000000000000000000000;

async fn repay_fixture(
    owner: &Account,
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

    let underlying = deploy_underlying(owner, worker).await?;
    let controller = deploy_controller(owner, worker).await?;
    let market = deploy_market(
        owner,
        worker,
        underlying.as_account(),
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
            "account_id": owner.id(),
            "amount": U128::from(200000000000000000000000000)
        }))
        .max_gas()
        .transact()
        .await?;

    let _ = owner
        .call(underlying.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": market.id(),
            "amount": U128::from(100000000000000000000000000),
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
        U128::from(100000000000000000000000000)
    );

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

    assert_eq!(
        contract_balance_field_after_reserve,
        contract_ft_balance_of_after_reserve
    );
    assert_ne!(contract_ft_balance_of, contract_ft_balance_of_after_reserve);
    assert_ne!(contract_balance_field, contract_balance_field_after_reserve);

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

    let _ = owner
        .call(underlying.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": market.id(),
            "amount": U128::from(100000000000000000000000000),
            "msg": "\"Supply\""
        }))
        .max_gas()
        .deposit(1)
        .transact()
        .await?;

    let _ = owner
        .call(market.id(), "borrow")
        .args_json(json!({
            "amount": U128::from(BORROW_AMOUNT),
        }))
        .max_gas()
        .transact()
        .await?;

    Ok((underlying, controller, market))
}

#[tokio::test]
async fn test_repay_part_of_accumulated_interest() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account()?;
    let (underlying, _, market) = repay_fixture(&owner, &worker).await?;

    // passing 100 blocks
    let blocks_to_advance = 100;
    worker.fast_forward(blocks_to_advance).await?;

    let contract_ft_balance_of_before_repay: U128 = worker
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

    let contract_balance_field_before_repay: U128 = worker
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
        contract_balance_field_before_repay, contract_ft_balance_of_before_repay,
        "Corresponding fields should match"
    );

    let repay_info: RepayInfo = worker
        .view(
            market.id(),
            "view_repay_info",
            json!({
                "user_id": owner.id(),
                "ft_balance": contract_balance_field_before_repay})
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let _repay_result = owner
        .call(underlying.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": market.id(),
            "amount": U128::from(repay_info.accumulated_interest.0 / 2) ,
            "msg": "\"Repay\""
        }))
        .max_gas()
        .deposit(1)
        .transact()
        .await?;

    let contract_ft_balance_of_after_repay: U128 = worker
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

    let contract_balance_field_after_repay: U128 = worker
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

    assert!(contract_balance_field_before_repay.0 < contract_balance_field_after_repay.0);
    assert!(contract_ft_balance_of_before_repay.0 < contract_balance_field_after_repay.0);

    assert_eq!(
        contract_ft_balance_of_before_repay.0 + (repay_info.accumulated_interest.0 / 2),
        contract_ft_balance_of_after_repay.0
    );
    assert_eq!(
        contract_balance_field_before_repay.0 + (repay_info.accumulated_interest.0 / 2),
        contract_balance_field_after_repay.0
    );

    assert_eq!(
        contract_balance_field_after_repay,
        contract_ft_balance_of_after_repay
    );

    Ok(())
}

#[tokio::test]
async fn test_repay_more() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account()?;
    let (underlying, _, market) = repay_fixture(&owner, &worker).await?;

    // passing 1000 blocks
    let blocks_to_advance = 1000;
    worker.fast_forward(blocks_to_advance).await?;

    let contract_ft_balance_of_before_repay: U128 = worker
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

    let contract_balance_field_before_repay: U128 = worker
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
        contract_ft_balance_of_before_repay, contract_ft_balance_of_before_repay,
        "Corresponding fields should match"
    );

    let repay_info_before_repay: RepayInfo = worker
        .view(
            market.id(),
            "view_repay_info",
            json!({
                "user_id": owner.id(),
                "ft_balance": contract_balance_field_before_repay})
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_ne!(repay_info_before_repay.accumulated_interest.0, 0);

    require!(owner
        .call(underlying.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": market.id(),
            "amount": U128::from(BORROW_AMOUNT * 2)  ,
            "msg": "\"Repay\""
        }))
        .max_gas()
        .deposit(1)
        .transact()
        .await?
        .is_success());

    // passing 10 blocks
    let blocks_to_advance = 10;
    worker.fast_forward(blocks_to_advance).await?;

    let contract_ft_balance_of_after_repay: U128 = worker
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

    let contract_balance_field_after_repay: U128 = worker
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
        contract_balance_field_after_repay, contract_ft_balance_of_after_repay,
        "Corresponding fields should match"
    );

    let repay_info_after_repay: RepayInfo = worker
        .view(
            market.id(),
            "view_repay_info",
            json!({
                "user_id": owner.id(),
                "ft_balance": contract_ft_balance_of_after_repay})
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(repay_info_after_repay.borrow_amount.0, 0);

    assert!(contract_balance_field_before_repay.0 < contract_balance_field_after_repay.0);
    assert!(contract_ft_balance_of_before_repay.0 < contract_balance_field_after_repay.0);

    Ok(())
}
