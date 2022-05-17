use crate::utils::{
    add_market, borrow, initialize_controller, initialize_two_dtokens, initialize_two_utokens,
    liquidate, mint_tokens, new_user, set_price, supply, view_balance,
};
use controller::ActionType::{Borrow, Supply};
use dtoken::InterestRateModel;
use general::Price;
use near_sdk::json_types::U128;
use near_sdk::Balance;
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};
use general::wbalance::WBalance;

const BORROWER_SUPPLY: Balance = 60000;
const BORROWER_BORROW: Balance = 40000;
const MINT_BALANCE: Balance = 100000000000;
const START_PRICE: Balance = 2000;
const CHANGED_PRICE: Balance = 1200;

fn liquidation_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let borrower = new_user(&root, "borrower".parse().unwrap());
    let liquidator = new_user(&root, "liquidator".parse().unwrap());
    let (weth, wnear) = initialize_two_utokens(&root);
    let controller = initialize_controller(&root);
    let (dweth, dwnear) = initialize_two_dtokens(
        &root,
        weth.account_id(),
        wnear.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
        InterestRateModel::default(),
    );

    let mint_amount = U128(MINT_BALANCE);
    mint_tokens(&weth, dweth.account_id(), mint_amount);
    mint_tokens(&wnear, dwnear.account_id(), mint_amount);
    mint_tokens(&weth, borrower.account_id(), mint_amount);
    mint_tokens(&wnear, liquidator.account_id(), mint_amount);
    mint_tokens(&weth, liquidator.account_id(), mint_amount);
    mint_tokens(&wnear, borrower.account_id(), mint_amount);
    mint_tokens(&wnear, borrower.account_id(), mint_amount);

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

    supply(&borrower, &wnear, dwnear.account_id(), BORROWER_SUPPLY).assert_success();

    borrow(&borrower, &dweth, BORROWER_BORROW).assert_success();

    let user_balance: Balance =
        view!(dweth.get_account_borrows(borrower.account_id())).unwrap_json();
    assert_eq!(
        user_balance, BORROWER_BORROW,
        "Borrow balance on dtoken should be {}",
        BORROWER_BORROW
    );

    let user_balance: Balance = view_balance(
        &controller,
        Borrow,
        borrower.account_id(),
        dweth.account_id(),
    );
    assert_eq!(
        user_balance, BORROWER_BORROW,
        "Borrow balance on controller should be {}",
        BORROWER_BORROW
    );

    set_price(
        &controller,
        dwnear.account_id(),
        &Price {
            ticker_id: "wnear".to_string(),
            value: WBalance::from(CHANGED_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    (dweth, dwnear, controller, weth, wnear, borrower, liquidator)
}

#[test]
fn scenario_liquidation_fail_due_to_low_health_factor() {
    let (dweth, dwnear, controller, weth, _wnear, borrower, liquidator) = liquidation_fixture();

    let amount = 4500;

    liquidate(&borrower, &liquidator, &dweth, &dwnear, &weth, amount).assert_success();

    let user_borrows: Balance =
        view!(dweth.get_account_borrows(borrower.account_id())).unwrap_json();
    assert_eq!(
        user_borrows, BORROWER_BORROW,
        "Borrow balance on dtoken should be BORROWER_BORROW"
    );

    let user_borrows: Balance = view_balance(
        &controller,
        Borrow,
        borrower.account_id(),
        dweth.account_id(),
    );
    assert_eq!(
        user_borrows, BORROWER_BORROW,
        "Borrow balance on controller should be BORROWER_BORROW"
    );

    let user_balance: Balance = view_balance(
        &controller,
        Supply,
        liquidator.account_id(),
        dwnear.account_id(),
    );

    assert_eq!(user_balance, 0, "Supply balance on dtoken should be 0");
}
