use crate::big_decimal::WBalance;
use crate::cancel_order::ext_self;
use crate::utils::{ext_token, NO_DEPOSIT};
use crate::{Contract, ContractExt};
use near_sdk::json_types::U128;
use near_sdk::utils::is_promise_success;
use near_sdk::{env, log, near_bindgen, require, AccountId, Balance, PromiseOrValue, ONE_YOCTO};

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

        ext_token::ext(token.clone())
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer(
                user.clone(),
                amount,
                Some(format!(
                    "Withdraw with token_amount {}",
                    Balance::from(amount)
                )),
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .withdraw_callback(user.clone(), token.clone(), amount),
            )
            .into()
    }

    pub fn withdraw_callback(
        &mut self,
        account_id: AccountId,
        token: AccountId,
        amount: U128,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(amount);
        };

        self.decrease_balance(&account_id, &token, amount.0);
        log!(
            "Withdraw executed for user {} token {} amount {}",
            account_id,
            token,
            amount.0
        );
        PromiseOrValue::Value(amount)
    }
}
