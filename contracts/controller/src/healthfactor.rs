use crate::*;

use std::collections::HashMap;

impl Contract {
    pub fn calculate_assets_weighted_price(&self, map: &HashMap<AccountId, Balance>) -> Balance {
        map.iter()
            .map(|(asset, balance)| {
                let price = self.get_price(asset.clone()).unwrap();

                Percentage::from(Percent::from(price.volatility)).apply_to(
                    Balance::from(price.value) * balance / 10u128.pow(price.fraction_digits),
                )
            })
            .sum()
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

        self.calculate_assets_weighted_price(&map_raw)
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_health_factor(&self, user_account: AccountId) -> Ratio {
        let collaterals = self.get_account_sum_per_action(user_account.clone(), ActionType::Supply);
        let borrows = self.get_account_sum_per_action(user_account, ActionType::Borrow);

        if borrows != 0 {
            collaterals * RATIO_DECIMALS / borrows
        } else {
            self.get_health_threshold()
        }
    }

    pub fn get_potential_health_factor(
        &self,
        user_account: AccountId,
        token_address: AccountId,
        amount: WBalance,
        action: ActionType,
    ) -> Ratio {
        let mut collaterals =
            self.get_account_sum_per_action(user_account.clone(), ActionType::Supply);
        let mut borrows = self.get_account_sum_per_action(user_account, ActionType::Borrow);

        let price = self.get_price(token_address).unwrap();
        let usd_amount = Percentage::from(Percent::from(price.volatility)).apply_to(
            Balance::from(price.value) * Balance::from(amount) / 10u128.pow(price.fraction_digits),
        );
        match action {
            ActionType::Supply => {
                collaterals -= usd_amount;
            }
            ActionType::Borrow => {
                borrows += usd_amount;
            }
        }

        if borrows != 0 {
            collaterals * RATIO_DECIMALS / borrows
        } else {
            self.get_health_threshold()
        }
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::test_env::{alice, bob};

    use super::*;

    // use crate::borrows_supplies::ActionType::{Borrow, Supply};

    fn init_price_volatility(
        near_price: u128,
        near_volatility: u128,
        eth_price: u128,
        eth_volatility: u128,
    ) -> (Contract, AccountId, AccountId) {
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
            value: U128(near_price),
            volatility: U128(near_volatility),
            fraction_digits: 4,
        });

        prices.push(Price {
            ticker_id: ticker_id_eth,
            value: U128(eth_price),
            volatility: U128(eth_volatility),
            fraction_digits: 4,
        });

        controller_contract.oracle_on_data(PriceJsonList {
            block_height: 83452949,
            price_list: prices,
        });

        let token_address: AccountId = AccountId::new_unchecked("near".to_string());

        (controller_contract, token_address, user_account)
    }

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
    fn test_calculate_assets_weighted_price_sum_empty_map() {
        let (controller_contract, _token_address, _user_account) = init();

        let raw_map_empty: HashMap<AccountId, Balance> = HashMap::new();
        assert_eq!(
            controller_contract.calculate_assets_weighted_price(&raw_map_empty),
            0,
            "Test for None Option has been failed"
        );
    }

    #[test]
    fn test_for_calculate_assets_weighted_price() {
        let (controller_contract, _token_address, _user_account) = init();

        let mut raw_map: HashMap<AccountId, Balance> = HashMap::new();
        raw_map.insert(AccountId::new_unchecked("dwnear.near".to_string()), 100);

        assert_eq!(
            controller_contract.calculate_assets_weighted_price(&raw_map),
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
            controller_contract.get_health_threshold(),
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
            (100 * controller_contract.get_health_threshold() / 100),
            "Health factor calculation has been failed"
        );
    }

    #[test]
    fn test_health_factor_wo_s_or_b() {
        let (mut controller_contract, _token_address, user_account) =
            init_price_volatility(0, 0, 10000, 100);

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(0),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(0),
        );

        assert_eq!(
            controller_contract.get_health_factor(user_account),
            controller_contract.get_health_threshold(),
            "Test for account w/o collaterals and borrows has been failed"
        );
    }

    #[test]
    fn test_health_factor_with_supply() {
        let (mut controller_contract, _token_address, user_account) =
            init_price_volatility(0, 0, 10000, 100);

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(100),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(0),
        );

        // Ratio that represents 150%
        assert_eq!(controller_contract.get_health_factor(user_account), 15000);
    }

    #[test]
    fn test_health_factor_with_supply_and_borrow_scenario_1() {
        let (mut controller_contract, _token_address, user_account) =
            init_price_volatility(0, 0, 10000, 100);

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(100),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(70),
        );

        // Ratio that represents 142.85%
        assert_eq!(controller_contract.get_health_factor(user_account), 14285);
    }

    #[test]
    fn test_health_factor_with_supply_and_borrow_scenario_2() {
        let (mut controller_contract, _token_address, user_account) =
            init_price_volatility(10000, 100, 10000, 100);

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(200),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dwnear.near".to_string()),
            WBalance::from(100),
        );

        // Ratio that represents 200%
        assert_eq!(controller_contract.get_health_factor(user_account), 20000);
    }

    #[test]
    fn test_health_factor_with_supply_and_borrow_scenario_3() {
        let (mut controller_contract, _token_address, user_account) =
            init_price_volatility(20000, 100, 5000, 100);

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(200),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dwnear.near".to_string()),
            WBalance::from(100),
        );

        // Ratio that represents 50%
        assert_eq!(controller_contract.get_health_factor(user_account), 5000);
    }

    #[test]
    fn test_health_factor_with_supply_and_borrow_scenario_4() {
        let (mut controller_contract, _token_address, user_account) =
            init_price_volatility(10000, 80, 10000, 90);

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(200),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dwnear.near".to_string()),
            WBalance::from(100),
        );

        // Ratio that represents 225%
        assert_eq!(controller_contract.get_health_factor(user_account), 22500);
    }

    #[test]
    fn test_health_factor_with_supply_and_multi_borrow() {
        let (mut controller_contract, _token_address, user_account) =
            init_price_volatility(10000, 80, 11000, 90);

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(200),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(50),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dwnear.near".to_string()),
            WBalance::from(100),
        );

        // Ratio that represents 153.48%
        assert_eq!(controller_contract.get_health_factor(user_account), 15348);
    }
}
