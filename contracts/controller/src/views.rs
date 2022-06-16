use crate::borrows_supplies::ActionType::Supply;
use crate::*;
use std::collections::HashMap;

use crate::admin::Market;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct AccountData {
    pub account_id: AccountId,
    pub total_borrows_usd: USD,
    pub total_supplies_usd: USD,
    pub total_available_borrows_usd: USD,
    pub blocked: bool,
    pub health_factor_ratio: WRatio,
    pub user_profile: WrappedUserProfile,
}

impl Default for AccountData {
    fn default() -> Self {
        AccountData {
            account_id: AccountId::new_unchecked("".to_string()),
            total_borrows_usd: U128(0),
            total_supplies_usd: U128(0),
            total_available_borrows_usd: U128(0),
            blocked: false,
            health_factor_ratio: WRatio::from(Ratio::one()),
            user_profile: Default::default(),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn view_total_borrows_usd(&self, user_id: AccountId) -> USD {
        self.get_total_borrows(user_id)
    }

    pub fn view_total_supplies_usd(&self, user_id: AccountId) -> USD {
        self.get_total_supplies(user_id)
    }

    pub fn view_markets(&self) -> Vec<Market> {
        self.get_markets_list()
    }

    pub fn view_accounts_with_borrows(&self) -> Vec<AccountData> {
        let users = self
            .user_profiles
            .iter()
            .filter(|(_, user_profile)| !user_profile.account_borrows.is_empty())
            .map(|(account_id, _)| account_id)
            .collect::<Vec<AccountId>>();

        self.view_accounts(users)
    }

    pub fn view_accounts(&self, user_ids: Vec<AccountId>) -> Vec<AccountData> {
        return user_ids
            .iter()
            .filter(|user_id| self.user_profiles.get(user_id).is_some())
            .map(|user_id| {
                let total_borrows = self.get_total_borrows(user_id.clone());
                let total_supplies = self.get_total_supplies(user_id.clone());

                let total_available_borrows_usd =
                    (Ratio::from(total_supplies) / self.liquidation_threshold).into();

                let health_factor = self.get_health_factor(user_id.clone());
                let user_profile = self.user_profiles.get(user_id).unwrap().get_wrapped();
                AccountData {
                    account_id: user_id.clone(),
                    total_borrows_usd: total_borrows,
                    total_available_borrows_usd,
                    total_supplies_usd: total_supplies,
                    blocked: false,
                    health_factor_ratio: WRatio::from(health_factor),
                    user_profile,
                }
            })
            .collect::<Vec<AccountData>>();
    }

    pub fn view_prices(&self, dtokens: Vec<AccountId>) -> HashMap<AccountId, Price> {
        self.get_prices_for_dtokens(dtokens)
    }

    pub fn view_borrow_max(&self, user_id: AccountId, dtoken_id: AccountId) -> WBalance {
        let supplies = self.get_total_supplies(user_id.clone());
        let gotten_borrow = self.get_total_borrows(user_id);

        let potential_borrow =
            Ratio::from(supplies.0) / self.liquidation_threshold - Ratio::from(gotten_borrow.0);
        let price = Ratio::from(self.get_price(dtoken_id).unwrap().value.0);

        WBalance::from(potential_borrow / price)
    }

    pub fn view_withdraw_max(&self, user_id: AccountId, dtoken_id: AccountId) -> WBalance {
        let supplies = self.get_total_supplies(user_id.clone());
        let borrows = self.get_total_borrows(user_id.clone());
        let accrued_interest = self.calculate_accrued_borrow_interest(user_id.clone());
        let supply_by_token = self.get_entity_by_token(Supply, user_id, dtoken_id.clone());

        let max_withdraw = supplies.0
            - (borrows.0
                + accrued_interest * self.liquidation_threshold.round_u128()
                    / Ratio::one().round_u128());
        let price = self.get_price(dtoken_id).unwrap().value.0;
        let max_withdraw_in_token =
            max_withdraw * Ratio::from(ONE_TOKEN).round_u128() / Ratio::from(price).round_u128();
        if supply_by_token <= max_withdraw_in_token {
            supply_by_token.into()
        } else {
            max_withdraw_in_token.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ActionType::{Borrow, Supply};
    use crate::{Config, Contract, OraclePriceHandlerHook, PriceJsonList};
    use general::{Price, ONE_TOKEN};
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId};

    pub fn init_test_env() -> (Contract, AccountId, AccountId) {
        let (owner_account, _oracle_account, user_account) = (alice(), bob(), carol());

        let mut controller_contract = Contract::new(Config {
            owner_id: owner_account.clone(),
            oracle_account_id: owner_account.clone(),
        });

        let context = VMContextBuilder::new()
            .signer_account_id(owner_account.clone())
            .predecessor_account_id(owner_account)
            .build();

        testing_env!(context);

        let ticker_id_1 = "weth".to_string();
        let asset_id_1 = AccountId::new_unchecked("token.weth".to_string());
        let dtoken_id_1 = AccountId::new_unchecked("dtoken.weth".to_string());

        let ticker_id_2 = "wnear".to_string();
        let asset_id_2 = AccountId::new_unchecked("token.wnear".to_string());
        let dtoken_id_2 = AccountId::new_unchecked("dtoken.wnear".to_string());

        controller_contract.add_market(asset_id_1, dtoken_id_1, ticker_id_1.clone());
        controller_contract.add_market(asset_id_2, dtoken_id_2, ticker_id_2.clone());

        let token_address: AccountId = "dtoken.wnear".parse().unwrap();

        let mut prices: Vec<Price> = Vec::new();
        prices.push(Price {
            ticker_id: ticker_id_2,
            value: U128(20000),
            volatility: U128(80),
            fraction_digits: 4,
        });
        prices.push(Price {
            ticker_id: ticker_id_1,
            value: U128(20000),
            volatility: U128(100),
            fraction_digits: 4,
        });

        controller_contract.oracle_on_data(PriceJsonList {
            block_height: 83456999,
            price_list: prices,
        });

        (controller_contract, token_address, user_account)
    }

    #[test]
    fn test_view_markets() {
        let (near_contract, _, _) = init_test_env();

        let ticker_id_1 = "weth".to_string();
        let asset_id_1 = AccountId::new_unchecked("token.weth".to_string());
        let dtoken_id_1 = AccountId::new_unchecked("dtoken.weth".to_string());

        let ticker_id_2 = "wnear".to_string();
        let asset_id_2 = AccountId::new_unchecked("token.wnear".to_string());
        let dtoken_id_2 = AccountId::new_unchecked("dtoken.wnear".to_string());

        let accounts = near_contract.view_markets();

        assert_eq!(accounts.len(), 2, "View market response doesn't match");
        assert_eq!(
            accounts[0].asset_id, asset_id_1,
            "View market AssetId check has been failed"
        );
        assert_eq!(
            accounts[0].dtoken, dtoken_id_1,
            "View market Dtoken check has been failed"
        );
        assert_eq!(
            accounts[0].ticker_id, ticker_id_1,
            "View market Ticker check has been failed"
        );
        assert_eq!(
            accounts[1].asset_id, asset_id_2,
            "View market AssetId check has been failed"
        );
        assert_eq!(
            accounts[1].dtoken, dtoken_id_2,
            "View market Dtoken check has been failed"
        );
        assert_eq!(
            accounts[1].ticker_id, ticker_id_2,
            "View market Ticker check has been failed"
        );
    }

    #[test]
    fn test_view_accounts() {
        let (mut near_contract, token_address, _) = init_test_env();
        let mut accounts = Vec::new();

        accounts.push(alice());
        accounts.push(bob());

        near_contract.set_entity_by_token(
            Supply,
            accounts[0].clone(),
            token_address,
            100 * ONE_TOKEN,
        );
        let result = near_contract.view_accounts(accounts);

        assert_eq!(result.len(), 1, "View accounts response doesn't match");
        assert_eq!(
            result[0].account_id,
            alice(),
            "View accounts account_id check has been failed"
        );
        assert_eq!(
            result[0].total_borrows_usd,
            U128(0),
            "View accounts total borrows check has been failed"
        );
        assert_eq!(
            result[0].total_supplies_usd,
            U128(100 * 20000),
            "View accounts total supplies check has been failed"
        );

        assert_eq!(
            result[0].total_available_borrows_usd,
            // total_supplies_usd * Ratio::one() / self.liquidation_threshold
            U128(100 * 20000 * 1000000000000000000000000 / 1500000000000000000000000),
            "View accounts total supplies check has been failed"
        );

        assert_eq!(
            result[0].health_factor_ratio,
            U128(1500000000000000000000000),
            "View accounts health factor check has been failed"
        );
    }

    #[test]
    fn test_view_withdraw_max() {
        let (mut near_contract, token_address, user) = init_test_env();

        near_contract.set_entity_by_token(
            Supply,
            user.clone(),
            token_address.clone(),
            5420000000000000000000000, // in yocto == 5.42 Near
        );

        // we are able to withdraw all the supplied funds hence 5 NEAR
        assert_eq!(
            U128(5420000000000000000000000),
            near_contract.view_withdraw_max(user, token_address)
        );
    }

    #[test]
    fn test_view_borrow_max() {
        let (mut near_contract, token_address, user) = init_test_env();

        near_contract.set_entity_by_token(
            Supply,
            user.clone(),
            token_address.clone(),
            54240000000000000000000000, // in yocto == 54.24 Near
        );

        near_contract.set_entity_by_token(
            Borrow,
            user.clone(),
            token_address.clone(),
            10 * ONE_TOKEN, // in yocto == 10 Near
        );

        // max_withdraw = (54.24 * 20_000 * 10^4 / 15000) - 10 * 20_000 = 523_200;
        // amount = 523_200 / 20_000 = 26.16

        // as we borrow and supply same token the easiest way to check is
        // 50 (supplied) / 1.5 (health threshold) = 36.16
        // hence we have 36.16 - 10 = 26.16 left to borrow not to violate health threshold

        // we still have some tokens to borrow  26.16 Near
        assert_eq!(
            U128(26160000000000000000000000),
            near_contract.view_borrow_max(user, token_address)
        );
    }
}
