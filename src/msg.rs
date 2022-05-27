use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::collections::LookupMap;

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AprInfo{
	pub apr: u16,
    pub time: u64,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct UserInfo{
	pub account: AccountId,
	pub amount: u128,
	pub reward_amount: u128,
    pub deposit_time: u64
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AmountInfo{
    pub near_amount: u128,
    pub near_reward: u128,
    pub time: u64,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FarmInfo{
    pub account: AccountId,
	pub amount: u128,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Status{
    // pub amount_history: Vec<AmountInfo>,
    pub near_apr_history: Vec<AprInfo>,
    // pub apr_luna_history: Vec<AprInfo>,
    // pub userinfo_ust: UserInfo,
    // pub userinfo_luna: UserInfo,
    pub farm_price: u128,
    // pub farm_info: FarmInfo,
    pub farm_starttime: u64,
    // pub total_rewards_ust: u128,
    // pub total_rewards_luna: u128,
    // pub pot_info: PotInfo,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PotInfo{
    pub account: AccountId,
    pub near_amount: u128,
    pub qualified_near_amount: u128,
}
