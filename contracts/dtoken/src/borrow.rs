use crate::*;

#[near_bindgen]
impl Contract {
    

    pub fn borrow(&mut self, token_amount: WBalance)  -> Promise {
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

        assert_eq!(is_promise_success(), true, "Increasing borrow has been failed");

        // Cross-contract call to market token
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
    ) ->bool {

        if is_promise_success(){
            self.increase_borrows(env::signer_account_id(), token_amount);
            return is_promise_success() ;
        } 
        else {
            log!("Transfer tokens from user to dtoken has failed");
            controller::decrease_borrows(
                env::signer_account_id(),
                self.get_contract_address(),
                token_amount,
                self.get_underlying_contract_address(),
                NO_DEPOSIT,
                self.terra_gas(10),
            )
            .then(ext_self::controller_decrease_borrows_fail(
                env::current_account_id().clone(),
                NO_DEPOSIT,
                self.terra_gas(10),
            ));
            return is_promise_success() ;
        }
    }

    pub fn controller_decrease_borrows_fail(&mut self){
        log!("Couldn't decrease borrow after mistake in transfer ");
    }






    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());

        assert!(existing_borrows >= Balance::from(tokens_amount), "Too much borrowed assets trying to pay out");

        let decreased_borrows: Balance = existing_borrows - Balance::from(tokens_amount);
        self.total_borrows -= Balance::from(tokens_amount);
        return self.set_borrows(account.clone(), U128(decreased_borrows));
    }

    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(tokens_amount);

        self.total_borrows += Balance::from(tokens_amount);
        return self.set_borrows(account.clone(), U128(increased_borrows));
    }

    #[private]
    pub fn set_borrows(&mut self, account: AccountId, tokens_amount: WBalance) -> Balance {
        self.borrows
            .insert(&account, &Balance::from(tokens_amount));
        return Balance::from(tokens_amount);
    }

    pub fn get_borrows_by_account(&self, account: AccountId) -> Balance{
        if self.borrows.get(&account).is_none(){
            return 0;
        }
        return self.borrows.get(&account).unwrap();
    }

}
