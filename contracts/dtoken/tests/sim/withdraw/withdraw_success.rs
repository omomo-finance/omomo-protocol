use crate::utils::{
    add_market, initialize_controller, initialize_dtoken, initialize_utoken, mint_tokens, new_user,
    set_price, supply, view_balance, withdraw,
};
use controller::ActionType::Supply;
use dtoken::InterestRateModel;
use general::Price;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};
use general::wbalance::WBalance;

const WETH_AMOUNT: Balance = 100;
const START_PRICE: Balance = 10000;

fn withdraw_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let weth = initialize_utoken(&root);
    let controller = initialize_controller(&root);
    let dweth = initialize_dtoken(
        &root,
        weth.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
    );

    mint_tokens(&weth, dweth.account_id(), U128(100));
    mint_tokens(&weth, user.account_id(), U128(WETH_AMOUNT));

    add_market(
        &controller,
        weth.account_id(),
        dweth.account_id(),
        "weth".to_string(),
    );

    set_price(
        &controller,
        dweth.account_id(),
        &Price {
            ticker_id: "weth".to_string(),
            value: WBalance::from(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    supply(&user, &weth, dweth.account_id(), WETH_AMOUNT).assert_success();

    (dweth, controller, weth, user)
}

#[test]
fn scenario_withdraw() {
    let (dweth, controller, weth, user) = withdraw_fixture();

    withdraw(&user, &dweth, WETH_AMOUNT).assert_success();

    let user_supply_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dweth.account_id());
    assert_eq!(
        user_supply_balance,
        WETH_AMOUNT / 2,
        "Balance should be {}",
        WETH_AMOUNT / 2
    );

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, WETH_AMOUNT / 2);
}
