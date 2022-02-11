use crate::*;

#[near_bindgen]
impl Contract {
    #[private]
    fn set_borrows_by_token(&mut self, account: AccountId, token_address: AccountId, tokens_amount: Balance) -> Balance {

        // if user hasn't borrowed yet, we set up the config in account_borrows LookupMap

        if !self.account_borrows.contains_key(&account) {
            let mut borrows_map: LookupMap<AccountId, u128> =
                LookupMap::new(StorageKeys::SuppliesToken);
            borrows_map.insert(&token_address, &tokens_amount);
            self.account_borrows.insert(&account, &borrows_map);
        } else {

            // otherwise insert into existing ones
            self.account_borrows
                .get(&account)
                .unwrap()
                .insert(&token_address, &tokens_amount);
        }
        return tokens_amount;
    }


    #[private]
    fn get_borrows_by_token(&mut self, account: AccountId, token_address: AccountId) -> Balance {
        // initial balance in case there are no borrowed  users assets
        let borrowed_balance: Balance = 0;

        // check whether lookupmap contains account if not -> 0 assets was borrowed
        if !self.account_borrows.contains_key(&account) {
            return borrowed_balance;
        }

        // get the map so that we are able to extract the
        // LookupMap< Token_addr, borrowed_amount  >
        let borrows_map = self.account_borrows.get(&account).unwrap();


        // get the respective borrowed amount
        return borrows_map.get(&token_address).unwrap_or(borrowed_balance);
    }


    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) {
        let existing_borrows: Balance = self.get_borrows_by_token(account.clone(), token_address.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(tokens_amount);

        self.set_borrows_by_token(account.clone(), token_address.clone(), increased_borrows);
    }


    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) {
        let existing_borrows: Balance = self.get_borrows_by_token(account.clone(), token_address.clone());


        // checking if i pay out ["return"] more than have to
        // f.e. i have 10 eth borrowed and trying to "give" to the contract 11
        assert!(existing_borrows >= Balance::from(tokens_amount), "Too much borrowed assets trying to pay out");

        let decreased_borrows: Balance = existing_borrows - Balance::from(tokens_amount);

        self.set_borrows_by_token(account.clone(), token_address.clone(), decreased_borrows);
    }


    fn is_borrow_allowed(&mut self, account: AccountId, token_address: AccountId) -> bool {


        // TODO add check if allowed  (account supplies > account borrowed)

        let existing_borrows = self.get_borrows_by_token(account.clone(), token_address.clone());

        return true;
    }
}


#[cfg(test)]
mod tests {
    use near_sdk::test_utils::test_env::{alice, bob, carol};

    use super::*;

    fn init() -> (Contract, AccountId, AccountId) {
        let (owner_account, oracle_account, user_account) = (alice(), bob(), carol());

        let mut eth_contract = Contract::new(Config { owner_id: owner_account, oracle_account_id: oracle_account });

        let token_address: AccountId = "eth".parse().unwrap();

        return (eth_contract, token_address, user_account);
    }


    #[test]
    fn success_increase_n_decrease_borrows() {
        let (mut eth_contract, token_address, user_account) = init();

        eth_contract.increase_borrows(user_account.clone(), token_address.clone(), U128(10));

        assert_eq!(eth_contract.get_borrows_by_token(user_account.clone(), token_address.clone()), 10);

        eth_contract.decrease_borrows(user_account.clone(), token_address.clone(), U128(2));

        assert_eq!(eth_contract.get_borrows_by_token(user_account.clone(), token_address.clone()), 8);
    }

    #[test]
    #[should_panic]
    fn failed_decrease() {
        let (mut eth_contract, token_address, user_account) = init();

        eth_contract.increase_borrows(user_account.clone(), token_address.clone(), U128(10));

        /*
        trying to decrease borrows of 20 having 10 borrowed units of asset
        should throw panic with err "Too much borrowed assets trying to pay out"
        */

        eth_contract.decrease_borrows(user_account.clone(), token_address.clone(), U128(20));

    }







}

