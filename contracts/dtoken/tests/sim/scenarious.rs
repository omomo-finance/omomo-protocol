use near_sdk::{AccountId, collections::LookupMap};
use near_sdk_sim::{call, init_simulator, view, to_yocto, ExecutionResult};
use crate::utils::{init_dtoken, init_utoken, init_controller};
use near_sdk::json_types::{ U128};
use dtoken::Config as dConfig;
use controller::Config as cConfig;

fn assert_failure(outcome: ExecutionResult, error_message: &str) {
    assert!(!outcome.is_ok());
    let exe_status = format!("{:?}", outcome.promise_errors()[0].as_ref().unwrap().status());
    println!("{}", exe_status);
    assert!(exe_status.contains(error_message));
}

#[test]
fn scenario_01() {

    // let root = init_simulator(None);
    // let droot = root.create_user("dtoken".parse().unwrap(), 1900000090000000000000000000000);
    // let uroot = root.create_user("utoken".parse().unwrap(), 9110000000086184677687500000000);
    


    // println!("--1--");
    // let (root, dtoken, user) = init_dtoken(
    //     droot,
    //     weth()
    // );
    // println!("--1/1--");

    // let (uroot, utoken, uuser) = init_utoken(
    //     uroot,
    //     weth()
    // );

    // call!(
    //     uroot,
    //     utoken.new_default_meta("owner".parse().unwrap(), U128(10000)),
    //     deposit = 0
    // )
    // .assert_success();


    // call!(
    //     root,
    //     dtoken.new(
    //         Config{
    //             initial_exchange_rate: U128(0), 
    //             underlying_token_id: utoken.account_id().clone(), 
    //             owner_id: "owner2".parse().unwrap(), 
    //             controller_account_id: "controller".parse().unwrap()
    //         }),
    //     deposit = 0
    // )
    // .assert_success();

    // println!("--3--");

    // // call!(
    // //     root,
    // //     dtoken.supply_balance_of_callback(U128(20)),
    // //     deposit = 0
    // // )
    // // .assert_success();

    // call!(
    //     user,
    //     dtoken.supply(U128(1)),
    //     deposit = 0
    // )
    // .assert_success();

    // //Если напрямую, ft_balance есть и отрабатывает
    // // let balance: u128 = view!(
    // //     utoken.ft_balance_of(dtoken.account_id())
    // // ).unwrap_json::<U128>().into();
    // // println!("Balance is {}", balance);

    // println!("--4--");

    // let total_supply: u128 = view!(
    //     dtoken.get_total_supplies()
    // ).unwrap_json();
    // println!("--5--");

    // assert_eq!(total_supply, 20);
}

#[test]
fn scenario_02(){
    let root = init_simulator(None);
    let droot = root.create_user("dtoken".parse().unwrap(), 1900000090000000000000000000000);
    let uroot = root.create_user("utoken".parse().unwrap(), 1900000090000000000000000000000);
    let croot = root.create_user("controller".parse().unwrap(), 1900000090000000000000000000000);


    println!("--1-- Deploy");
    let (droot, dtoken, user) = init_dtoken(
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

    println!("--2-- Init");

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

    println!("--3-- Call");

    // 1. If User doesn't supply any tokens
    let result = call!(
        user,
        dtoken.withdraw(U128(20)),
        deposit = 0
    );

    assert_failure(result, "Withdrawal operation is not allowed");

    println!("--4-- Call");

    // 2. If User supply some tokens and wants to withdraw the same amount
        // Simulate supply process
    call!(
        user,
        utoken.mint(dtoken.account_id(), U128(20)),
        0,
        100000000000000
    ).assert_success();

    call!(
        user,
        controller.increase_supplies(user.account_id(), utoken.account_id(), U128(20)),
        0,
        100000000000000
    ).assert_success();

    call!(
        uroot,
        utoken.ft_transfer_call(
            dtoken.account_id(), 
            U128(20), 
            Some(format!("Supply with token_amount 20")), 
            format!("Supply to dtoken from utoken with token_amount")),
        1,
        100000000000000
    ).assert_success();

    // let total_supply: u128 = view!(
    //     dtoken.get_total_supplies()
    // ).unwrap_json();
    // assert_eq!(total_supply, 20);

    call!(
        user,
        dtoken.withdraw(U128(20)),
        deposit = 0
    ).assert_success();

    // let total_supply: u128 = view!(
    //     dtoken.get_total_supplies()
    // ).unwrap_json();
    // assert_eq!(total_supply, 0);

    
    
}