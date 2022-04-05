use crate::*;

use std::collections::HashMap;

impl Contract {
    pub fn get_price_sum(&self, map: &HashMap<AccountId, Balance>) -> Balance {
        let mut result: Balance = 0;
        for (asset, balance) in map.iter() {
            let price = self.get_price(asset.clone()).unwrap();
            result += Percentage::from(Percent::from(price.volatility))
                .apply_to(Balance::from(price.value) * balance / 10u128.pow(price.fraction_digits));
        }

        result
    }

    fn get_account_sum_per_action(&self, user_account: AccountId, action: ActionType) -> Balance {
        let map_raw: HashMap<AccountId, Balance> = match action {
            ActionType::Supply => {
                self.user_profiles
                    .get(&user_account)
                    .unwrap_or_default()
                    .account_supplies
            }
            ActionType::Borrow => {
                self.user_profiles
                    .get(&user_account)
                    .unwrap_or_default()
                    .account_borrows
            }
        };

        self.get_price_sum(&map_raw)
    }

    pub fn get_potential_health_factor(&self, user_account: AccountId, token_address: AccountId, amount: Balance, action: ActionType) -> Ratio {
        let mut ratio = HEALTH_THRESHOLD;
        let mut collaterals = self.get_account_sum_per_action(user_account.clone(), ActionType::Supply);
        let mut borrows = self.get_account_sum_per_action(user_account, ActionType::Borrow);

        let price = self.get_price(token_address).unwrap();
        let usd_amount = Percentage::from(Percent::from(price.volatility)).apply_to(Balance::from(price.value) * amount / 10u128.pow(price.fraction_digits));
        match action {
            ActionType::Supply => {
                collaterals -= usd_amount;
            }
            ActionType::Borrow => {
                borrows += usd_amount;
            }
        }

        if borrows != 0 {
            ratio = collaterals * RATIO_DECIMALS / borrows;
        }
        ratio
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_health_factor(&self, user_account: AccountId) -> Ratio {
        let mut ratio = HEALTH_THRESHOLD;
        let collaterals = self.get_account_sum_per_action(user_account.clone(), ActionType::Supply);
        let borrows = self.get_account_sum_per_action(user_account, ActionType::Borrow);

        if borrows != 0 {
            ratio = collaterals * RATIO_DECIMALS / borrows;
        }

        ratio
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::test_env::{alice, bob};

    use super::*;

    // use crate::borrows_supplies::ActionType::{Borrow, Supply};

    fn init() -> (Contract, AccountId, AccountId) {
        let (_owner_account, user_account) = (alice(), bob());

        let mut controller_contract = Contract::new(Config {
            owner_id: user_account.clone(),
            oracle_account_id: user_account.clone(),
        });

        let utoken_address_near = AccountId::new_unchecked("wnear.near".to_string());
        let dtoken_address_near = AccountId::new_unchecked("dwnear.near".to_string());
        let ticker_id_near = "wnear".to_string();

        controller_contract.add_market(
            utoken_address_near,
            dtoken_address_near,
            ticker_id_near.clone(),
        );

        let utoken_address_eth = AccountId::new_unchecked("weth.near".to_string());
        let dtoken_address_eth = AccountId::new_unchecked("dweth.near".to_string());
        let ticker_id_eth = "weth".to_string();

        controller_contract.add_market(
            utoken_address_eth,
            dtoken_address_eth,
            ticker_id_eth.clone(),
        );

        let mut prices: Vec<Price> = Vec::new();
        prices.push(Price {
            ticker_id: ticker_id_near,
            value: U128(20000),
            volatility: U128(80),
            fraction_digits: 4,
        });
        prices.push(Price {
            ticker_id: ticker_id_eth,
            value: U128(20000),
            volatility: U128(100),
            fraction_digits: 4,
        });

        controller_contract.oracle_on_data(PriceJsonList {
            block_height: 83452949,
            price_list: prices,
        });

        let token_address: AccountId = AccountId::new_unchecked("near".to_string());

        (controller_contract, token_address, user_account)
    }

    #[test]
    fn test_get_price_sum_empty_map() {
        let (controller_contract, _token_address, _user_account) = init();

        let raw_map_empty: HashMap<AccountId, Balance> = HashMap::new();
        assert_eq!(
            controller_contract.get_price_sum(&raw_map_empty),
            0,
            "Test for None Option has been failed"
        );
    }

    #[test]
    fn test_for_get_price_sum() {
        let (controller_contract, _token_address, _user_account) = init();

        let mut raw_map: HashMap<AccountId, Balance> = HashMap::new();
        raw_map.insert(AccountId::new_unchecked("dwnear.near".to_string()), 100);

        assert_eq!(
            controller_contract.get_price_sum(&raw_map),
            160,
            "Test for None Option has been failed"
        );
    }

    #[test]
    fn test_for_get_health_factor() {
        let (mut controller_contract, _token_address, user_account) = init();

        let balance: Balance = 50;

        assert_eq!(
            controller_contract.get_health_factor(user_account.clone()),
            HEALTH_THRESHOLD,
            "Test for account w/o collaterals and borrows has been failed"
        );

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dwnear.near".to_string()),
            WBalance::from(balance),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(0),
        );

        assert_eq!(
            controller_contract.get_health_factor(user_account),
            (100 * HEALTH_THRESHOLD / 100),
            "Health factor calculation has been failed"
        );
    }
}
