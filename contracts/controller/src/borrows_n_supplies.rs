use crate::*;
use crate::borrows_n_supplies::EventType::{Borrow, Supply};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub enum EventType {
    Supply,
    Borrow,
}


#[near_bindgen]
impl Contract {
    #[private]
    fn set_by_token(&mut self, event: EventType, account: AccountId, token_address: AccountId, tokens_amount: Balance) -> Balance {
        // Receive EventType whether its Supply or Borrow so that
        // it will be doing respective variable configuration

        let (accounts, key_prefix) = match event {
            EventType::Supply => (&mut self.account_supplies, StorageKeys::SuppliesToken),
            EventType::Borrow => (&mut self.account_borrows, StorageKeys::BorrowsToken)
        };


        if !accounts.contains_key(&account) {
            let mut account_map: LookupMap<AccountId, u128> =
                LookupMap::new(key_prefix);
            account_map.insert(&token_address, &tokens_amount);
            accounts.insert(&account, &account_map);
        } else {
            accounts
                .get(&account)
                .unwrap()
                .insert(&token_address, &tokens_amount);
        }
        return tokens_amount;
    }


    // #[private] hafta be public
    pub fn get_by_token(&mut self, action: EventType, account: AccountId, token_address: AccountId) -> Balance {
        let balance: Balance = 0;


        let accounts = match action {
            EventType::Supply => {
                &self.account_supplies
            }
            EventType::Borrow => {
                &self.account_borrows
            }
        };

        if !accounts.contains_key(&account) {
            return balance;
        }

        let accounts_map = accounts.get(&account).unwrap();

        accounts_map.get(&token_address).unwrap_or(balance)
    }


    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) {
        let existing_borrows: Balance = self.get_by_token(Borrow, account.clone(), token_address.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(tokens_amount);

        self.set_by_token(Borrow, account.clone(), token_address.clone(), increased_borrows);
    }


    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) {
        let existing_borrows: Balance = self.get_by_token(Borrow, account.clone(), token_address.clone());

        // checking if i pay out ["return"] more than have to
        // f.e. i have 10 eth borrowed and trying to "give" to the contract 11
        assert!(existing_borrows >= Balance::from(tokens_amount), "Too much borrowed assets trying to pay out");

        let decreased_borrows: Balance = existing_borrows - Balance::from(tokens_amount);

        self.set_by_token(Borrow, account.clone(), token_address.clone(), decreased_borrows);
    }


    pub fn increase_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) {
        let existing_supplies = self.get_by_token(Supply, account.clone(), token_address.clone());
        let increased_supplies: Balance = existing_supplies + Balance::from(tokens_amount);

        self.set_by_token(Supply, account.clone(), token_address.clone(), increased_supplies);
    }

    pub fn decrease_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        let existing_supplies = self.get_by_token(Supply, account.clone(), token_address.clone());

        assert!(
            Balance::from(tokens_amount) <= existing_supplies,
            "Not enough existing supplies"
        );
        let decreased_supplies: Balance = existing_supplies - Balance::from(tokens_amount);

        return self.set_by_token(Supply,
                                 account.clone(),
                                 token_address.clone(),
                                 decreased_supplies,
        );
    }

    fn is_withdraw_allowed(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) -> bool {
        let existing_supplies = self.get_by_token(Supply, account.clone(), token_address.clone());

        return existing_supplies >= Balance::from(tokens_amount);
    }

    pub fn withdraw_supplies(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        assert_eq!(
            self.is_withdraw_allowed(
                account_id.clone(),
                token_address.clone(),
                tokens_amount.clone(),
            ),
            true,
            "Withdrawal operation is not allowed for account {} token_address {} tokens_amount {}",
            account_id,
            token_address,
            Balance::from(tokens_amount)
        );

        return self.decrease_supplies(account_id, token_address, tokens_amount);
    }


    #[warn(dead_code)]
    fn is_borrow_allowed(&mut self, account: AccountId, token_address: AccountId, _tokens_amount: WBalance) -> bool {
        let existing_borrows = self.get_by_token(Borrow, account.clone(), token_address.clone());


        let existing_supplies = self.get_by_token(Supply, account.clone(), token_address.clone());
        // TODO add check if allowed  (USD-estimated ACCOUNT SUPPLIES > USD-estimated ACCOUNT BORROWED  * ratio ? (or just 0.8) )

        // FIXME mock-checking for now
        return existing_supplies >= existing_borrows;
    }

    /*
    pub fn borrow_allowed(
        &mut self,
        dtoken_address: AccountId,
        user_address: AccountId,
        amount: u128,
    ) -> bool {
        let is_user_cap_allowed = match self.borrow_caps.get(&dtoken_address) {
            None => false,
            Some(user_cap) => amount < user_cap,
        };

        self.has_collaterall(user_address) && is_user_cap_allowed
    }
     */
}


#[cfg(test)]
mod tests {
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};

    use crate::borrows_n_supplies::EventType::{Borrow, Supply};

    use super::*;

    fn init() -> (Contract, AccountId, AccountId) {
        let (owner_account, oracle_account, user_account) = (alice(), bob(), carol());

        let eth_contract = Contract::new(Config { owner_id: owner_account, oracle_account_id: oracle_account });

        let token_address: AccountId = "eth".parse().unwrap();

        return (eth_contract, token_address, user_account);
    }


    #[test]
    fn test_for_supply_and_borrow_getters() {
        let (mut eth_contract, token_address, user_account) = init();
        assert_eq!(eth_contract.get_by_token(Supply, user_account.clone(), token_address.clone()), 0);
        assert_eq!(eth_contract.get_by_token(Borrow, user_account.clone(), token_address.clone()), 0);
    }


    #[test]
    fn test_for_supply_and_borrow_setters() {
        let (mut eth_contract, token_address, user_account) = init();
        eth_contract.set_by_token(Supply, user_account.clone(), token_address.clone(), 100);
        assert_eq!(eth_contract.get_by_token(Supply, user_account.clone(), token_address.clone()), 100);


        eth_contract.set_by_token(Borrow, user_account.clone(), token_address.clone(), 50);
        assert_eq!(eth_contract.get_by_token(Borrow, user_account.clone(), token_address.clone()), 50);
    }


    #[test]
    fn success_increase_n_decrease_borrows() {
        let (mut eth_contract, token_address, user_account) = init();

        eth_contract.increase_borrows(user_account.clone(), token_address.clone(), U128(10));

        assert_eq!(eth_contract.get_by_token(Borrow, user_account.clone(), token_address.clone()), 10);

        eth_contract.decrease_borrows(user_account.clone(), token_address.clone(), U128(2));

        assert_eq!(eth_contract.get_by_token(Borrow, user_account.clone(), token_address.clone()), 8);
    }


    #[test]
    fn success_increase_n_decrease_supplies() {
        let (mut eth_contract, token_address, user_account) = init();

        eth_contract.increase_supplies(user_account.clone(), token_address.clone(), U128(10));

        assert_eq!(eth_contract.get_by_token(Supply, user_account.clone(), token_address.clone()), 10);

        eth_contract.decrease_supplies(user_account.clone(), token_address.clone(), U128(2));

        assert_eq!(eth_contract.get_by_token(Supply, user_account.clone(), token_address.clone()), 8);
    }


    #[test]
    #[should_panic]
    fn failed_decrease_borrows() {
        let (mut eth_contract, token_address, user_account) = init();

        eth_contract.increase_borrows(user_account.clone(), token_address.clone(), U128(10));

        /*
        trying to decrease borrows of 20 having 10 borrowed units of asset
        should throw panic with err "Too much borrowed assets trying to pay out"
        */

        eth_contract.decrease_borrows(user_account.clone(), token_address.clone(), U128(20));
    }
}