use crate::*;

#[near_bindgen]
impl Contract {
    pub fn borrow(&mut self, token_amount: WBalance) -> Promise {
        if !self.mutex.try_lock(env::current_account_id()) {
            return Promise::new(env::current_account_id());
        }

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
        assert_eq!(is_promise_success(), true, "Failed to increase borrow for {} with token amount {}", env::signer_account_id(), Balance::from(token_amount));

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
                env::current_account_id().clone(),
                NO_DEPOSIT,
                self.terra_gas(10),
            ));
        } 
        else {
            self.increase_borrows(env::signer_account_id(), token_amount);
            self.mutex.unlock(env::signer_account_id());
        }
    }

    pub fn controller_decrease_borrows_fail(&mut self){
        if !is_promise_success(){
            log!("Failed to decrease borrows for {}", env::signer_account_id());
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

        let new_borrows = self.total_borrows.overflowing_sub(Balance::from(token_amount));
        assert_eq!(new_borrows.1, false, "Overflow occurs while decreasing total supply");
        self.total_borrows = new_borrows.0;
        
        return self.set_borrows(account.clone(), U128(decreased_borrows));
    }

    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(token_amount);

        let new_borrows = self.total_borrows.overflowing_add(Balance::from(token_amount));
        assert_eq!(new_borrows.1, false, "Overflow occurs while incresing total supply");
        self.total_borrows = new_borrows.0;
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
