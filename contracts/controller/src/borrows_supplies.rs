use near_sdk::require;
use near_sdk::BlockHeight;
use std::collections::HashMap;

use crate::borrows_supplies::ActionType::{Borrow, Supply};
use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    Supply,
    Borrow,
}

#[near_bindgen]
impl Contract {
    pub fn make_borrow(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
        borrow_block: BlockHeight,
        borrow_rate: WRatio,
    ) {
        assert!(
            self.is_borrow_allowed(account_id.clone(), token_address.clone(), token_amount,),
            "Borrow operation is not allowed for account {} token_address {} token_amount {}",
            account_id,
            token_address,
            Balance::from(token_amount)
        );
        self.increase_borrows(
            account_id,
            token_address,
            token_amount,
            borrow_block,
            Ratio::from(borrow_rate.0),
        );
    }

    pub fn withdraw_supplies(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        assert!(
            self.is_withdraw_allowed(account_id.clone(), token_address.clone(), token_amount,),
            "Withdrawal operation is not allowed for account {} token_address {} token_amount` {}",
            account_id,
            token_address,
            Balance::from(token_amount)
        );

        self.decrease_supplies(account_id, token_address, token_amount)
    }

    pub fn get_entity_by_token(
        &self,
        action: ActionType,
        user_id: AccountId,
        token_address: AccountId,
    ) -> Balance {
        let user = self.user_profiles.get(&user_id).unwrap_or_default();

        user.get(action, token_address)
    }

    pub fn increase_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) {
        let existing_supplies =
            self.get_entity_by_token(Supply, account.clone(), token_address.clone());
        let increased_supplies: Balance = existing_supplies + Balance::from(token_amount);

        self.set_entity_by_token(Supply, account, token_address, increased_supplies);
    }

    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
        borrow_block: BlockHeight,
        borrow_rate: Ratio,
    ) {
        let existing_borrows: Balance =
            self.get_entity_by_token(Borrow, account.clone(), token_address.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(token_amount);

        let mut borrow_block = borrow_block;
        if existing_borrows != 0 {
            borrow_block = self
                .user_profiles
                .get(&account)
                .unwrap_or_default()
                .get_borrow_data(token_address.clone())
                .borrow_block;
        }
        let borrow_data = BorrowData {
            borrow_block,
            borrow_rate,
        };

        self.user_profiles
            .get(&account)
            .unwrap_or_default()
            .insert_borrow_data(token_address.clone(), borrow_data);
        self.set_entity_by_token(Borrow, account, token_address, increased_borrows);
    }

    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
        borrow_block: BlockHeight,
        borrow_rate: WRatio,
    ) -> Balance {
        let existing_borrows: Balance =
            self.get_entity_by_token(Borrow, account.clone(), token_address.clone());

        let decreased_borrows: Balance = existing_borrows - Balance::from(token_amount);

        let mut borrow_rate = borrow_rate.0;
        if decreased_borrows == 0 {
            borrow_rate = 0;
        }
        let borrow_data = BorrowData {
            borrow_block,
            borrow_rate: Ratio::from(borrow_rate),
        };
        self.user_profiles
            .get(&account)
            .unwrap_or_default()
            .insert_borrow_data(token_address.clone(), borrow_data);

        self.set_entity_by_token(Borrow, account, token_address, decreased_borrows)
    }

    pub fn decrease_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        let existing_supplies =
            self.get_entity_by_token(Supply, account.clone(), token_address.clone());

        assert!(
            Balance::from(token_amount) <= existing_supplies,
            "Not enough existing supplies for {} with supplies {} {} {}",
            account,
            existing_supplies,
            token_address,
            Balance::from(token_amount)
        );
        let decreased_supplies: Balance = existing_supplies - Balance::from(token_amount);

        self.set_entity_by_token(Supply, account, token_address, decreased_supplies)
    }
}

impl Contract {
    pub fn set_entity_by_token(
        &mut self,
        action: ActionType,
        user_id: AccountId,
        token_address: AccountId,
        token_amount: Balance,
    ) -> Balance {
        let mut user = self.user_profiles.get(&user_id).unwrap_or_default();
        user.set(action, token_address, token_amount);
        self.user_profiles.insert(&user_id, &user);

        token_amount
    }

    fn is_withdraw_allowed(
        &self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> bool {
        require!(!self.is_action_paused.withdraw, "withdrawing is paused");
        let existing_supplies =
            self.get_entity_by_token(Supply, account.clone(), token_address.clone());
        assert!(
            Balance::from(token_amount) <= existing_supplies,
            "Not enough existing supplies for {}",
            account
        );
        self.get_potential_health_factor(account, token_address, token_amount, Supply)
            >= self.get_liquidation_threshold()
    }

    #[warn(dead_code)]
    fn is_borrow_allowed(
        &self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> bool {
        require!(!self.is_action_paused.borrow, "borrowing is paused");
        self.get_potential_health_factor(account, token_address, token_amount, Borrow)
            >= self.get_liquidation_threshold()
    }

    pub fn calculate_assets_price(&self, map: &HashMap<AccountId, Balance>) -> Balance {
        map.iter()
            .map(|(asset, balance)| {
                let price = self.get_price(asset.clone()).unwrap();

                (BigBalance::from(price.value) * BigBalance::from(balance.to_owned())
                    / BigBalance::from(U128(ONE_TOKEN)))
                .round_u128()
            })
            .sum()
    }

    pub fn get_total_supplies(&self, user_id: AccountId) -> USD {
        let supplies = self
            .user_profiles
            .get(&user_id)
            .unwrap_or_default()
            .account_supplies;

        self.calculate_assets_price(&supplies).into()
    }

    pub fn get_total_borrows(&self, user_id: AccountId) -> USD {
        let borrows = self
            .user_profiles
            .get(&user_id)
            .unwrap_or_default()
            .account_borrows;

        self.calculate_assets_price(&borrows).into()
    }
}

#[cfg(test)]
mod tests {
    use general::ratio::Ratio;
    use general::{Price, WRatio, ONE_TOKEN};
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::AccountId;

    use crate::borrows_supplies::ActionType::{Borrow, Supply};
    use crate::{Config, Contract};

    pub fn init_test_env() -> (Contract, AccountId, AccountId) {
        let (owner_account, oracle_account, user_account) = (alice(), bob(), carol());

        let near_contract = Contract::new(Config {
            owner_id: owner_account,
            oracle_account_id: oracle_account,
        });

        let token_address: AccountId = "near".parse().unwrap();

        (near_contract, token_address, user_account)
    }

    #[test]
    fn test_for_supply_and_borrow_getters() {
        let (near_contract, token_address, user_account) = init_test_env();
        assert_eq!(
            near_contract.get_entity_by_token(Supply, user_account.clone(), token_address.clone()),
            0
        );
        assert_eq!(
            near_contract.get_entity_by_token(Borrow, user_account, token_address),
            0
        );
    }

    #[test]
    fn test_for_supply_and_borrow_setters() {
        let (mut near_contract, token_address, user_account) = init_test_env();
        near_contract.set_entity_by_token(Supply, user_account.clone(), token_address.clone(), 100);
        assert_eq!(
            near_contract.get_entity_by_token(Supply, user_account.clone(), token_address.clone()),
            100
        );

        near_contract.set_entity_by_token(Borrow, user_account.clone(), token_address.clone(), 50);
        assert_eq!(
            near_contract.get_entity_by_token(Borrow, user_account, token_address),
            50
        );
    }

    #[test]
    fn success_increase_n_decrease_borrows() {
        let (mut near_contract, token_address, user_account) = init_test_env();

        near_contract.increase_borrows(
            user_account.clone(),
            token_address.clone(),
            U128(10),
            0,
            Ratio::zero(),
        );
        near_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("test.nearlend".to_string()),
            U128(100),
            0,
            Ratio::zero(),
        );

        assert_eq!(
            near_contract.get_entity_by_token(Borrow, user_account.clone(), token_address.clone()),
            10
        );
        assert_eq!(
            near_contract.get_entity_by_token(
                Borrow,
                user_account.clone(),
                AccountId::new_unchecked("test.nearlend".to_string())
            ),
            100
        );

        near_contract.decrease_borrows(
            user_account.clone(),
            token_address.clone(),
            U128(2),
            0,
            WRatio::from(0),
        );
        near_contract.decrease_borrows(
            user_account.clone(),
            AccountId::new_unchecked("test.nearlend".to_string()),
            U128(2),
            0,
            WRatio::from(0),
        );

        assert_eq!(
            near_contract.get_entity_by_token(Borrow, user_account.clone(), token_address),
            8
        );
        assert_eq!(
            near_contract.get_entity_by_token(
                Borrow,
                user_account,
                AccountId::new_unchecked("test.nearlend".to_string())
            ),
            98
        );
    }

    #[test]
    fn success_increase_n_decrease_supplies() {
        let (mut near_contract, token_address, user_account) = init_test_env();

        near_contract.increase_supplies(user_account.clone(), token_address.clone(), U128(10));
        near_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("test.nearlend".to_string()),
            U128(20),
        );

        assert_eq!(
            near_contract.get_entity_by_token(Supply, user_account.clone(), token_address.clone()),
            10
        );
        assert_eq!(
            near_contract.get_entity_by_token(
                Supply,
                user_account.clone(),
                AccountId::new_unchecked("test.nearlend".to_string())
            ),
            20
        );

        near_contract.decrease_supplies(user_account.clone(), token_address.clone(), U128(2));
        near_contract.decrease_supplies(
            user_account.clone(),
            AccountId::new_unchecked("test.nearlend".to_string()),
            U128(2),
        );

        assert_eq!(
            near_contract.get_entity_by_token(Supply, user_account.clone(), token_address),
            8
        );
        assert_eq!(
            near_contract.get_entity_by_token(
                Supply,
                user_account,
                AccountId::new_unchecked("test.nearlend".to_string())
            ),
            18
        );
    }

    #[test]
    #[should_panic]
    fn failed_decrease_borrows() {
        /*
        Test for decrease flow behavior computation
        */
        let (mut near_contract, token_address, user_account) = init_test_env();

        near_contract.increase_borrows(
            user_account.clone(),
            token_address.clone(),
            U128(10),
            0,
            Ratio::zero(),
        );

        near_contract.decrease_borrows(user_account, token_address, U128(20), 0, WRatio::from(0));
    }

    #[test]
    fn get_total_supplies() {
        let (mut near_contract, token_address, user_account) = init_test_env();

        let price = Price {
            ticker_id: "wnear".to_string(),
            value: U128(100 * ONE_TOKEN),
            volatility: U128(1),
            fraction_digits: 4u32,
        };
        near_contract.upsert_price(token_address.clone(), &price);
        near_contract.increase_supplies(user_account.clone(), token_address, U128(10));

        assert_eq!(near_contract.get_total_supplies(user_account), U128(1000));
    }

    #[test]
    fn get_total_borrows() {
        let (mut near_contract, token_address, user_account) = init_test_env();

        let price = Price {
            ticker_id: "wnear".to_string(),
            value: U128(100 * ONE_TOKEN),
            volatility: U128(1),
            fraction_digits: 4u32,
        };
        near_contract.upsert_price(token_address.clone(), &price);
        near_contract.increase_borrows(
            user_account.clone(),
            token_address,
            U128(10),
            0,
            Ratio::zero(),
        );

        assert_eq!(near_contract.get_total_borrows(user_account), U128(1000));
    }
}
