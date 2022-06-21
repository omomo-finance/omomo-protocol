use near_sdk::env::{block_height, signer_account_id};
use crate::*;

const GAS_FOR_REPAY: Gas = Gas(120_000_000_000_000);

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RepayAmount {
    borrow: Balance,
    accumulated_interest: Balance,
}


impl Contract {
    pub fn repay(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_REPAY,
            "Prepaid gas is not enough for repay flow"
        );
        self.mutex_account_lock(Actions::Repay, token_amount, self.terra_gas(140))
    }

    pub fn post_repay(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(token_amount);
        }
        underlying_token::ft_balance_of(
            self.get_contract_address(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            TGAS,
        )
            .then(ext_self::repay_balance_of_callback(
                token_amount,
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(60),
            ))
            .into()
    }
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn repay_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::RepayFailedToGetUnderlyingBalance(
                    env::signer_account_id(),
                    Balance::from(token_amount),
                    self.get_contract_address(),
                    self.get_underlying_contract_address()
                )
            );
            self.mutex_account_unlock();

            return PromiseOrValue::Value(token_amount);
        }

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };
        let borrow_rate = self.get_borrow_rate(
            U128(balance_of - Balance::from(token_amount)),
            U128(self.get_total_borrows()),
            U128(self.get_total_reserves()),
        );

        let borrow_amount = self.get_account_borrows(env::signer_account_id());

        let borrow_accumulated_interest = self
            .config
            .get()
            .unwrap()
            .interest_rate_model
            .calculate_accrued_interest(
                borrow_rate,
                self.get_account_borrows(env::signer_account_id()),
                self.get_accrued_borrow_interest(env::signer_account_id()),
            ).accumulated_interest;

        let mut accumulated_interest = 0u128;

        let mut token_to_repay = 0u128;

        let mut fund_total_reserve = 0u128;

        if borrow_accumulated_interest - token_amount.0 > 0 {
            accumulated_interest += borrow_accumulated_interest - token_amount.0;

            fund_total_reserve += token_amount.0;
        } else {
            token_to_repay += token_amount.0 - borrow_accumulated_interest;

            fund_total_reserve += borrow_accumulated_interest;
        }

        let new_borrow_accrued_interest = AccruedInterest {
            last_recalculation_block: block_height(),
            accumulated_interest,
        };

        self.set_accrued_borrow_interest(env::signer_account_id(), new_borrow_accrued_interest.clone());

        let new_total_reserve = self.get_total_reserves()
            + fund_total_reserve
            * self.model.get_reserve_factor().round_u128();
        self.set_total_reserves(new_total_reserve);


        if token_to_repay > borrow_amount {
            token_to_repay = borrow_amount;
        }


        controller::repay_borrows(
            env::signer_account_id(),
            self.get_contract_address(),
            U128(token_to_repay),
            new_borrow_accrued_interest.last_recalculation_block,
            U128::from(borrow_rate),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(5),
        )
            .then(ext_self::controller_repay_borrows_callback(
                U128(token_to_repay),
                U128(token_amount.0),
                U128(borrow_accumulated_interest),
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(20),
            ))
            .into()
    }

    #[private]
    pub fn controller_repay_borrows_callback(
        &mut self,
        token_to_repay: WBalance,
        token_amount: WBalance,
        accumulated_interest: WBalance,
    ) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::RepayFailedToUpdateUserBalance(
                    env::signer_account_id(),
                    Balance::from(token_amount.0 + accumulated_interest.0)
                )
            );
            self.mutex_account_unlock();
            return PromiseOrValue::Value(token_to_repay);
        }
        let user_borrows = self.get_account_borrows(signer_account_id());


        let mut extra_balance = 0;
        let mut accumulated_interest_have_left=0;

        if token_amount.0 < accumulated_interest.0 {
            accumulated_interest_have_left += accumulated_interest.0 - token_amount.0;
        } else {
            if token_amount.0 - accumulated_interest.0 - user_borrows > 0 {
                extra_balance += token_amount.0 - accumulated_interest.0 - user_borrows;
            }
        }

        let accrued_borrow_interest = AccruedInterest {
            last_recalculation_block: block_height(),
            accumulated_interest: accumulated_interest_have_left,
        };

        self.set_accrued_borrow_interest(env::signer_account_id(), accrued_borrow_interest);
        self.decrease_borrows(env::signer_account_id(), token_to_repay);

        self.mutex_account_unlock();
        log!(
            "{}",
            Events::RepaySuccess(env::signer_account_id(), Balance::from(token_amount.0 + accumulated_interest.0))
        );
        PromiseOrValue::Value(U128(extra_balance))
    }
}
