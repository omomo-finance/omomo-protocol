use crate::*;

impl Contract {

    pub fn supply_ft_transfer_fallback(&mut self, user_account: AccountId, token_amount: WBalance) {
        let message = format!(
            "Supply_ft_transfer_fallback from to user {} with token_amount {}",
            user_account,
            Balance::from(token_amount)
        );

        underlying_token::ft_transfer(
            self.get_signer_address(),
            token_amount,
            Some(message.clone()),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            self.terra_gas(20),
        );
        self.assert_on_supply_promise(
            "ft_balance_of".into(),
            false,
            token_amount
        );
    }

    pub fn assert_on_supply_promise(&self, method: String, promise_success: bool, amount: WBalance) {
        assert_eq!(
            promise_success,
            true,
            "Supply has failed on {}: Account {} amount {}",
            method,
            env::signer_account_id(),
            Balance::from(amount)
        );
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn supply(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {

        underlying_token::ft_balance_of(
            env::current_account_id(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(40),
        )
        .then(ext_self::supply_balance_of_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(60),
        ));
        PromiseOrValue::Value(U128(0))
    }

    #[allow(dead_code)]
    pub fn supply_balance_of_callback(&mut self, token_amount: WBalance) -> Promise {

        if !is_promise_success() {
            self.supply_ft_transfer_fallback(env::signer_account_id(), token_amount);
        }

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };
        let exchange_rate: Balance = self.get_exchange_rate(balance_of.into());
        let dtoken_amount = Balance::from(token_amount) * exchange_rate;

        // Dtokens minting and adding them to the user account
        self.mint(
            &self.get_signer_address(),
            dtoken_amount.into()
        );
        log!(
            "Supply from Account {} to Dtoken contract {} with tokens amount {} was successfully done!",
            self.get_signer_address(),
            self.get_contract_address(),
            Balance::from(token_amount)
        );
        
        return controller::increase_supplies(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(20),
        )
        .then(ext_self::controller_increase_supplies_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(10),
        ));
    }

    #[allow(dead_code)]
    pub fn controller_increase_supplies_callback(&mut self, amount: WBalance) {
        // let promise_success: bool = is_promise_success();
        // if promise_success {
        //     let total_reserves = self.total_reserves;
        //     self.total_reserves = total_reserves + Balance::from(amount);
        // }
        // PromiseOrValue::Value(U128(self.total_reserves))
    }
}
