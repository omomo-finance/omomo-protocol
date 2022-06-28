use crate::*;

#[near_bindgen]
impl Contract {
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        // adding new field to contract before it to migrate all the information
        let contract: Contract = env::state_read().expect("Contract is not initialized");

        contract
    }

    // Return a version of contract
    pub fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }


    #[cfg(target_arch = "wasm32")]
    pub fn upgrade(self) {
        use near_sys;

        //input is code:<Vec<u8> on REGISTER 0
        //log!("bytes.length {}", code.unwrap().len());
        const GAS_FOR_UPGRADE: u64 = 20 * TGAS.0; //gas occupied by this fn

        //after upgrade we call *pub fn migrate()* on the NEW CODE
        let current_id = env::current_account_id();
        let migrate_method_name = "migrate".as_bytes().to_vec();
        let attached_gas = env::prepaid_gas().0 - env::used_gas().0 - GAS_FOR_UPGRADE;

        unsafe {
            // Load input (new contract code) into register 0
            near_sys::input(0);

            //prepare self-call promise
            let promise_id =
                near_sys::promise_batch_create(current_id.as_bytes().len() as _, current_id.as_bytes().as_ptr() as _);

            //1st action, deploy/upgrade code (takes code from register 0)
            near_sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);

            // 2nd action, schedule a call to "migrate_with_new_field()".
            // Will execute on the **new code**
            near_sys::promise_batch_action_function_call(
                promise_id,
                migrate_method_name.len() as _,
                migrate_method_name.as_ptr() as _,
                0 as _,
                0 as _,
                0 as _,
                attached_gas,
            );
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version() {
        let contract = Contract::new(Config {
            initial_exchange_rate: U128(10000),
            underlying_token_id: "weth".parse().unwrap(),
            owner_id: "dtoken".parse().unwrap(),
            controller_account_id: "controller".parse().unwrap(),
            interest_rate_model: InterestRateModel::default(),
        });

        let current_version = "0.0.1";
        assert_eq!(contract.get_version(), current_version);
    }
}

