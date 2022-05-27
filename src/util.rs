use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use crate::contract::{Pool};

pub trait Check{
    fn check_onlyowner(&self);
    fn check_onlytreasury(&self);
    fn append_amount_history(&mut self, near_amount:u128, bAdd: bool);
    fn deposit_potinfo(&mut self, account: AccountId, near_amount: u128, qualified: bool);
}