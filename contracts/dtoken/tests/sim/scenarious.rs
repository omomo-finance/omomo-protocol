use near_sdk::{AccountId, collections::LookupMap};
use near_sdk_sim::{call, init_simulator, view, to_yocto, ExecutionResult, ContractAccount, UserAccount};
use crate::utils::{init_dtoken, init_utoken, init_controller};
use near_sdk::json_types::{ U128};
use dtoken::Config as dConfig;
use controller::{Config as cConfig, ContractContract};
use controller::borrows_supplies::ActionType::{Borrow, Supply};

fn assert_failure(outcome: ExecutionResult, error_message: &str) {
    assert!(!outcome.is_ok());
    let exe_status = format!("{:?}", outcome.promise_errors()[0].as_ref().unwrap().status());
    println!("{}", exe_status);
    assert!(exe_status.contains(error_message));
}

fn view_balance(contract: &ContractAccount<ContractContract>, user_account: AccountId, dtoken_account: AccountId) -> u128{
    view!(
        contract.get_by_token(controller::borrows_supplies::Supply, user_account, dtoken_account)
    ).unwrap_json()
}

#[test]
fn scenario_01() {
    // Supply
    let root = init_simulator(None);
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let uroot = root.create_user("utoken".parse().unwrap(), 1200000000000000000000000000000);
    let croot = root.create_user("controller".parse().unwrap(), 1200000000000000000000000000000);


    let (droot, dtoken, d_user) = init_dtoken(
        droot,
        AccountId::new_unchecked("dtoken_contract".to_string())
    );

    let (uroot, utoken, u_user) = init_utoken(
        uroot,
        AccountId::new_unchecked("utoken_contract".to_string())
    );

    let (croot, controller, c_user) = init_controller(
        croot,
        AccountId::new_unchecked("controller_contract".to_string())
    );


    //  Initialize
    call!(
        uroot,
        utoken.new_default_meta(uroot.account_id(), U128(10000)),
        deposit = 0
    )
    .assert_success();

    call!(
        croot,
        controller.new(
            cConfig{
                owner_id: croot.account_id().clone(), 
                oracle_account_id: "oracle".parse().unwrap()
            }),
        deposit = 0
    )
    .assert_success();

    call!(
        droot,
        dtoken.new(
            dConfig{
                initial_exchange_rate: U128(1), 
                underlying_token_id: utoken.account_id().clone(), 
                owner_id: droot.account_id().clone(), 
                controller_account_id: controller.account_id().clone()
            }),
        deposit = 0
    )
    .assert_success();

    // Supply preparation 
    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    ).assert_success();

    call!(
        d_user,
        utoken.mint(d_user.account_id(), U128(20)),
        0,
        100000000000000
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(d_user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 20.to_string(), "User balance should be 20");

    // Supply test with error in command
    call!(
        d_user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("SUPPL".to_string()),
            "SUPPL".to_string()
        ),
        deposit = 1
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(d_user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 20.to_string(), "As to mistake in command, transfer shouldn't be done");

    // Supply test with calling from dtoken instead of utoken
    let result = call!(
        d_user,
        dtoken.ft_on_transfer(
            d_user.account_id(),
            U128(20),
            "SUPPLY".to_string()
        ),
        deposit = 0
    );

    assert_failure(result, "The call should come from token account");

    // Supply test

    call!(
        d_user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("SUPPLY".to_string()),
            "SUPPLY".to_string()
        ),
        deposit = 1
    ).assert_success();


    let user_balance: String = view!(
        utoken.ft_balance_of(d_user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 0.to_string(), "User balance should be 0");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 20.to_string(), "Dtoken balance should be 20");

    let user_balance: u128 = view_balance(&controller, d_user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance on controller should be 20");

    // Supply test with 0 balance
    let result = call!(
        d_user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("SUPPLY".to_string()),
            "SUPPLY".to_string()
        ),
        deposit = 1
    );
    assert_failure(result, "The account doesn't have enough balance");

}

#[test]
fn scenario_02(){
    // Wihdraw
    let root = init_simulator(None);
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let uroot = root.create_user("utoken".parse().unwrap(), 1200000000000000000000000000000);
    let croot = root.create_user("controller".parse().unwrap(), 1200000000000000000000000000000);


    let (droot, dtoken, d_user) = init_dtoken(
        droot,
        AccountId::new_unchecked("dtoken_contract".to_string())
    );

    let (uroot, utoken, u_user) = init_utoken(
        uroot,
        AccountId::new_unchecked("utoken_contract".to_string())
    );

    let (croot, controller, c_user) = init_controller(
        croot,
        AccountId::new_unchecked("controller_contract".to_string())
    );


    //  Initialize
    call!(
        uroot,
        utoken.new_default_meta(uroot.account_id(), U128(10000)),
        deposit = 0
    )
    .assert_success();

    call!(
        croot,
        controller.new(
            cConfig{
                owner_id: croot.account_id().clone(), 
                oracle_account_id: "oracle".parse().unwrap()
            }),
        deposit = 0
    )
    .assert_success();

    call!(
        droot,
        dtoken.new(
            dConfig{
                initial_exchange_rate: U128(1), 
                underlying_token_id: utoken.account_id().clone(), 
                owner_id: droot.account_id().clone(), 
                controller_account_id: controller.account_id().clone()
            }),
        deposit = 0
    )
    .assert_success();


    // 1. If User doesn't supply any tokens
    
    let result = call!(
        d_user,
        dtoken.withdraw(U128(20)),
        deposit = 0
    );

    assert_failure(result, "Withdrawal operation is not allowed");


    // 2. If User supply some tokens and wants to withdraw 1) More 2) Less 3) The same
        // Simulate supply process
    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    ).assert_success();

    call!(
        d_user,
        utoken.mint(d_user.account_id(), U128(20)),
        0,
        100000000000000
    ).assert_success();

    call!(
        d_user,
        dtoken.mint(&d_user.account_id(), U128(20)),
        0,
        100000000000000
    ).assert_success();

    call!(
        d_user,
        controller.increase_supplies(d_user.account_id(), dtoken.account_id(), U128(20)),
        0,
        100000000000000
    ).assert_success();

    call!(
        d_user,
        utoken.ft_transfer(
            dtoken.account_id(), 
            U128(20), 
            Some(format!("Supply with token_amount 20"))),
        1,
        100000000000000
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(d_user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 0.to_string(), "User balance should be 0");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 20.to_string(), "Dtoken balance should be 20");


    let user_balance: u128 = view_balance(&controller, d_user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");


    let result = call!(
        d_user,
        dtoken.withdraw(U128(30)),
        deposit = 0
    );

    assert_failure(result, "Withdrawal operation is not allowed");

    let user_balance: u128 = view_balance(&controller, d_user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");

    call!(
        d_user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, d_user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 10, "Balance should be 10");

    call!(
        d_user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, d_user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Balance should be 0");

    let result = call!(
        d_user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    );

    assert_failure(result, "Withdrawal operation is not allowed");
    
    let user_balance: u128 = view_balance(&controller, d_user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Balance should be 0");

}