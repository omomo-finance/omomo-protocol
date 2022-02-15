use crate::*;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Receives the transfer from the underlying fungible token and executes method call on controller
    /// Requires to be called by the fungible underlying token account.
    /// amount - Token amount
    #[allow(unreachable_code)]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        log!(format!("sender_id {}, msg {}", sender_id, msg));

        assert_eq!(
            sender_id,
            self.underlying_token,
            "ft_on_transfer: sender_id is not a valid address, actual {} expected {}",
            sender_id,
            self.underlying_token
        );

        let tkn_amount: Balance = amount.into();
        let config: Config = self.get_contract_config();
        let user_account = AccountId::new_unchecked(config.controller_account_id.to_string());

        controller::increase_supplies(
            env::signer_account_id(),
            tkn_amount,
            user_account,
            NO_DEPOSIT,
            TGAS * 20u64,
        )
            .then(ext_self::controller_increase_supplies_callback(
                tkn_amount,
                env::current_account_id().clone(),
                NO_DEPOSIT,
                TGAS * 20u64,
            ));

        PromiseOrValue::Value(U128(0))
    }
}

