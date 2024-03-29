use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk::{log, serde_json, Balance, PromiseOrValue};

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Accepts token to be deposited by user.
    ///
    /// msg format for deposit "{"Deposit": {"token": "<token_to_be_deposited>"}}"
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );

        log!(format!("sender_id {sender_id}, msg {msg}"));

        let action: Actions = serde_json::from_str(&msg).expect("Incorrect command in transfer");

        match action {
            Actions::Deposit { token } => self.deposit(amount, token),
            #[allow(unreachable_patterns)]
            _ => {
                panic!("Incorrect action in transfer")
            }
        }
    }
}
