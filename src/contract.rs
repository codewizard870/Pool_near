use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::collections::LookupMap;

use crate::msg::{AprInfo, UserInfo, AmountInfo, FarmInfo, PotInfo};

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Pool {
    owner: AccountId,
    treasury: AccountId,
    vnear: AccountId,
    near_apr_history: Vec<AprInfo>,
    near_user_infos: LookupMap<AccountId, UserInfo>,
    near_total_rewards: u128,
    
    amount_history: Vec<AmountInfo>,
    
    //--------farm-----------------
    farm_starttime: u64,
    farm_price: u128,
    farm_infos: LookupMap<AccountId, FarmInfo>,
    total_farmed: u128,
    
    //--------qualify----------------------
    pot_infos: LookupMap<AccountId, PotInfo>
}

#[near_bindgen]
impl Pool {
    #[init]
    pub fn new(owner: Option<AccountId>, treasury: AccountId, near_apr: u16, vnear: AccountId ) -> Self {
        Self {
            owner: match owner{
                Some(_owner) => _owner,
                None => env::current_account_id()
            },
            treasury: treasury,
            vnear: vnear,
            near_apr_history: vec![
                AprInfo{
                    apr: near_apr,
                    time: env::block_timestamp()
                }],
            near_user_infos: LookupMap::new(b"n"),
            near_total_rewards: 0,
            amount_history: Vec::new(),
            farm_starttime: env::block_timestamp(),
            farm_price: 25,
            farm_infos: LookupMap::new(b"f"),
            total_farmed: 0,
            pot_infos: LookupMap::new(b"p")
        }
    }

    // pub fn get_solution(&self) -> String {
    //     self.crossword_solution.clone()
    // }

    // pub fn guess_solution(&mut self, solution: String) -> bool {
    //     let hashed_input = env::sha256(solution.as_bytes());
    //     let hashed_input_hex = hex::encode(&hashed_input);

    //     if hashed_input_hex == self.crossword_solution {
    //         env::log_str("You guessed right!");
    //         true
    //     } else {
    //         env::log_str("Try again.");
    //         false
    //     }
    // }
    pub fn get_status(self, account: AccountId) -> Pool{
        self
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    #[test]
    fn debug_get_hash() {
        // Basic set up for a unit test
        testing_env!(VMContextBuilder::new().build());

        // Using a unit test to rapidly debug and iterate
        let debug_solution = "near nomicon ref finance";
        let debug_hash_bytes = env::sha256(debug_solution.as_bytes());
        let debug_hash_string = hex::encode(debug_hash_bytes);
        println!("Let's debug: {:?}", debug_hash_string);
    }

    #[test]
    fn main_test() {
        // Get Alice as an account ID
        let alice = AccountId::new_unchecked("alice.testnet".to_string());
        let owner = None;
        let treasury = AccountId::new_unchecked("treasury.testnet".to_string());
        let vnear = AccountId::new_unchecked("vnear.testnet".to_string());
        let near_apr = 121;
        // Set up the testing context and unit test environment
        let context = get_context(alice.clone());
        testing_env!(context.build());

        let mut pool = Pool::new(owner, treasury, near_apr, vnear );

        let res = pool.get_status(alice);
        println!("{:?}", res.near_apr_history[0].apr);
    }
}