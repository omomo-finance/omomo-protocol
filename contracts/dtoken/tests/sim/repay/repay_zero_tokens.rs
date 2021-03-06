use crate::utils::{
    add_market, assert_failure, borrow, initialize_controller, initialize_three_dtokens,
    initialize_three_utokens, mint_tokens, new_user, repay, set_price, supply, view_balance,
};
use controller::ActionType::Borrow;
use dtoken::InterestRateModel;
use general::Price;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: Balance = 60;
const WNEAR_AMOUNT: Balance = 70;
const WBTC_AMOUNT: Balance = 100;
const WETH_BORROW: Balance = 30;
const WNEAR_BORROW: Balance = 40;
const START_BALANCE: Balance = 100;
const START_PRICE: Balance = 10000;

fn repay_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (weth, wnear, wbtc) = initialize_three_utokens(&root);
    let controller = initialize_controller(&root);
    let (_, dweth, dwnear, dwbtc) = initialize_three_dtokens(
        &root,
        weth.account_id(),
        wnear.account_id(),
        wbtc.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
        InterestRateModel::default(),
        InterestRateModel::default(),
    );

    let mint_amount = U128(START_BALANCE);
    mint_tokens(&weth, dweth.account_id(), U128(WETH_AMOUNT));
    mint_tokens(&wnear, dwnear.account_id(), U128(WNEAR_AMOUNT));
    mint_tokens(&wbtc, dwbtc.account_id(), U128(WBTC_AMOUNT));
    mint_tokens(&weth, user.account_id(), mint_amount);
    mint_tokens(&wnear, user.account_id(), mint_amount);
    mint_tokens(&wbtc, user.account_id(), mint_amount);

    add_market(
        &controller,
        weth.account_id(),
        dweth.account_id(),
        "weth".to_string(),
    );

    add_market(
        &controller,
        wnear.account_id(),
        dwnear.account_id(),
        "wnear".to_string(),
    );

    add_market(
        &controller,
        wbtc.account_id(),
        dwbtc.account_id(),
        "wbtc".to_string(),
    );

    set_price(
        &controller,
        dwnear.account_id(),
        &Price {
            ticker_id: "wnear".to_string(),
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
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

    set_price(
        &controller,
        dwbtc.account_id(),
        &Price {
            ticker_id: "wbtc".to_string(),
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    supply(&user, &weth, dweth.account_id(), WETH_AMOUNT).assert_success();

    supply(&user, &wnear, dwnear.account_id(), WNEAR_AMOUNT).assert_success();

    borrow(&user, &dweth, WETH_BORROW).assert_success();

    borrow(&user, &dwnear, WNEAR_BORROW).assert_success();

    (dwnear, controller, wnear, user)
}

#[test]
fn scenario_repay_zero_tokens() {
    let (dwnear, controller, wnear, user) = repay_fixture();

    let result = repay(&user, dwnear.account_id(), &wnear, 0);

    assert_failure(result, "The amount should be a positive number");

    let user_balance: U128 = view!(wnear.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance.0,
        START_BALANCE - WNEAR_AMOUNT + WNEAR_BORROW,
        "Repay wasn`t done"
    );

    let user_balance: Balance = view!(dwnear.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance, WNEAR_BORROW,
        "Borrow balance on dtoken should be {}",
        WNEAR_BORROW
    );

    let user_balance: Balance =
        view_balance(&controller, Borrow, user.account_id(), dwnear.account_id());
    assert_eq!(
        user_balance, WNEAR_BORROW,
        "Borrow balance on controller should be {}",
        WNEAR_BORROW
    );
}
