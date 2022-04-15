use crate::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct MarketData {
    pub total_supplies: WBalance,
    pub total_borrows: WBalance,
    pub total_reserves: WBalance,
    pub exchange_rate_ratio: WRatio,
    pub interest_rate_ratio: WRatio,
    pub borrow_rate_ratio: WRatio,
}

#[near_bindgen]
impl Contract {
    pub fn view_total_supplies(&self) -> WBalance {
        WBalance::from(self.get_total_supplies())
    }

    pub fn view_total_borrows(&self) -> WBalance {
        WBalance::from(self.get_total_borrows())
    }

    pub fn view_total_reserves(&self) -> WBalance {
        WBalance::from(self.get_total_reserves())
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
            total_supplies: WBalance::from(total_supplies),
            total_borrows: WBalance::from(total_borrows),
            total_reserves: WBalance::from(total_reserves),
            exchange_rate_ratio: WRatio::from(exchange_rate),
            interest_rate_ratio: WRatio::from(interest_rate),
            borrow_rate_ratio: WRatio::from(borrow_rate),
        }
    }

    pub fn view_repay_info(&self, ft_balance: WBalance) -> RepayInfo {
        self.get_repay_info(ft_balance)
    }
}

#[cfg(test)]
mod tests {
    use crate::InterestRateModel;
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
            interest_rate_model: InterestRateModel::default(),
        });

        contract.set_total_reserves(200);

        contract
    }

    #[test]
    fn test_view_market_data() {
        let contract = init_test_env();

        let gotten_md = contract.view_market_data(WBalance::from(1000));

        let _expected_md = MarketData {
            total_supplies: U128(0),
            total_borrows: U128(0),
            total_reserves: U128(200),
            exchange_rate_ratio: U128(1000000),
            interest_rate_ratio: U128(0),
            borrow_rate_ratio: U128(10000),
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
            &gotten_md.exchange_rate_ratio, &_expected_md.exchange_rate_ratio,
            "Exchange rate values check has been failed"
        );
        assert_eq!(
            &gotten_md.interest_rate_ratio, &_expected_md.interest_rate_ratio,
            "Interest rate values check has been failed"
        );
        assert_eq!(
            &gotten_md.borrow_rate_ratio, &_expected_md.borrow_rate_ratio,
            "Borrow rate values check has been failed"
        );
    }
}
