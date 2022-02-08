use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_exchange_rate(&self, underlying_balance: Balance) -> Balance {
        if self.token.total_supply == 0 {
            return self.initial_exchange_rate;
        }
        return (underlying_balance + self.total_borrows - self.total_supplies)
            / self.token.total_supply;
    }

    pub fn get_total_supplies(&self) -> Balance {
        return self.total_supplies;
    }

    pub fn get_total_borrows(&self) -> Balance {
        return self.total_borrows;
    }

    #[private]
    pub fn set_total_supplies(&mut self, amount: Balance) -> Balance {
        self.total_supplies = amount;
        return self.get_total_supplies();
    }

    #[private]
    pub fn set_total_borrows(&mut self, amount: Balance) -> Balance {
        self.total_borrows = amount;
        return self.get_total_borrows();
    }
}
