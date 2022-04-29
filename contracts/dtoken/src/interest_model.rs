use crate::*;
use general::ratio::{Ratio, RATIO_DECIMALS};
use std::cmp::{max, min};

const MAX_RESERVE_FACTOR_VALUE: Ratio = RATIO_DECIMALS;

#[near_bindgen]
impl Contract {
    pub fn get_supply_rate(
        &self,
        underlying_balance: WBalance,
        total_borrows: WBalance,
        total_reserves: WBalance,
        reserve_factor: WBalance,
    ) -> Ratio {
        assert!(
            Balance::from(reserve_factor) <= MAX_RESERVE_FACTOR_VALUE.0,
            "Reserve factor should be less {}",
            MAX_RESERVE_FACTOR_VALUE
        );
        let rest_of_supply_factor = RATIO_DECIMALS - Ratio(reserve_factor.0);
        let borrow_rate = self.get_borrow_rate(underlying_balance, total_borrows, total_reserves);
        let rate_to_pool = borrow_rate * rest_of_supply_factor / RATIO_DECIMALS;
        let util_rate = self.get_util(underlying_balance, total_borrows, total_reserves);
        util_rate * rate_to_pool / RATIO_DECIMALS
    }

    pub fn get_borrow_rate(
        &self,
        underlying_balance: WBalance,
        total_borrows: WBalance,
        total_reserves: WBalance,
    ) -> Ratio {
        let util = self.get_util(underlying_balance, total_borrows, total_reserves);
        let interest_rate_model = self.config.get().unwrap().interest_rate_model;
        let kink = interest_rate_model.get_kink();
        let multiplier_per_block = interest_rate_model.get_multiplier_per_block();
        let base_rate_per_block = interest_rate_model.get_base_rate_per_block();
        let jump_multiplier_per_block = interest_rate_model.get_jump_multiplier_per_block();
        min(util, kink) * multiplier_per_block / RATIO_DECIMALS
            + Ratio(max(0, util.0 as i128 - kink.0 as i128) as u128) * jump_multiplier_per_block
                / RATIO_DECIMALS
            + base_rate_per_block
    }

    fn get_util(
        &self,
        underlying_balance: WBalance,
        total_borrows: WBalance,
        total_reserves: WBalance,
    ) -> Ratio {
        let sum_balance_borrows =
            Balance::from(underlying_balance).checked_add(Balance::from(total_borrows));
        assert!(
            sum_balance_borrows.is_some(),
            "Overflowing occurs while adding underlying balance and total borrows"
        );
        let denominator = sum_balance_borrows
            .unwrap()
            .checked_sub(Balance::from(total_reserves));
        assert!(denominator.is_some(), "Overflowing occurs while subtracting total reserves from sum of underlying balance and total borrows");
        assert_ne!(
            denominator.unwrap(),
            0,
            "Cannot calculate utilization rate as denominator is equal 0"
        );
        Ratio(Balance::from(total_borrows) * RATIO_DECIMALS.0 / denominator.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::InterestRateModel;
    use general::ratio::Ratio;
    use general::WRatio;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};

    use crate::{Config, Contract};

    pub fn init_test_env() -> Contract {
        let (user_account, underlying_token_account, controller_account) =
            (alice(), bob(), carol());

        Contract::new(Config {
            initial_exchange_rate: U128(10000),
            underlying_token_id: underlying_token_account,
            owner_id: user_account,
            controller_account_id: controller_account,
            interest_rate_model: InterestRateModel::default(),
        })
    }

    #[test]
    fn test_get_util_rate() {
        let contract = init_test_env();
        assert_eq!(contract.get_util(U128(20), U128(180), U128(0)), Ratio(9000));
    }

    #[test]
    fn test_get_borrow_rate() {
        let contract = init_test_env();

        let mut interest_rate_model = contract.config.get().unwrap().interest_rate_model;

        interest_rate_model.set_base_rate_per_block(WRatio::from(0));
        interest_rate_model.set_multiplier_per_block(WRatio::from(500));
        interest_rate_model.set_kink(WRatio::from(8000));
        interest_rate_model.set_jump_multiplier_per_block(WRatio::from(10900));

        assert_eq!(
            contract.get_borrow_rate(U128(20), U128(180), U128(0)),
            Ratio(19000)
        );
    }

    #[test]
    fn test_get_supply_rate() {
        let contract = init_test_env();

        let mut interest_rate_model = contract.config.get().unwrap().interest_rate_model;

        interest_rate_model.set_base_rate_per_block(WRatio::from(0));
        interest_rate_model.set_multiplier_per_block(WRatio::from(500));
        interest_rate_model.set_kink(WRatio::from(8000));
        interest_rate_model.set_jump_multiplier_per_block(WRatio::from(10900));
        interest_rate_model.set_reserve_factor(WRatio::from(700));

        assert_eq!(
            contract.get_supply_rate(
                U128(20),
                U128(180),
                U128(0),
                U128(interest_rate_model.get_reserve_factor().0),
            ),
            Ratio(15903)
        );
    }
}
