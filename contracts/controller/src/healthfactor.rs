use crate::*;

impl Contract {
    pub fn get_price_sum(&self, map_raw: Option<UnorderedMap<AccountId, Balance>>) -> Balance {
        let mut result: Balance = 0;
        if let Some(map) = map_raw {
            for (asset, balance) in map.iter() {
                let price = self.get_price(asset).unwrap();
                result += Percentage::from(Percent::from(price.volatility))
                    .apply_to(Balance::from(price.value) * balance / Balance::from(10u128.pow(price.fraction_digits)));
            }
        }
        return result;
    }

    fn get_account_sum_per_action(&self, user_account: AccountId, action: ActionType) -> Balance {
        let map_raw: Option<UnorderedMap<AccountId, Balance>> = match action {
            ActionType::Supply => self.account_supplies.get(&user_account),
            ActionType::Borrow => self.account_borrows.get(&user_account),
        };

        return self.get_price_sum(map_raw);
    }
}

#[near_bindgen]
impl Contract{
    pub fn get_health_factor(&self, user_account: AccountId) -> Ratio {
        let mut ratio = RATIO_DECIMALS;
        let collaterals = self.get_account_sum_per_action(user_account.clone(), ActionType::Supply);
        let borrows = self.get_account_sum_per_action(user_account.clone(), ActionType::Borrow);

        if borrows != 0 {
            ratio = collaterals * RATIO_DECIMALS / borrows;
        }

        return ratio;
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

        let mut prices: Vec<Price> = Vec::new();
        prices.push(Price {
            asset_id: AccountId::new_unchecked("wnear.near".to_string()),
            value: U128(20000),
            volatility: U128(80),
            fraction_digits: 4
        });
        prices.push(Price {
            asset_id: AccountId::new_unchecked("weth.near".to_string()),
            value: U128(20000),
            volatility: U128(100),
            fraction_digits: 4
        });

        controller_contract.oracle_on_data(PriceJsonList {
            block_height: 83452949,
            price_list: prices,
        });

        let token_address: AccountId = AccountId::new_unchecked("near".to_string());

        return (controller_contract, token_address, user_account);
    }

    #[test]
    fn test_for_get_price_sum() {
        let (controller_contract, _token_address, _user_account) = init();

        let balance: Balance = 100;

        let raw_map_empty: UnorderedMap<AccountId, Balance> = UnorderedMap::new(b"t");
        let mut raw_map: UnorderedMap<AccountId, Balance> = UnorderedMap::new(b"t");

        assert_eq!(
            controller_contract.get_price_sum(None),
            0,
            "Test for None Option has been failed"
        );

        assert_eq!(
            controller_contract.get_price_sum(Some(raw_map_empty)),
            0,
            "Test for None Option has been failed"
        );

        raw_map.insert(
            &AccountId::new_unchecked("wnear.near".to_string()),
            &balance,
        );

        assert_eq!(
            controller_contract.get_price_sum(Some(raw_map)),
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
            RATIO_DECIMALS,
            "Test for account w/o collaterals and borrows has been failed"
        );

        controller_contract.increase_supplies(
            user_account.clone(),
            AccountId::new_unchecked("wnear.near".to_string()),
            WBalance::from(balance),
        );

        controller_contract.increase_borrows(
            user_account.clone(),
            AccountId::new_unchecked("weth.near".to_string()),
            WBalance::from(0),
        );

        assert_eq!(
            controller_contract.get_health_factor(user_account.clone()),
            (100 * RATIO_DECIMALS / 100),
            "Health factor calculation has been failed"
        );
    }
}
