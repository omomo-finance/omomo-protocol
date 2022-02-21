use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn supply(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {

        near_sdk::PromiseOrValue::Promise(underlying_token::ft_balance_of(
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
        )))
    }

    #[allow(dead_code)]
    pub fn supply_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {

        if !is_promise_success() {
            return PromiseOrValue::Value(token_amount);
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
        
        near_sdk::PromiseOrValue::Promise(controller::increase_supplies(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(20),
        )
        .then(ext_self::controller_increase_supplies_callback(
            token_amount,
            U128(dtoken_amount),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )))
    }

    #[allow(dead_code)]
    pub fn controller_increase_supplies_callback(&mut self, amount: WBalance, dtoken_amount: WBalance) -> PromiseOrValue<U128> {
        let promise_success: bool = is_promise_success();
        if !promise_success {
            self.burn(
                &self.get_signer_address(),
                dtoken_amount.into()
            );
            PromiseOrValue::Value(amount)
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }
}
