use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde::de::Unexpected::Option;
use near_sdk::BorshStorageKey;
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Promise, PromiseResult};

const RATIO_DECIMALS: u128 = 10_u128.pow(8);

#[ext_contract(ext_interest_rate_model)]
pub trait InterestRateModel {
    fn get_borrow_rate(
        &self,
        underlying_balance: Balance,
        total_borrows: Balance,
        total_reserve: Balance,
    ) -> U128;
}

#[ext_contract()]
pub trait ExtSelf {
    fn callback_promise_result(&self) -> U128;
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    SupportedMarkets,
    InterestRateModels,
    BorrowCaps,
    Prices,
    UserBorrowsPerToken,
    UserSupplies,
    TemporaryUserSupply,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Controller {
    supported_markets: UnorderedMap<AccountId, AccountId>,
    interest_rate_models: LookupMap<AccountId, AccountId>,
    borrow_caps: LookupMap<AccountId, u128>,
    prices: LookupMap<AccountId, U128>,
    user_borrows_per_token: UnorderedMap<AccountId, UnorderedMap<AccountId, Balance>>,
    users_supplies: UnorderedMap<AccountId, UnorderedMap<AccountId, Balance>>, // user per dtoken and balance
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            supported_markets: UnorderedMap::new(StorageKeys::SupportedMarkets),
            interest_rate_models: LookupMap::new(StorageKeys::InterestRateModels),
            borrow_caps: LookupMap::new(StorageKeys::BorrowCaps),
            prices: LookupMap::new(StorageKeys::Prices),
            user_borrows_per_token: UnorderedMap::new(StorageKeys::UserBorrowsPerToken),
            users_supplies: UnorderedMap::new(StorageKeys::UserSupplies),
        }
    }
}

#[near_bindgen]
impl Controller {
    #[private]
    pub fn callback_promise_result(&self) -> U128 {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ASSERT|callback_promise_result:promise_results_count"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::panic_str("PANIC|callback_promise_result:PromiseResult::Failed")
            }
            PromiseResult::Successful(result) => {
                near_sdk::serde_json::from_slice::<U128>(&result).unwrap()
            }
        }
    }

    pub fn add_market(&mut self, underlying: AccountId, dtoken_address: AccountId) {
        self.supported_markets.insert(&underlying, &dtoken_address);
    }

    pub fn get_markets(&self) -> Vec<(AccountId, AccountId)> {
        return self.supported_markets.to_vec();
    }

    pub fn supply_allowed(
        &mut self,
        dtoken_address: AccountId,
        user_address: AccountId,
        amount: u128,
    ) -> bool {
        true
    }

    pub fn borrow_allowed(
        &self,
        user_address: AccountId,
        dtoken_address: AccountId,
        total_reserve: Balance,
        amount: u128,
    ) -> bool {
        let amount_usd = self.get_price(dtoken_address.clone()).0 * amount / RATIO_DECIMALS;
        self.get_account_theoretical_liquidity(user_address.clone(), dtoken_address.clone(), total_reserve) - amount_usd >= 0
    }

    pub fn set_interest_rate_model(
        &mut self,
        dtoken_address: AccountId,
        interest_rate_model_address: AccountId,
    ) {
        self.interest_rate_models
            .insert(&dtoken_address, &interest_rate_model_address);
    }

    pub fn get_interest_rate(
        &mut self,
        dtoken_address: AccountId,
        underlying_balance: Balance,
        total_borrows: Balance,
        total_reserve: Balance,
    ) -> Promise {
        assert!(self.interest_rate_models.contains_key(&dtoken_address));

        let interest_rate_model_address = self.interest_rate_models.get(&dtoken_address).unwrap();

        ext_interest_rate_model::get_borrow_rate(
            underlying_balance,
            total_borrows,
            total_reserve,
            interest_rate_model_address,
            0,                        // attached yocto NEAR
            5_000_000_000_000.into(), // attached gas
        )
        .then(ext_self::callback_promise_result(
            env::current_account_id(), // this contract's account id
            0,                         // yocto NEAR to attach to the callback
            5_000_000_000_000.into(),  // gas to attach to the callback
        ))
    }

    pub fn set_borrow_cap(&mut self, dtoken_address: AccountId, decimal: u128) {
        self.borrow_caps.insert(&dtoken_address, &decimal);
    }

    pub fn has_collaterall(&mut self, user_address: AccountId) -> bool {
        // calling dToken contract
        true
    }

    pub fn set_price(&mut self, dtoken_address: AccountId, price: U128) {
        self.prices.insert(&dtoken_address, &price);
    }

    pub fn get_price(&self, dtoken_address: AccountId) -> U128 {
        assert!(
            self.interest_rate_models.contains_key(&dtoken_address),
            "price for token not found"
        );

        return self.prices.get(&dtoken_address).unwrap().into();
    }

    fn get_sum(user_address: AccountId, container: &UnorderedMap<AccountId, UnorderedMap<AccountId, Balance>>) -> Balance
    {
        let sum = match container.get(&user_address.clone()) {
            None => 0,
            Some(value) => {
                let mut sum = 0;
                for supplyPerMarket in value.values_as_vector().iter() {
                    sum += supplyPerMarket;
                }

                sum
            }
        };

        sum
    }


    pub fn get_account_theoretical_liquidity(&self, user_address: AccountId, dtoken_address: AccountId, total_reserve: Balance) -> u128 {
        let price = self.get_price(user_address.clone());

        let total_supplies = Controller::get_sum(user_address.clone(), &self.users_supplies);
        let total_borrows = Controller::get_sum(user_address.clone(), &self.user_borrows_per_token);

        let usd_sum_of_supplies = total_supplies * price.0;
        let usd_sum_of_borrows = total_borrows * price.0;

        self.get_interest_rate();

        0
    }

    pub fn set_user_borrows_per_token(
        &mut self,
        user_address: AccountId,
        dtoken_address: AccountId,
        amount: U128,
    ) {
        match self.user_borrows_per_token.get(&user_address) {
            None => {
                let mut tmp: UnorderedMap<AccountId, u128> = UnorderedMap::new(b"z".to_vec());
                tmp.insert(&dtoken_address, &amount.0);

                self.user_borrows_per_token.insert(&user_address, &tmp);
            }
            Some(_) => {
                self.user_borrows_per_token
                    .get(&user_address)
                    .unwrap()
                    .insert(&dtoken_address, &amount.0);
            }
        }
    }

    fn check_market(&self, user_address: AccountId) {
        let markets = self.supported_markets.values_as_vector();
        let mut is_found_market = false;
        for market in markets.iter() {
            if market == env::predecessor_account_id() {
                is_found_market = true
            }
        }

        if !is_found_market {
            env::panic_str("DToken address is not part of allowed markets")
        }
    }

    // Interface for working with custom data structure
    pub fn increase_user_supply(
        &mut self,
        user_address: AccountId,
        dtoken: AccountId,
        amount: Balance,
    ) {
        self.check_market(user_address.clone());

        match self.users_supplies.get(&user_address) {
            None => {
                let mut dtoken_per_supply: UnorderedMap<AccountId, Balance> =
                    UnorderedMap::new(StorageKeys::TemporaryUserSupply);
                dtoken_per_supply.insert(&dtoken, &amount);
                self.users_supplies
                    .insert(&user_address, &dtoken_per_supply);
            }
            Some(mut value) => {
                let new_amount = value.get(&dtoken).unwrap() + amount;
                value.insert(&dtoken, &new_amount);
            }
        }
    }

    pub fn decrease_user_supply(
        &mut self,
        user_address: AccountId,
        dtoken: AccountId,
        amount: Balance,
    ) {
        self.check_market(user_address.clone());

        match self.users_supplies.get(&user_address) {
            None => {
                env::panic_str("Cannot decrease amount of user that not stored");
            }
            Some(mut value) => {
                let new_amount = value.get(&dtoken).unwrap() + amount;
                if new_amount >= 0 {
                    value.insert(&dtoken, &new_amount);
                } else {
                    env::panic_str("Tried to set negative supply to user");
                }
            }
        }
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    // TESTS HERE
}
