use crate::Contract;

use near_sdk::json_types::U128;

const SYSTEM_DECIMALS: u32 = 24;

impl Contract {
    pub fn to_decimals_system(&self, amount: U128) -> U128 {
        if self.underlying_token_decimals != 24 {
            U128::from(
                amount.0 / 10_u128.pow(self.underlying_token_decimals as u32)
                    * 10_u128.pow(SYSTEM_DECIMALS),
            )
        } else {
            amount
        }
    }

    pub fn to_decimals_token(&self, amount: U128) -> U128 {
        if self.underlying_token_decimals != 24 {
            U128::from(
                amount.0 / 10_u128.pow(SYSTEM_DECIMALS)
                    * 10_u128.pow(self.underlying_token_decimals as u32),
            )
        } else {
            amount
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::InterestRateModel;
    use general::ratio::Ratio;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};

    use crate::{Config, Contract};

    pub fn init_test_env() -> Contract {
        let (dtoken_account, underlying_token_account, controller_account) =
            (alice(), bob(), carol());

        Contract::new(Config {
            initial_exchange_rate: U128::from(Ratio::one()),
            underlying_token_id: underlying_token_account,
            underlying_token_decimals: 6,
            owner_id: dtoken_account,
            controller_account_id: controller_account,
            interest_rate_model: InterestRateModel::default(),
            disable_transfer_token: true,
        })
    }

    #[test]
    fn test_conversion_to_decimals_system() {
        let contract = init_test_env();

        let decimals_token = 6;
        let decimals_system = 24;

        let amount_with_decimals_token = U128(200 * 10_u128.pow(decimals_token));
        let amount_with_decimals_system = contract.to_decimals_system(amount_with_decimals_token);

        assert_eq!(
            amount_with_decimals_system,
            U128(200 * 10_u128.pow(decimals_system))
        );
    }

    #[test]
    fn test_conversion_to_decimals_token() {
        let contract = init_test_env();

        let decimals_token = 6;
        let decimals_system = 24;

        let amount_with_decimals_system = U128(200 * 10_u128.pow(decimals_system));
        let amount_with_decimals_token = contract.to_decimals_token(amount_with_decimals_system);

        assert_eq!(
            amount_with_decimals_token,
            U128(200 * 10_u128.pow(decimals_token))
        );
    }
}
