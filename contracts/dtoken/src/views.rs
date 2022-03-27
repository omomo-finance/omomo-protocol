use crate::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct MarketData {
    pub market_total_supplies: Balance,
    pub market_total_borrows: Balance,
    pub market_total_reserves: Balance,
    pub exchange_rate: Ratio,
    pub interest_rate: Ratio,
    pub borrow_rate: Ratio
}


#[near_bindgen]
impl Contract {

    pub fn get_market_data(&self, balance_of: WBalance) -> MarketData {

        let market_total_supplies = self.get_total_supplies();
        let market_total_borrows = self.get_total_borrows();
        let market_total_reserves = self.get_total_reserves();
        let exchange_rate = self.get_exchange_rate(balance_of);
        let reserve_factor = self.model.get_reserve_factor();

        let interest_rate = self.get_supply_rate(
            balance_of,
                WBalance::from(market_total_borrows),
                WBalance::from(market_total_reserves),
                WBalance::from(reserve_factor)
        );
        let borrow_rate = self.get_borrow_rate(
            balance_of,
            WBalance::from(market_total_borrows),
            WBalance::from(market_total_reserves),
        );

        return MarketData {
            market_total_supplies,
            market_total_borrows,
            market_total_reserves,
            exchange_rate,
            interest_rate,
            borrow_rate
        };
    }

}

#[cfg(test)]
mod tests {
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use general::WBalance;

    use crate::{Config, Contract};
    use crate::views::MarketData;

    pub fn init_test_env() -> Contract {
        let (user_account, underlying_token_account, controller_account) = (alice(), bob(), carol());

        let mut contract = Contract::new(Config {
            initial_exchange_rate: U128(100),
            underlying_token_id: underlying_token_account.clone(),
            owner_id: user_account.clone(),
            controller_account_id: controller_account.clone(),
        });

        contract.set_total_reserves(200);

        return contract;
    }

    #[test]
    fn test_get_market_data() {
        let contract = init_test_env();

        let gotten_md = contract.get_market_data(WBalance::from(1000));

        let _expected_md = MarketData {
            market_total_supplies: 0,
            market_total_borrows: 0,
            market_total_reserves: 200,
            exchange_rate: 1000000,
            interest_rate: 0,
            borrow_rate: 10000
        } ;

        assert_eq!(&gotten_md.market_total_supplies, &_expected_md.market_total_supplies, "Market total supplies values check has been failed");
        assert_eq!(&gotten_md.market_total_borrows, &_expected_md.market_total_borrows, "Market total borrows values check has been failed");
        assert_eq!(&gotten_md.market_total_reserves, &_expected_md.market_total_reserves, "Market total reserves values check has been failed");
        assert_eq!(&gotten_md.exchange_rate, &_expected_md.exchange_rate, "Exchange rate values check has been failed");
        assert_eq!(&gotten_md.interest_rate, &_expected_md.interest_rate, "Interest rate values check has been failed");
        assert_eq!(&gotten_md.borrow_rate, &_expected_md.borrow_rate, "Borrow rate values check has been failed");
    }
}