use near_sdk::AccountId;
use near_sdk_sim::{call, init_simulator, view};
use near_sdk_sim::types::Balance;
use crate::utils::{init_dtoken, init_utoken};

pub fn weth() -> AccountId {
    "weth".to_string()
}

#[test]
fn scenario_01() {

    let root = init_simulator(None);

    let (root, dtoken, user) = init_dtoken(
        &root,
        weth()
    );

    // call!(
    //     root,
    //     dtoken.supply(),
    //     deposit = 1
    // )
    // .assert_success();

    //....
}