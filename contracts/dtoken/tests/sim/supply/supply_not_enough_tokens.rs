use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, ContractAccount, UserAccount};

use general::Price;

use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_dtoken, initialize_utoken,
    new_user, supply, 
};

const WNEAR_BALANCE: u128 = 50;
const SUPPLY_AMOUNT: u128 = 100;

fn supply_not_enough_tokens_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (uroot, wnear) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dwnear) = initialize_dtoken(&root, wnear.account_id(), controller.account_id());

    call!(
        uroot,
        wnear.mint(dwnear.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        uroot,
        wnear.mint(user.account_id(), U128(WNEAR_BALANCE)),
        0,
        100000000000000
    );

    add_market(
        &controller,
        wnear.account_id(),
        dwnear.account_id(),
        "wnear".to_string(),
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dwnear.account_id(),
            &Price {
                ticker_id: "wnear".to_string(),
                value: U128(10000),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    (dwnear, wnear, user)
}

#[test]
fn scenario_supply_not_enough_balance() {
    let (dwnear, wnear, user) = supply_not_enough_tokens_fixture();

    let result = supply(&user, &wnear, dwnear.account_id(), SUPPLY_AMOUNT);
    assert_failure(result, "The account doesn't have enough balance");
}
