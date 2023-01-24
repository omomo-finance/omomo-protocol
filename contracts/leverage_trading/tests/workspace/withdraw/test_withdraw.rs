use crate::utils::*;
use leverage_trading::Actions;
use near_sdk::json_types::U128;
use near_sdk::require;
use near_sdk::serde_json::json;
use workspaces::network::Sandbox;
use workspaces::{Account, Worker};

async fn withdraw_fixture(
    owner: &Account,
    user: &Account,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<(workspaces::Contract, workspaces::Contract), anyhow::Error> {
    ////////////////////////////////////////////////////////////////////////////
    // Stage 1: Deploy contracts such as leverage_trading and mock_token
    ////////////////////////////////////////////////////////////////////////////

    let leverage_trading = deploy_leverage_trading(owner, worker).await?;
    let mock_token = deploy_mock_token(owner, worker).await?;

    ////////////////////////////////////////////////////////////////////////////
    // Stage 2: Adding a marker to a contract leverage_trading.
    ////////////////////////////////////////////////////////////////////////////

    let _ = leverage_trading
        .call("add_pair")
        .args_json(json!({"pair_data": {
            "sell_ticker_id": mock_token.id().to_string(),
            "sell_token": mock_token.id(),
            "sell_token_decimals": 24,
            "sell_token_market": mock_token.id(),
            "buy_ticker_id": mock_token.id().to_string(),
            "buy_token": mock_token.id(),
            "buy_token_decimals": 24,
            "pool_id": mock_token.id().to_string(),
            "max_leverage": "2500000000000000000000000",
            "swap_fee": "300000000000000000000"
        }}))
        .max_gas()
        .transact()
        .await?;

    ////////////////////////////////////////////////////////////////////////////
    // Stage 3: Deposit the storage for contract leverage_trading,
    // mint for user and user transfer to a contract leverage_trading
    ////////////////////////////////////////////////////////////////////////////

    let _ = mock_token
        .call("storage_deposit")
        .args_json(json!({
            "account_id": leverage_trading.id()
        }))
        .max_gas()
        .deposit(25 * 10_u128.pow(23))
        .transact()
        .await?;

    let _ = mock_token
        .call("mint")
        .args_json(json!({
            "account_id": user.id(),
            "amount": U128::from(25 * 10_u128.pow(26))
        }))
        .max_gas()
        .transact()
        .await?;

    let user_ft_balance_of_after_mint: U128 = worker
        .view(
            mock_token.id(),
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
        user_ft_balance_of_after_mint,
        U128::from(25 * 10_u128.pow(26)),
        "user_ft_balance_of_after_mint"
    );

    let token: near_sdk::AccountId = mock_token.id().to_string().parse().unwrap();
    let action = Actions::Deposit { token };

    let _ = user
        .call(mock_token.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": leverage_trading.id(),
            "amount": U128::from(12 * 10_u128.pow(26)),
            "msg": near_sdk::serde_json::to_string(&action).unwrap()
        }))
        .max_gas()
        .deposit(1)
        .transact()
        .await?;

    ////////////////////////////////////////////////////////////////////////////
    // Stage 4: Check balance after reserve ft_transfer_call
    ////////////////////////////////////////////////////////////////////////////

    let contract_ft_balance_of_after_transfer_call: U128 = worker
        .view(
            mock_token.id(),
            "ft_balance_of",
            json!({
                "account_id": leverage_trading.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(
        contract_ft_balance_of_after_transfer_call,
        U128::from(12 * 10_u128.pow(26)),
    );

    let user_ft_balance_of_after_transfer_call: U128 = worker
        .view(
            mock_token.id(),
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
        user_ft_balance_of_after_transfer_call,
        U128::from(13 * 10_u128.pow(26))
    );

    Ok((mock_token, leverage_trading))
}

#[tokio::test]
async fn test_successful_withdraw() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account()?;
    let user = worker.dev_create_account().await?;
    let (mock_token, leverage_trading) = withdraw_fixture(&owner, &user, &worker).await?;

    let contract_ft_balance_of_before_withdraw: U128 = worker
        .view(
            mock_token.id(),
            "ft_balance_of",
            json!({
                "account_id": leverage_trading.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let user_balance_of_before_withdraw: U128 = worker
        .view(
            leverage_trading.id(),
            "balance_of",
            json!({
                "account_id": user.id(),
                "token": mock_token.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let _ = user
        .call(leverage_trading.id(), "withdraw")
        .args_json(json!({
            "token": mock_token.id(),
            "amount": U128::from(6 * 10_u128.pow(26)),
        }))
        .max_gas()
        .transact()
        .await?;

    let contract_ft_balance_of_after_withdraw: U128 = worker
        .view(
            mock_token.id(),
            "ft_balance_of",
            json!({
                "account_id": leverage_trading.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let user_balance_of_after_withdraw: U128 = worker
        .view(
            leverage_trading.id(),
            "balance_of",
            json!({
                "account_id": user.id(),
                "token": mock_token.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let user_ft_balance_of_after_withdraw: U128 = worker
        .view(
            mock_token.id(),
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
        contract_ft_balance_of_before_withdraw,
        user_balance_of_before_withdraw
    );
    assert_eq!(
        contract_ft_balance_of_after_withdraw,
        user_balance_of_after_withdraw
    );
    assert_eq!(
        user_ft_balance_of_after_withdraw,
        U128(19 * 10_u128.pow(26))
    );

    Ok(())
}

#[tokio::test]
async fn test_withdraw_with_more_than_a_balance() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account()?;
    let user = worker.dev_create_account().await?;
    let (mock_token, leverage_trading) = withdraw_fixture(&owner, &user, &worker).await?;

    let contract_ft_balance_of_before_withdraw: U128 = worker
        .view(
            mock_token.id(),
            "ft_balance_of",
            json!({
                "account_id": leverage_trading.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let user_balance_of_before_withdraw: U128 = worker
        .view(
            leverage_trading.id(),
            "balance_of",
            json!({
                "account_id": user.id(),
                "token": mock_token.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let withdraw = user
        .call(leverage_trading.id(), "withdraw")
        .args_json(json!({
            "token": mock_token.id(),
            "amount": U128::from(85 * 10_u128.pow(26)),
        }))
        .max_gas()
        .transact()
        .await?
        .into_result();

    require!(withdraw.is_err());

    let contract_ft_balance_of_after_withdraw: U128 = worker
        .view(
            mock_token.id(),
            "ft_balance_of",
            json!({
                "account_id": leverage_trading.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let user_balance_of_after_withdraw: U128 = worker
        .view(
            leverage_trading.id(),
            "balance_of",
            json!({
                "account_id": user.id(),
                "token": mock_token.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let user_ft_balance_of_after_withdraw: U128 = worker
        .view(
            mock_token.id(),
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
        contract_ft_balance_of_before_withdraw,
        user_balance_of_before_withdraw
    );
    assert_eq!(
        contract_ft_balance_of_after_withdraw,
        user_balance_of_after_withdraw
    );
    assert_eq!(
        user_ft_balance_of_after_withdraw,
        U128(13 * 10_u128.pow(26))
    );

    Ok(())
}
