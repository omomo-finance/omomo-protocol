use near_sdk::AccountId;
use near_sdk_sim::{call, init_simulator, view, to_yocto};
use near_sdk_sim::types::Balance;
use crate::utils::{init_dtoken, init_utoken};
use near_sdk::json_types::{ U128};
use dtoken::Config;

pub fn weth() -> AccountId {
    AccountId::new_unchecked("weth".to_string())
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
    let uroot = root.create_user("utoken".parse().unwrap(), 9110000000086184677687500000000);

    println!("--1--");
    let (root, dtoken, user) = init_dtoken(
        droot,
        weth()
    );
    println!("--1/1--");

    let (uroot, utoken, uuser) = init_utoken(
        uroot,
        weth()
    );

    call!(
        uroot,
        utoken.new_default_meta("owner".parse().unwrap(), U128(10000)),
        deposit = 0
    )
    .assert_success();


    call!(
        root,
        dtoken.new(
            Config{
                initial_exchange_rate: U128(1), 
                underlying_token_id: utoken.account_id().clone(), 
                owner_id: "owner2".parse().unwrap(), 
                controller_account_id: "controller".parse().unwrap()
            }),
        deposit = 0
    )
    .assert_success();

    println!("--2--");

    // 1. If User doesn't supply any tokens
    call!(
        user,
        dtoken.withdraw(U128(20)),
        deposit = 0
    )
    .assert_success();

}