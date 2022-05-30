use crate::*;
use near_sys;

#[near_bindgen]
impl Contract {
    /// For next version upgrades, change this function.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let contract: Contract = env::state_read().expect("Contract is not initialized");
        contract
    }

    /// Returns semver of this contract.
    pub fn view_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

mod upgrade {
    use near_sdk::{require, Gas};

    use super::*;
    use near_sys as sys;

    const GAS_TO_COMPLETE_UPGRADE_CALL: Gas = Gas(Gas::ONE_TERA.0 * 10);
    const GAS_FOR_GET_CONFIG_CALL: Gas = Gas(Gas::ONE_TERA.0 * 5);
    const MIN_GAS_FOR_MIGRATE_STATE_CALL: Gas = Gas(Gas::ONE_TERA.0 * 10);

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub extern "C" fn upgrade() {
        env::setup_panic_hook();
        let contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.is_valid_admin_call();
        let current_account_id = env::current_account_id().as_bytes().to_vec();
        let migrate_method_name = b"migrate".to_vec();
        let empty_args = b"{}".to_vec();
        unsafe {
            sys::input(0);
            let promise_id = sys::promise_batch_create(
                current_account_id.len() as _,
                current_account_id.as_ptr() as _,
            );
            sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // Gas required to complete this call.
            let required_gas =
                env::used_gas() + GAS_TO_COMPLETE_UPGRADE_CALL + GAS_FOR_GET_CONFIG_CALL;
            require!(
                env::prepaid_gas() >= required_gas + MIN_GAS_FOR_MIGRATE_STATE_CALL,
                "Not enough gas to complete state migration"
            );
            let migrate_state_attached_gas = env::prepaid_gas() - required_gas;
            // Scheduling state migration.
            sys::promise_batch_action_function_call(
                promise_id,
                migrate_method_name.len() as _,
                migrate_method_name.as_ptr() as _,
                empty_args.len() as _,
                empty_args.as_ptr() as _,
                0 as _,
                migrate_state_attached_gas.0,
            );

            sys::promise_return(promise_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_version() {
        let contract = Contract::new(Config {
            initial_exchange_rate: U128(10000),
            underlying_token_id: "weth".parse().unwrap(),
            owner_id: "dtoken".parse().unwrap(),
            controller_account_id: "controller".parse().unwrap(),
            interest_rate_model: InterestRateModel::default(),
        });

        let current_version = "0.0.1";
        assert_eq!(contract.view_version(), current_version);
    }
}
