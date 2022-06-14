use crate::*;
use general::ratio::Ratio;
use std::cmp::min;

#[near_bindgen]
impl Contract {
    pub fn get_supply_rate(
        &self,
        underlying_balance: WBalance,
        total_borrows: WBalance,
        total_reserves: WBalance,
        reserve_factor: Ratio,
    ) -> Ratio {
        let max_reserve_factor_value = Ratio::one();

        assert!(
            reserve_factor <= max_reserve_factor_value,
            "Reserve factor should be less {}",
            max_reserve_factor_value
        );
        let rest_of_supply_factor = Ratio::one() - reserve_factor;
        let borrow_rate = self.get_borrow_rate(underlying_balance, total_borrows, total_reserves);
        let rate_to_pool = borrow_rate * rest_of_supply_factor / Ratio::one();
        let util_rate = self.get_util_rate(underlying_balance, total_borrows, total_reserves);

        util_rate * rate_to_pool / Ratio::one()
    }

    pub fn get_borrow_rate(
        &self,
        underlying_balance: WBalance,
        total_borrows: WBalance,
        total_reserves: WBalance,
    ) -> Ratio {
        let util = self.get_util_rate(underlying_balance, total_borrows, total_reserves);
        let interest_rate_model = self.config.get().unwrap().interest_rate_model;
        let kink = interest_rate_model.get_kink();
        let multiplier_per_block = interest_rate_model.get_multiplier_per_block();
        let base_rate_per_block = interest_rate_model.get_base_rate_per_block();
        let jump_multiplier_per_block = interest_rate_model.get_jump_multiplier_per_block();

        let mut multiplier = Ratio::zero();

        if util > kink {
            multiplier = util - kink;
        }

        min(util, kink) * multiplier_per_block / Ratio::one()
            + multiplier * jump_multiplier_per_block / Ratio::one()
            + base_rate_per_block
    }

    fn get_util_rate(
        &self,
        underlying_balance: WBalance,
        total_borrows: WBalance,
        total_reserves: WBalance,
    ) -> Ratio {
        let denominator = Balance::from(underlying_balance) + Balance::from(total_borrows)
            - Balance::from(total_reserves);
        // this may happen when there is no supplies
        if denominator == 0u128 {
            return Ratio::zero();
        }

        Ratio::from(total_borrows.0) * Ratio::one() / Ratio::from(denominator)
    }
}

#[cfg(test)]
mod tests {
    use crate::InterestRateModel;
    use general::ratio::Ratio;
    use general::WRatio;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use std::str::FromStr;

    use crate::{Config, Contract};

    pub fn init_test_env() -> Contract {
        let (user_account, underlying_token_account, controller_account) =
            (alice(), bob(), carol());

        Contract::new(Config {
            initial_exchange_rate: U128::from(Ratio::one()),
            underlying_token_id: underlying_token_account,
            owner_id: user_account,
            controller_account_id: controller_account,
            interest_rate_model: InterestRateModel::default(),
        })
    }

    #[test]
    fn test_get_util_rate() {
        let contract = init_test_env();

        assert_eq!(
            contract.get_util_rate(U128(20), U128(180), U128(0)),
            Ratio::from_str("0.9").unwrap()
        );
    }

    #[test]
    fn test_get_borrow_rate() {
        let contract = init_test_env();

        let mut interest_rate_model = contract.config.get().unwrap().interest_rate_model;

        interest_rate_model.set_base_rate_per_block(WRatio::from(0));
        interest_rate_model.set_multiplier_per_block(WRatio::from(500000000));
        interest_rate_model.set_kink(WRatio::from(800000000000000000000000));
        interest_rate_model.set_jump_multiplier_per_block(WRatio::from(10900000000));

        assert_eq!(
            contract.get_borrow_rate(U128(20), U128(180), U128(0)),
            Ratio::from_str("1.9").unwrap()
        );
    }

    #[test]
    fn test_get_supply_rate() {
        let contract = init_test_env();

        let mut interest_rate_model = contract.config.get().unwrap().interest_rate_model;

        interest_rate_model.set_base_rate_per_block(WRatio::from(0));
        interest_rate_model.set_multiplier_per_block(WRatio::from(500000000));
        interest_rate_model.set_kink(WRatio::from(800000000000000000000000));
        interest_rate_model.set_jump_multiplier_per_block(WRatio::from(10900000000));
        interest_rate_model.set_reserve_factor(WRatio::from(700000000));

        assert_eq!(
            contract.get_supply_rate(
                U128(20),
                U128(180),
                U128(0),
                interest_rate_model.get_reserve_factor(),
            ),
            Ratio::from_str("1.709999999999998803").unwrap()
        );
    }
}
