use crate::*;
const GAS_FOR_BORROW: Gas = Gas(120_000_000_000_000);


#[near_bindgen]
impl Contract {

    pub fn borrow(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(env::prepaid_gas() >= GAS_FOR_BORROW, "Prepaid gas is not enough for borrow flow");
        self.mutex_account_lock(String::from("borrow"));

        underlying_token::ft_balance_of(
            env::current_account_id(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            TGAS,
        )
        .then(ext_self::borrow_balance_of_callback(
            token_amount,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(90),
        ))
        .into()
    }

    #[private]
    pub fn borrow_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            Contract::custom_fail_log(String::from("borrow_fail"), env::signer_account_id(), Balance::from(token_amount), format!("failed to get {} balance on {}", self.get_contract_address(), self.get_underlying_contract_address()));
            self.mutex_account_unlock();
            return PromiseOrValue::Value(token_amount);
        }

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let borrow_rate: Balance = self.get_borrow_rate(U128(balance_of), U128(self.get_total_borrows()), U128(self.total_reserves));
        let borrow_accrued_interest = self.model.calculate_accrued_interest(
            borrow_rate, 
            self.get_account_borrows(env::signer_account_id()), 
            self.get_accrued_borrow_interest(env::signer_account_id())
        );
        self.set_accrued_borrow_interest(env::signer_account_id(), borrow_accrued_interest);

        return controller::make_borrow(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(5),
        )
        .then(ext_self::make_borrow_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(60),
        ))
        .into()
    }

    #[private]
    pub fn make_borrow_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            Contract::custom_fail_log(String::from("borrow_fail"), env::signer_account_id(), Balance::from(token_amount), format!("failed to make borrow for {} on {} token amount", env::signer_account_id(), Balance::from(token_amount)));
            self.mutex_account_unlock();
            return PromiseOrValue::Value(token_amount);
        }

        underlying_token::ft_transfer(
            env::signer_account_id(),
            token_amount,
            Some(format!(
                "Borrow with token_amount {}",
                Balance::from(token_amount)
            )),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            self.terra_gas(10),
        )
        .then(ext_self::borrow_ft_transfer_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(25),
        ))
        .into()
    }

    #[private]
    pub fn borrow_ft_transfer_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if is_promise_success() {
            self.increase_borrows(env::signer_account_id(), token_amount);
            self.mutex_account_unlock();
            Contract::custom_success_log(String::from("borrow_success"), env::signer_account_id(), Balance::from(token_amount));
            return PromiseOrValue::Value(token_amount);
        } else {
            controller::decrease_borrows(
                env::signer_account_id(),
                self.get_contract_address(),
                token_amount,
                self.get_controller_address(),
                NO_DEPOSIT,
                self.terra_gas(2),
            )
            .then(ext_self::controller_decrease_borrows_fail(
                token_amount,
                env::current_account_id().clone(),
                NO_DEPOSIT,
                self.terra_gas(2),
            ))
            .into()
        }
    }

    #[private]
    pub fn controller_decrease_borrows_fail(&mut self, token_amount: WBalance){
        if !is_promise_success(){
            Contract::custom_fail_log(String::from("borrow_fail"), env::signer_account_id(), Balance::from(token_amount), format!("failed to revert state for {}", env::signer_account_id()));
            self.add_inconsistent_account(env::signer_account_id());
        }
        self.mutex_account_unlock();
        // TODO: does it really success ???? Perhaps borrow fallback was successfully finished ???
        // TODO: Change string messages into Enum type
        Contract::custom_success_log(String::from("borrow_success"), env::signer_account_id(), Balance::from(token_amount));
    }


    pub fn decrease_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        let borrows = self.get_account_borrows(account.clone());
        let new_borrows = borrows - Balance::from(token_amount);

        return self.set_account_borrows(account.clone(), U128(new_borrows));
    }

    pub fn increase_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        let borrows: Balance = self.get_account_borrows(account.clone());
        let new_borrows = borrows + Balance::from(token_amount);

        return self.set_account_borrows(account.clone(), U128(new_borrows));
    }

    pub fn set_account_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        let mut user = self.user_profiles.get(&account).unwrap_or_default();
        user.borrows = Balance::from(token_amount);
        self.user_profiles.insert(&account, &user);

        self.get_account_borrows(account)
    }

    pub fn get_account_borrows(&self, account: AccountId) -> Balance {
        return self.user_profiles.get(&account).unwrap_or_default().borrows;
    }
}