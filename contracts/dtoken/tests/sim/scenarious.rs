use near_sdk::AccountId;
use near_sdk_sim::{call, init_simulator, view};
use near_sdk_sim::types::Balance;
use crate::utils::{eth, init_dtoken, init_utoken, usdt};


#[test]
fn deploy_and_supply() {
    let root = init_simulator(None);

    let (root, dtoken, user) = init_dtoken(
        &root,
        usdt(),
    );
    // call!()

}