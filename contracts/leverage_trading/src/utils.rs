use crate::*;
use near_sdk::ext_contract;

pub const NO_DEPOSIT: Balance = 0;
pub const PROTOCOL_DECIMALS: u8 = 24;
pub const DAYS_PER_YEAR: u16 = 360;
pub const MILLISECONDS_PER_DAY: u64 = 86400000;

#[ext_contract(ext_token)]
pub trait NEP141Token {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: WBalance,
        memo: Option<String>,
        msg: String,
    );

    fn ft_transfer(&mut self, receiver_id: AccountId, amount: WBalance, memo: Option<String>);
}

impl Contract {
    pub fn get_order_by(&self, order_id: u128) -> Option<Order> {
        if let Some(account) = self.get_account_by(order_id) {
            self.orders
                .get(&account)
                .unwrap()
                .get(&(order_id as u64))
                .cloned()
        } else {
            None
        }
    }
}

#[ext_contract(ext_market)]
pub trait MarketInterface {
    fn borrow(&mut self, amount: WBalance) -> PromiseOrValue<U128>;
    fn view_market_data(&self) -> MarketData;
}
