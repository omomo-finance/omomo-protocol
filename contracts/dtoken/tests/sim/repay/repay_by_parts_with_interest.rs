use crate::utils::{
    add_market, borrow, initialize_controller, initialize_three_dtokens, initialize_three_utokens,
    mint_and_reserve, mint_tokens, new_user, repay, repay_info, set_price, supply, view_balance,
};
use controller::ActionType::Borrow;
use dtoken::{InterestRateModel, WRatio};
use general::{ratio::Ratio, Price};
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: Balance = 60;
const WNEAR_AMOUNT: Balance = 70;
const WBTC_AMOUNT: Balance = 100;
const WETH_BORROW: Balance = 30;
const WNEAR_BORROW: Balance = 40;
const START_BALANCE: Balance = 200;
const START_PRICE: Balance = 10000;
const FIRST_PART_TO_REPAY: Balance = 10;

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
    let interest_rate_model = InterestRateModel {
        kink: WRatio::from(650000000000000000000000),
        base_rate_per_block: WRatio::from(0),
        multiplier_per_block: WRatio::from(62800000000000000),
        jump_multiplier_per_block: WRatio::from(761),
        reserve_factor: WRatio::from(1000000000),
    };
    let (droot, dweth, dwnear, dwbtc) = initialize_three_dtokens(
        &root,
        weth.account_id(),
        wnear.account_id(),
        wbtc.account_id(),
        controller.account_id(),
        interest_rate_model.clone(),
        interest_rate_model.clone(),
        interest_rate_model,
    );

    mint_and_reserve(&droot, &weth, &dweth, WETH_AMOUNT);
    mint_and_reserve(&droot, &wnear, &dwnear, WNEAR_AMOUNT);
    mint_and_reserve(&droot, &wbtc, &dwbtc, WBTC_AMOUNT);

    let mint_amount = U128(START_BALANCE);
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
fn repay_by_parts_with_interest() {
    let (dwnear, controller, wnear, user) = repay_fixture();

    let dwnear_balance: U128 = view!(wnear.ft_balance_of(dwnear.account_id())).unwrap_json();

    let repay_info = repay_info(&user, &dwnear, dwnear_balance);
    println!("{:?}", repay_info);

    let repay_amount = Balance::from(repay_info.total_amount);

    repay(&user, dwnear.account_id(), &wnear, FIRST_PART_TO_REPAY).assert_success();
    let dwnear_balance: U128 = view!(wnear.ft_balance_of(dwnear.account_id())).unwrap_json();
    let exchange_rate: Ratio = view!(dwnear.view_exchange_rate(dwnear_balance)).unwrap_json();
    assert_eq!(exchange_rate, Ratio::one(), "xrate should be 1.0");

    let user_balance: U128 = view!(wnear.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance.0,
        START_BALANCE - WNEAR_AMOUNT + WNEAR_BORROW - FIRST_PART_TO_REPAY,
        "Repay was partially done, user balance should be {}",
        START_BALANCE - WNEAR_AMOUNT + WNEAR_BORROW - FIRST_PART_TO_REPAY
    );

    let user_balance: Balance = view!(dwnear.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        WNEAR_BORROW - FIRST_PART_TO_REPAY,
        "Borrow balance on dtoken should be {}",
        WNEAR_BORROW - FIRST_PART_TO_REPAY
    );

    let user_balance: Balance =
        view_balance(&controller, Borrow, user.account_id(), dwnear.account_id());
    assert_eq!(
        user_balance,
        WNEAR_BORROW - FIRST_PART_TO_REPAY,
        "Borrow balance on controller should be {}",
        WNEAR_BORROW - FIRST_PART_TO_REPAY
    );

    repay(
        &user,
        dwnear.account_id(),
        &wnear,
        repay_amount - FIRST_PART_TO_REPAY,
    )
    .assert_success();

    let balance_after_first_repay =
        START_BALANCE - WNEAR_AMOUNT + WNEAR_BORROW - FIRST_PART_TO_REPAY;

    let user_balance: U128 = view!(wnear.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance.0,
        balance_after_first_repay - (repay_amount - FIRST_PART_TO_REPAY),
        "Repay was fully done, user balance should be {}",
        balance_after_first_repay - (repay_amount - FIRST_PART_TO_REPAY)
    );

    let user_balance: Balance = view!(dwnear.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: Balance =
        view_balance(&controller, Borrow, user.account_id(), dwnear.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
    let dwnear_balance: U128 = view!(wnear.ft_balance_of(dwnear.account_id())).unwrap_json();
    let exchange_rate: Ratio = view!(dwnear.view_exchange_rate(dwnear_balance)).unwrap_json();
    assert_eq!(exchange_rate, Ratio::one(), "xrate should be 1.0");
}
