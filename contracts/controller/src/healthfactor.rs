use crate::*;

use general::ratio::Ratio;
use near_sdk::env::block_height;
use std::collections::HashMap;

impl Contract {
    pub fn calculate_supplies_weighted_price_and_lth(&self, user_id: AccountId) -> Balance {
        let supplies = self
            .user_profiles
            .get(&user_id)
            .unwrap_or_default()
            .account_supplies;

        supplies
            .iter()
            .map(|(asset, balance)| {
                let price = self.get_price(asset).unwrap();
                let market = self.markets.get(asset).unwrap();

                (BigBalance::from(price.value) * BigBalance::from(balance.to_owned()) * market.lth)
                    .round_u128()
            })
            .sum()
    }

    pub fn get_collaterals_by_borrows(&self, user_id: AccountId) -> USD {
        let borrows = self
            .user_profiles
            .get(&user_id)
            .unwrap_or_default()
            .account_borrows;

        let collaterals: Balance = borrows
            .iter()
            .map(|(asset, balance)| {
                let price = self.get_price(asset).unwrap();
                let market = self.markets.get(asset).unwrap();

                (BigBalance::from(price.value) * BigBalance::from(balance.to_owned()) / market.ltv)
                    .round_u128()
            })
            .sum();

        USD::from(collaterals)
    }

    pub fn get_theoretical_borrows_max(&self, user_id: AccountId) -> USD {
        let supplies = self
            .user_profiles
            .get(&user_id)
            .unwrap_or_default()
            .account_supplies;

        let borrow_max: Balance = supplies
            .iter()
            .map(|(asset, balance)| {
                let price = self.get_price(asset).unwrap();
                let market = self.markets.get(asset).unwrap();

                (BigBalance::from(price.value) * BigBalance::from(balance.to_owned()) * market.ltv)
                    .round_u128()
            })
            .sum();

        USD::from(borrow_max)
    }

    pub fn calculate_assets_weighted_price(&self, map: &HashMap<AccountId, Balance>) -> Balance {
        map.iter()
            .map(|(asset, balance)| {
                let price = self.get_price(asset).unwrap();

                Percentage::from(price.volatility.0).apply_to(
                    (BigBalance::from(price.value) * BigBalance::from(balance.to_owned())
                        / Ratio::from(10u128.pow(price.fraction_digits)))
                    .0
                    .low_u128(),
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

    pub fn calculate_accrued_borrow_interest(&self, account_id: AccountId) -> Balance {
        let mut total_accrued_interest = 0;
        let user_profile = self.user_profiles.get(&account_id).unwrap_or_default();
        let total_borrows: Balance = user_profile
            .account_borrows
            .iter()
            .map(|(_, balance)| balance)
            .sum();

        for (token_address, borrow_data) in user_profile.borrow_data.iter() {
            let accrued_interest = Ratio::from(total_borrows)
                * borrow_data.borrow_rate
                * Ratio::from(block_height() - borrow_data.borrow_block);

            let price = self.get_price(token_address).unwrap();
            let accrued_interest_amount = Percentage::from(price.volatility.0).apply_to(
                (BigBalance::from(price.value) * accrued_interest
                    / Ratio::from(10u128.pow(price.fraction_digits)))
                .0
                .low_u128(),
            );

            total_accrued_interest += accrued_interest_amount;
        }
        total_accrued_interest
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_health_factor(&self, user_account: AccountId) -> Ratio {
        let supplies_weighted_lth =
            self.calculate_supplies_weighted_price_and_lth(user_account.clone());
        let max_borrows = self.get_theoretical_borrows_max(user_account.clone());
        let mut borrows = self.get_account_sum_per_action(user_account.clone(), ActionType::Borrow);

        borrows += self.calculate_accrued_borrow_interest(user_account.clone());

        if borrows != 0 {
            Ratio::from(supplies_weighted_lth) / Ratio::from(borrows)
        } else {
            Ratio::from(supplies_weighted_lth) / Ratio::from(max_borrows.0)
        }
    }

    pub fn get_potential_health_factor(
        &self,
        user_account: AccountId,
        token_address: AccountId,
        amount: WBalance,
        action: ActionType,
    ) -> Ratio {
        let mut collaterals = self.calculate_supplies_weighted_price_and_lth(user_account.clone());
        let max_borrows = self.get_theoretical_borrows_max(user_account.clone());
        let mut borrows = self.get_account_sum_per_action(user_account.clone(), ActionType::Borrow);
        borrows += self.calculate_accrued_borrow_interest(user_account);

        let price = self.get_price(&token_address).unwrap();
        let usd_amount = Percentage::from(price.volatility.0).apply_to(
            (BigBalance::from(price.value) * BigBalance::from(amount.0)
                / Ratio::from(10u128.pow(price.fraction_digits)))
            .0
            .low_u128(),
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
            Ratio::from(collaterals) / Ratio::from(borrows)
        } else {
            Ratio::from(collaterals) / Ratio::from(max_borrows.0)
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
            Ratio::from_str("0.6").unwrap(),
            Ratio::from_str("0.8").unwrap(),
        );

        let utoken_address_eth = AccountId::new_unchecked("weth.near".to_string());
        let dtoken_address_eth = AccountId::new_unchecked("dweth.near".to_string());
        let ticker_id_eth = "weth".to_string();

        controller_contract.add_market(
            utoken_address_eth,
            dtoken_address_eth,
            ticker_id_eth.clone(),
            Ratio::from_str("0.6").unwrap(),
            Ratio::from_str("0.8").unwrap(),
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
            Ratio::from_str("0.6").unwrap(),
            Ratio::from_str("0.8").unwrap(),
        );

        let utoken_address_eth = AccountId::new_unchecked("weth.near".to_string());
        let dtoken_address_eth = AccountId::new_unchecked("dweth.near".to_string());
        let ticker_id_eth = "weth".to_string();

        controller_contract.add_market(
            utoken_address_eth,
            dtoken_address_eth,
            ticker_id_eth.clone(),
            Ratio::from_str("0.6").unwrap(),
            Ratio::from_str("0.8").unwrap(),
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
    fn test_for_get_health_factor_threshold() {
        let (mut controller_contract, _token_address, user_account) = init();

        let balance: Balance = 50;

        assert_eq!(
            controller_contract.get_health_factor(user_account.clone()),
            controller_contract.get_liquidation_threshold(),
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
            0,
            Ratio::zero(),
        );

        assert_eq!(
            controller_contract.get_health_factor(user_account),
            Ratio::from(100u128) * controller_contract.get_liquidation_threshold()
                / Ratio::from(100u128),
            "Health factor calculation has been failed"
        );
    }

    #[test]
    fn test_health_factor_without_supply_or_borrow() {
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
            0,
            Ratio::zero(),
        );

        assert_eq!(
            controller_contract.get_health_factor(user_account),
            controller_contract.get_liquidation_threshold(),
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
            0,
            Ratio::zero(),
        );

        // Ratio that represents 150%
        assert_eq!(
            controller_contract.get_health_factor(user_account),
            Ratio::from_str("1.5").unwrap()
        );
    }

    #[test]
    fn test_health_factor_with_supply_and_borrow() {
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
            0,
            Ratio::zero(),
        );

        // Ratio that represents 142.85714285%
        assert_eq!(
            controller_contract.get_health_factor(user_account),
            Ratio::from_str("1.428571428571428571428571").unwrap()
        );
    }

    #[test]
    fn test_health_factor_increasing_supply() {
        let (mut controller_contract, _token_address, user_account) =
            init_price_volatility(10000, 100, 10000, 100);

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(100),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dwnear.near".to_string()),
            WBalance::from(100),
            0,
            Ratio::zero(),
        );

        // Ratio that represents 100%
        assert_eq!(
            controller_contract.get_health_factor(user_account.clone()),
            Ratio::from_str("1").unwrap()
        );

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(100),
        );

        // Ratio that represents 200%
        assert_eq!(
            controller_contract.get_health_factor(user_account),
            Ratio::from_str("2").unwrap()
        );
    }

    #[test]
    fn test_health_factor_updating_price() {
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
            0,
            Ratio::zero(),
        );

        // Ratio that represents 200%
        assert_eq!(
            controller_contract.get_health_factor(user_account.clone()),
            Ratio::from_str("2").unwrap()
        );

        controller_contract.oracle_on_data(PriceJsonList {
            block_height: 83452949,
            price_list: vec![
                Price {
                    ticker_id: "wnear".to_string(),
                    value: U128(20000),
                    volatility: U128(100),
                    fraction_digits: 4,
                },
                Price {
                    ticker_id: "weth".to_string(),
                    value: U128(5000),
                    volatility: U128(100),
                    fraction_digits: 4,
                },
            ],
        });

        // Ratio that represents 50%
        assert_eq!(
            controller_contract.get_health_factor(user_account),
            Ratio::from_str("0.5").unwrap()
        );
    }

    #[test]
    fn test_health_factor_updating_volatility() {
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
            0,
            Ratio::zero(),
        );

        // Ratio that represents 200%
        assert_eq!(
            controller_contract.get_health_factor(user_account.clone()),
            Ratio::from_str("2").unwrap()
        );

        controller_contract.oracle_on_data(PriceJsonList {
            block_height: 83452949,
            price_list: vec![
                Price {
                    ticker_id: "wnear".to_string(),
                    value: U128(10000),
                    volatility: U128(80),
                    fraction_digits: 4,
                },
                Price {
                    ticker_id: "weth".to_string(),
                    value: U128(10000),
                    volatility: U128(90),
                    fraction_digits: 4,
                },
            ],
        });

        // Ratio that represents 225%
        assert_eq!(
            controller_contract.get_health_factor(user_account),
            Ratio::from_str("2.25").unwrap()
        );
    }

    #[test]
    fn test_get_potential_health_factor() {
        let (mut controller_contract, _token_address, user_account) =
            init_price_volatility(300, 59, 400, 36);

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(200),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dweth.near".to_string()),
            WBalance::from(50),
            0,
            Ratio::zero(),
        );

        let result = controller_contract.get_potential_health_factor(
            user_account,
            AccountId::new_unchecked("dweth.near".to_string()),
            U128(1000),
            ActionType::Borrow,
        );

        assert_eq!(result, Ratio::from(2u128) / Ratio::from(14u128));
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
            0,
            Ratio::zero(),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("dwnear.near".to_string()),
            WBalance::from(100),
            0,
            Ratio::zero(),
        );

        // Ratio that represents 1.534883720930232558139534%
        assert_eq!(
            controller_contract.get_health_factor(user_account),
            Ratio::from_str("1.534883720930232558139534").unwrap()
        );
    }
}
