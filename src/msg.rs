use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use std::fmt;

use crate::contract::{COIN_COUNT};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AprInfo{
	pub apr: u16,
    pub time: u64,
}
impl fmt::Debug for AprInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(apr:{}, time:{})", self.apr, self.time)
    }
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct UserInfo{
	pub account: AccountId,
	pub amount: u128,
	pub reward_amount: u128,
    pub deposit_time: u64,
    pub withdraw_reserve: u128,
}
impl fmt::Debug for UserInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(account:{}, amount:{}, reward_amount:{}, deposit_time:{})", self.account, self.amount, self.reward_amount, self.deposit_time)
    }
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AmountInfo{
    pub amount: Vec<u128>,
    pub reward: Vec<u128>,
    pub time: u64,
}
impl fmt::Debug for AmountInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(near_amount:{}, near_reward:{}, time:{})", self.amount[0], self.reward[0], self.time)
    }
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct FarmInfo{
    pub account: AccountId,
	pub amount: u128,
}
impl fmt::Debug for FarmInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(account:{}, amount:{})", self.account, self.amount)
    }
}


#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct PotInfo{
    pub account: AccountId,
    pub amount: u128,
    pub qualified_amount: u128,
}
impl fmt::Debug for PotInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(account:{}, amount:{}, qualified_amount:{})", self.account, self.amount, self.qualified_amount)
    }
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct DepositParam{
    pub coin: String,
    pub qualified: bool,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct WithdrawParam{
    pub account: AccountId,
    pub coin: String,
    pub price: [u128; COIN_COUNT],
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Status{
    pub amount_history: Vec<AmountInfo>,

    pub user_info: Vec<UserInfo>,
    pub farm_price: u128,
    pub farm_info: FarmInfo,
    pub farm_starttime: u64,
    pub total_rewards: Vec<u128>,
    pub pot_info: Vec<PotInfo>,
}
