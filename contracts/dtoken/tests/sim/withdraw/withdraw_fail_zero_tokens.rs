use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_dtoken, initialize_utoken,
    mint_tokens, new_user, set_price, supply, view_balance, withdraw,
};
use controller::ActionType::Supply;
use dtoken::InterestRateModel;
use general::Price;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: Balance = 100;
const START_PRICE: Balance = 10000;

fn withdraw_fail_zero_tokens_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let weth = initialize_utoken(&root);
    let controller = initialize_controller(&root);
    let (_, weth_market) = initialize_dtoken(
        &root,
        weth.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
    );

    mint_tokens(&weth, weth_market.account_id(), U128(100));
    mint_tokens(&weth, user.account_id(), U128(WETH_AMOUNT));

    add_market(
        &controller,
        weth.account_id(),
        weth_market.account_id(),
        "weth".to_string(),
    );

    set_price(
        &controller,
        weth_market.account_id(),
        &Price {
            ticker_id: "weth".to_string(),
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    supply(&user, &weth, weth_market.account_id(), WETH_AMOUNT).assert_success();

    (weth_market, controller, weth, user)
}

#[test]
fn scenario_withdraw_fail_zero_tokens() {
    let (weth_market, controller, weth, user) = withdraw_fail_zero_tokens_fixture();

    let result = withdraw(&user, &weth_market, 0);
    assert_failure(result, "Amount should be a positive number");

    let user_supply_balance: Balance =
        view_balance(&controller, Supply, user.account_id(), weth_market.account_id());
    assert_eq!(user_supply_balance, WETH_AMOUNT, "Balance should be 0");

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 0);
}
