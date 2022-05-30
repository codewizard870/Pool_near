use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use crate::contract::{COIN_COUNT};

pub trait Check{
    fn check_onlyowner(&self);
    fn check_onlytreasury(&self);
    fn append_amount_history(&mut self, coin: String, amount: u128,  bAdd: bool);
    fn deposit_potinfo(&mut self, account: AccountId, coin: String, amount: u128, qualified: bool);
    fn withdraw_potinfo(&mut self, account: AccountId, coin: String, amount: u128);
    fn farm_withdraw(&mut self, account: AccountId, coin: String, amount: u128, price: [u128; COIN_COUNT]);
    fn update_farm_info( &mut self, account: AccountId, amount: u128 );

    fn deposit(&mut self, coin: String, amount: u128, qualified: bool);
}