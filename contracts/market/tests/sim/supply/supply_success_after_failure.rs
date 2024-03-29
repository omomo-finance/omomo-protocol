use market::InterestRateModel;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

use controller::ActionType::Supply;
use general::Price;

use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_two_dtokens,
    initialize_two_utokens, mint_tokens, new_user, set_price, supply, view_balance,
};

const WNEAR_BALANCE: Balance = 50;
const WETH_BALANCE: Balance = 100;
const SUPPLY_WETH_AMOUNT: Balance = 100;
const SUPPLY_WNEAR_AMOUNT: Balance = 0;
const START_PRICE: Balance = 10000;

fn supply_success_after_failure_fixture() -> (
    ContractAccount<market::ContractContract>,
    ContractAccount<market::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<mock_token::ContractContract>,
    ContractAccount<mock_token::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (weth, wnear) = initialize_two_utokens(&root);
    let controller = initialize_controller(&root);
    let (_, weth_market, wnear_market) = initialize_two_dtokens(
        &root,
        weth.account_id(),
        wnear.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
        InterestRateModel::default(),
    );

    mint_tokens(&wnear, wnear_market.account_id(), U128(100));
    mint_tokens(&wnear, user.account_id(), U128(WNEAR_BALANCE));
    mint_tokens(&weth, weth_market.account_id(), U128(100));
    mint_tokens(&weth, user.account_id(), U128(WETH_BALANCE));

    add_market(
        &controller,
        weth.account_id(),
        weth_market.account_id(),
        "weth".to_string(),
    );

    add_market(
        &controller,
        wnear.account_id(),
        wnear_market.account_id(),
        "weth".to_string(),
    );

    set_price(
        &controller,
        wnear_market.account_id(),
        &Price {
            ticker_id: "wnear".to_string(),
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
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

    (weth_market, wnear_market, controller, weth, wnear, user)
}

#[test]
fn scenario_supply_success_after_failure() {
    let (weth_market, wnear_market, controller, weth, wnear, user) =
        supply_success_after_failure_fixture();

    let result = supply(
        &user,
        &wnear,
        wnear_market.account_id(),
        SUPPLY_WNEAR_AMOUNT,
    );
    assert_failure(result, "The amount should be a positive number");

    supply(&user, &weth, weth_market.account_id(), SUPPLY_WETH_AMOUNT).assert_success();

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance, U128(0), "User balance should be 0");

    let user_balance: Balance = view_balance(
        &controller,
        Supply,
        user.account_id(),
        weth_market.account_id(),
    );
    assert_eq!(
        user_balance, SUPPLY_WETH_AMOUNT,
        "Balance on controller should be {}",
        SUPPLY_WETH_AMOUNT
    );
}
