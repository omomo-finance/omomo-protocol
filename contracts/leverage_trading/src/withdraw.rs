use crate::big_decimal::WBalance;
use crate::cancel_order::ext_self;
use crate::common::Event;
use crate::utils::ext_token;
use crate::{Contract, ContractExt};
use near_sdk::json_types::U128;
use near_sdk::utils::is_promise_success;
use near_sdk::{env, near_bindgen, require, AccountId, Balance, PromiseOrValue, ONE_YOCTO};

#[near_bindgen]
impl Contract {
    pub fn withdraw(&mut self, token: AccountId, amount: U128) -> PromiseOrValue<WBalance> {
        let user = env::signer_account_id();
        let user_balance = self.balance_of(user.clone(), token.clone());

        require!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );
        require!(
            user_balance >= amount,
            "The account doesn't have enough digital tokens to do withdraw"
        );

        let token_decimals = self.view_token_decimals(&token);
        let token_amount = self.from_protocol_to_token_decimals(amount, token_decimals);

        ext_token::ext(token.clone())
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer(user.clone(), token_amount, None)
            .then(ext_self::ext(env::current_account_id()).withdraw_callback(user, token, amount))
            .into()
    }

    pub fn withdraw_callback(
        &mut self,
        account_id: AccountId,
        token: AccountId,
        amount: U128,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(U128(0));
        };

        self.decrease_balance(&account_id, &token, amount.0);
        Event::WithdrawEvent { token, amount }.emit();
        PromiseOrValue::Value(amount)
    }
}
