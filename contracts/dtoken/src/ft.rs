use crate::*;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde_json;
use near_sdk::AccountId;

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_eq!(
            env::predecessor_account_id(),
            self.underlying_token,
            "The call should come from token account"
        );
        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );

        log!(format!("sender_id {}, msg {}", sender_id, msg));

        let action: Actions = serde_json::from_str(&msg).expect("Incorrect command in transfer");

        match action {
            Actions::Supply => self.supply(amount),
            Actions::Repay => self.repay(amount),
            Actions::Liquidate {
                borrower,
                borrowing_dtoken,
                collateral_dtoken,
            } => self.liquidate(
                borrower,
                borrowing_dtoken,
                env::signer_account_id(),
                collateral_dtoken,
                amount,
            ),
            Actions::Reserve => self.reserve(amount),
            _ => {
                panic!("Incorrect action in transfer")
            }
        }
    }
}

#[near_bindgen]
impl FungibleTokenCore for Contract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        require!(!self.disable_transfer, "Transfer dtoken is disabled");
        self.token.ft_transfer(receiver_id, amount, memo);
    }

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        require!(!self.disable_transfer, "Transfer dtoken is disabled");
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}
