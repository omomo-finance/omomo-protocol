use crate::*;

#[near_bindgen]
impl Contract {
    pub fn borrow(&mut self, token_amount: WBalance) -> Promise {
        return controller::make_borrow(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::make_borrow_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(150),
        ));
    }

    pub fn make_borrow_callback(
        &mut self,
        token_amount: WBalance,
    ) ->Promise {
        if !is_promise_success() {
            log!(
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "borrow_fail", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to make borrow for {} on {} token amount"}}}}"#,  
                env::signer_account_id(), Balance::from(token_amount), env::signer_account_id(), Balance::from(token_amount)
            ); 
            panic!(); 
        } 

        underlying_token::ft_transfer(
            env::signer_account_id(),
            token_amount,
            Some(format!("Borrow with token_amount {}", Balance::from(token_amount))),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            self.terra_gas(40),
        )
        .then(ext_self::borrow_ft_transfer_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(80),
        ))
    }

    pub fn borrow_ft_transfer_callback(
        &mut self,
        token_amount: WBalance,
    ) {
        if !is_promise_success(){
            log!("Failed to transfer tokens from {} to user {} with token amount {}", self.get_contract_address(), env::signer_account_id(), Balance::from(token_amount));
            controller::decrease_borrows(
                env::signer_account_id(),
                self.get_contract_address(),
                token_amount,
                self.get_controller_address(),
                NO_DEPOSIT,
                self.terra_gas(10),
            )
            .then(ext_self::controller_decrease_borrows_fail(
                token_amount,
                env::current_account_id().clone(),
                NO_DEPOSIT,
                self.terra_gas(10),
            ));
        } 
        else {
            self.increase_borrows(env::signer_account_id(), token_amount);
            log!(
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "borrow_success", "data": {{"account_id": "{}", "amount": "{}"}}}}"#,  env::signer_account_id(), Balance::from(token_amount)
            );
        }
    }

    pub fn controller_decrease_borrows_fail(&mut self, token_amount: WBalance,){
        if !is_promise_success(){
            log!(
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "borrow_fail", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to decrease borrow for {} on {} token amount after transfer fail"}}}}"#,  
                env::signer_account_id(), Balance::from(token_amount), env::signer_account_id(), Balance::from(token_amount)
            );
            // TODO Account should be marked
        }
    }

    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());

        assert!(existing_borrows >= Balance::from(token_amount), "Repay amount is more than existing borrows");
        let decreased_borrows: Balance = existing_borrows - Balance::from(token_amount);

        let new_total_borrows = self.total_borrows.checked_sub(Balance::from(token_amount));
        assert!(new_total_borrows.is_some(), "Overflow occurs while decreasing total borrow");
        self.total_borrows = new_total_borrows.unwrap();
        
        return self.set_borrows(account.clone(), U128(decreased_borrows));
    }

    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(token_amount);

        let new_total_borrows = self.total_borrows.checked_add(Balance::from(token_amount));
        assert!(new_total_borrows.is_some(), "Overflow occurs while incresing total borrow");
        self.total_borrows = new_total_borrows.unwrap();
        return self.set_borrows(account.clone(), U128(increased_borrows));
    }

    #[private]
    pub fn set_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        self.borrows
            .insert(&account, &Balance::from(token_amount));
        return Balance::from(token_amount);
    }

    pub fn get_borrows_by_account(&self, account: AccountId) -> Balance{
        if self.borrows.get(&account).is_none(){
            return 0;
        }
        return self.borrows.get(&account).unwrap();
    }

}
