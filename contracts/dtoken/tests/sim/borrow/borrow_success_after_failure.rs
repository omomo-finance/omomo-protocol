use crate::utils::{
    add_market, assert_failure, borrow, initialize_controller, initialize_three_dtokens,
    initialize_three_utokens, mint_tokens, new_user, set_price, supply, view_balance,
};
use controller::ActionType::Borrow;
use dtoken::InterestRateModel;
use general::wbalance::WBalance;
use general::Price;
use near_sdk::json_types::U128;
use near_sdk::Balance;
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: Balance = 10;
const WNEAR_AMOUNT: Balance = 10;
const BORROW_AMOUNT: Balance = 11;
const WBTC_START_BALANCE: Balance = 100;
const START_BALANCE: Balance = 100;
const START_PRICE: Balance = 10000;

fn borrow_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (weth, wnear, wbtc) = initialize_three_utokens(&root);
    let controller = initialize_controller(&root);
    let (dweth, dwnear, dwbtc) = initialize_three_dtokens(
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
    mint_tokens(&weth, dweth.account_id(), mint_amount);
    mint_tokens(&wnear, dwnear.account_id(), mint_amount);
    mint_tokens(&wbtc, dwbtc.account_id(), U128(WBTC_START_BALANCE));
    mint_tokens(&weth, user.account_id(), U128(WETH_AMOUNT));
    mint_tokens(&wnear, user.account_id(), U128(WNEAR_AMOUNT));
    mint_tokens(&wbtc, user.account_id(), U128(0));

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
        dweth.account_id(),
        &Price {
            ticker_id: "weth".to_string(),
            value: WBalance::from(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    set_price(
        &controller,
        dwnear.account_id(),
        &Price {
            ticker_id: "wnear".to_string(),
            value: WBalance::from(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    set_price(
        &controller,
        dwbtc.account_id(),
        &Price {
            ticker_id: "wbtc".to_string(),
            value: WBalance::from(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    supply(&user, &weth, dweth.account_id(), WETH_AMOUNT).assert_success();

    supply(&user, &wnear, dwnear.account_id(), WNEAR_AMOUNT).assert_success();

    (dwbtc, controller, wbtc, user)
}

#[test]
fn scenario_borrow_zero_tokens() {
    let (dwbtc, controller, wbtc, user) = borrow_fixture();

    let result = borrow(&user, &dwbtc, 0);
    assert_failure(result, "Amount should be a positive number");

    borrow(&user, &dwbtc, BORROW_AMOUNT).assert_success();

    let user_balance: Balance =
        view_balance(&controller, Borrow, user.account_id(), dwbtc.account_id());
    assert_eq!(
        user_balance, BORROW_AMOUNT,
        "User borrow balance on controller should be {}",
        BORROW_AMOUNT
    );

    let user_balance: Balance = view!(dwbtc.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance, BORROW_AMOUNT,
        "User borrow balance on dtoken should be {}",
        BORROW_AMOUNT
    );

    let user_balance: U128 = view!(wbtc.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        U128(BORROW_AMOUNT),
        "User utoken balance should be {}",
        BORROW_AMOUNT
    );

    let dtoken_balance: U128 = view!(wbtc.ft_balance_of(dwbtc.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        U128(WBTC_START_BALANCE - BORROW_AMOUNT),
        "Dtoken balance on utoken should be {}",
        WBTC_START_BALANCE - BORROW_AMOUNT
    );
}
