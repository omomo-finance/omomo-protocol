use crate::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct MarketData {
    pub total_supplies: Balance,
    pub total_borrows: Balance,
    pub total_reserves: Balance,
    pub exchange_rate: Ratio,
    pub interest_rate: Ratio,
    pub borrow_rate: Ratio,
}

#[near_bindgen]
impl Contract {
    pub fn view_total_supplies(&self) -> Balance {
        self.get_total_supplies()
    }

    pub fn view_total_borrows(&self) -> Balance {
        self.get_total_borrows()
    }

    pub fn view_total_reserves(&self) -> Balance {
        self.get_total_reserves()
    }

    pub fn view_market_data(&self, ft_balance_of: WBalance) -> MarketData {
        let total_supplies = self.get_total_supplies();
        let total_borrows = self.get_total_borrows();
        let total_reserves = self.get_total_reserves();
        let exchange_rate = self.get_exchange_rate(ft_balance_of);
        let reserve_factor = self.model.get_reserve_factor();

        let interest_rate = self.get_supply_rate(
            ft_balance_of,
            WBalance::from(total_borrows),
            WBalance::from(total_reserves),
            WBalance::from(reserve_factor),
        );
        let borrow_rate = self.get_borrow_rate(
            ft_balance_of,
            WBalance::from(total_borrows),
            WBalance::from(total_reserves),
        );

        MarketData {
            total_supplies,
            total_borrows,
            total_reserves,
            exchange_rate,
            interest_rate,
            borrow_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use general::WBalance;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    use crate::views::MarketData;
    use crate::{Config, Contract};

    pub fn init_test_env() -> Contract {
        let (dtoken_account, underlying_token_account, controller_account) =
            (alice(), bob(), carol());

        let context = VMContextBuilder::new()
            .current_account_id(dtoken_account.clone())
            .signer_account_id(dtoken_account.clone())
            .is_view(false)
            .build();

        testing_env!(context);

        let mut contract = Contract::new(Config {
            initial_exchange_rate: U128(1000000),
            underlying_token_id: underlying_token_account,
            owner_id: dtoken_account,
            controller_account_id: controller_account,
        });

        contract.set_total_reserves(200);

        contract
    }

    #[test]
    fn test_view_market_data() {
        let contract = init_test_env();

        let gotten_md = contract.view_market_data(WBalance::from(1000));

        let _expected_md = MarketData {
            total_supplies: 0,
            total_borrows: 0,
            total_reserves: 200,
            exchange_rate: 1000000,
            interest_rate: 0,
            borrow_rate: 10000,
        };

        assert_eq!(
            &gotten_md.total_supplies, &_expected_md.total_supplies,
            "Market total supplies values check has been failed"
        );
        assert_eq!(
            &gotten_md.total_borrows, &_expected_md.total_borrows,
            "Market total borrows values check has been failed"
        );
        assert_eq!(
            &gotten_md.total_reserves, &_expected_md.total_reserves,
            "Market total reserves values check has been failed"
        );
        assert_eq!(
            &gotten_md.exchange_rate, &_expected_md.exchange_rate,
            "Exchange rate values check has been failed"
        );
        assert_eq!(
            &gotten_md.interest_rate, &_expected_md.interest_rate,
            "Interest rate values check has been failed"
        );
        assert_eq!(
            &gotten_md.borrow_rate, &_expected_md.borrow_rate,
            "Borrow rate values check has been failed"
        );
    }
}
