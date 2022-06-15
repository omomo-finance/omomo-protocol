use dtoken::InterestRateModel;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

use controller::ActionType::Supply;
use general::Price;

use crate::utils::{
    add_market, initialize_controller, initialize_dtoken, initialize_utoken, mint_tokens, new_user,
    set_price, supply, view_balance,
};

const SUPPLY_AMOUNT: Balance = 100;
const START_PRICE: Balance = 10000;

fn supply_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let weth = initialize_utoken(&root);
    let controller = initialize_controller(&root);
    let (_, dweth) = initialize_dtoken(
        &root,
        weth.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
    );

    mint_tokens(&weth, dweth.account_id(), U128(100));
    mint_tokens(&weth, user.account_id(), U128(SUPPLY_AMOUNT));

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
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    (dweth, controller, weth, user)
}

#[test]
fn scenario_supply() {
    let (dweth, controller, weth, user) = supply_fixture();

    supply(&user, &weth, dweth.account_id(), SUPPLY_AMOUNT).assert_success();

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance, U128(0), "User balance should be 0");

    let user_dtoken_balance: U128 = view!(dweth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_dtoken_balance, U128(SUPPLY_AMOUNT), "User dtoken balance should be {}", SUPPLY_AMOUNT);

    let user_balance: Balance =
        view_balance(&controller, Supply, user.account_id(), dweth.account_id());
    assert_eq!(
        user_balance, SUPPLY_AMOUNT,
        "Balance on controller should be {}",
        SUPPLY_AMOUNT
    );
}
