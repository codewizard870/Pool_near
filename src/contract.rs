use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise, PromiseOrValue, require, log, 
    Gas};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

use crate::msg::{AprInfo, UserInfo, AmountInfo, FarmInfo, PotInfo, Status};
use crate::util::{Check};

const BASE_GAS: u64 = 5_000_000_000_000;
const PROMISE_CALL: u64 = 5_000_000_000_000;
const GAS_FOR_FT_ON_TRANSFER: Gas = Gas(BASE_GAS + PROMISE_CALL);

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

    pub fn set_config(&mut self, owner: Option<AccountId>, treasury: Option<AccountId>, vnear: Option<AccountId>){
        if let Some(account) = owner {
            self.owner = account;
        }
        if let Some(account) = treasury {
            self.treasury = account;
        }
        if let Some(account) = vnear {
            self.vnear = account;
        }
    }

    pub fn set_apr_near(&mut self, apr: u16 ){
        self.check_onlyowner();

        let apr_info = AprInfo{
            apr,
            time: env::block_timestamp(),
        };
    
        self.near_apr_history.push(apr_info);
    }

    #[payable]
    pub fn deposit_near(&mut self, qualified: bool) {
        let account = env::predecessor_account_id();
        let fund = env::attached_deposit();
    
        if fund == 0 {
            env::panic_str("None deposit");
        }
    
        if let Some(mut user_info) = self.near_user_infos.get(&account){
            user_info.amount += fund;
            user_info.deposit_time = env::block_timestamp();
            self.near_user_infos.insert(&account, &user_info);
        } else {
            self.near_user_infos.insert(&account,
                &UserInfo{
                    account: account.clone(),
                    amount: fund,
                    reward_amount: 0,
                    deposit_time: env::block_timestamp(),
                }
            );
        }

        self.append_amount_history(fund, true);
        self.deposit_potinfo(account, fund, qualified);
    
        Promise::new(self.treasury.clone()).transfer(fund);
        // Promise::new(self.vnear.clone()).function_call("Mint", arguments: Vec<u8>, amount: Balance, gas: Gas)
        // let send2_treasury = BankMsg::Send { 
        //     to_address: TREASURY.load(deps.storage)?.to_string(),
        //     amount: _fund
        // };
    
        // let mint2_user = WasmMsg::Execute { 
        //     contract_addr: VUST.load(deps.storage)?.to_string(), 
        //     msg: to_binary(
        //         &Cw20ExecuteMsg::Mint{
        //             recipient: wallet.to_string(), 
        //             amount: fund.amount
        //         }
        //     )?, 
        //     funds: vec![]
        // };
    
        // Ok(Response::new()
        //     .add_attribute("action", "desposit")
        //     .add_messages([
        //         CosmosMsg::Bank(send2_treasury), 
        //         CosmosMsg::Wasm(mint2_user)
        //     ])
        //     .add_attribute("amount", fund.amount.to_string())
        // )
    }
    
    pub fn get_status(self, account: AccountId) -> Status{
        Status {
            near_apr_history: self.near_apr_history,
            farm_price: self.farm_price,
            farm_starttime: self.farm_starttime
        }
    }
}

impl Check for Pool{
    fn check_onlyowner(&self){
        if self.owner != env::predecessor_account_id() {
            env::panic_str("Not Authorized")
        }
    }
    fn check_onlytreasury(&self){
        if self.treasury != env::predecessor_account_id() {
            env::panic_str("Only treasury")
        }
    }

    fn append_amount_history(&mut self, near_amount: u128,  bAdd: bool){
        if self.amount_history.len() == 0 {
            self.amount_history.push(AmountInfo{
                near_amount,
                near_reward: 0,
                time: env::block_timestamp()
            });
        } 
        else {
            let last_index = self.amount_history.len() - 1;
            let mut info = self.amount_history[last_index].clone();
            if bAdd {
                info.near_amount += near_amount;
            } else {
                info.near_amount -= near_amount;
            }
            info.time = env::block_timestamp();
            info.near_reward = self.near_total_rewards;

            self.amount_history.push(info);

            if last_index > 50 {
                let mut retain = vec![true; self.amount_history.len()];
                retain[0] = false;

                let mut iter = retain.iter();
                self.amount_history.retain(|_| *iter.next().unwrap());
            }
        }
    }

    fn deposit_potinfo(&mut self, account: AccountId, near_amount: u128, qualified: bool){
        let mut pot_info = if let Some(info) = self.pot_infos.get(&account) {
                info
            } else {
                PotInfo{
                    account: account.clone(),
                    near_amount: 0,
                    qualified_near_amount: 0
                }
            };

        if qualified {
            pot_info.qualified_near_amount += near_amount;
        } else {
            pot_info.near_amount += near_amount;
        }
        self.pot_infos.insert(&account, &pot_info);
    }
}


#[near_bindgen]
impl FungibleTokenReceiver for Pool {
    /// If given `msg: "take-my-money", immediately returns U128::From(0)
    /// Otherwise, makes a cross-contract call to own `value_please` function, passing `msg`
    /// value_please will attempt to parse `msg` as an integer and return a U128 version of it
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Verifying that we were called by fungible token contract that we expect.
        // require!(
        //     env::predecessor_account_id() == self.fungible_token_account_id,
        //     "Only supports the one fungible token contract"
        // );
        log!("in {} tokens from @{} ft_on_transfer, msg = {}", amount.0, sender_id.as_ref(), msg);
        match msg.as_str() {
            "take-my-money" => PromiseOrValue::Value(U128::from(0)),
            _ => {
                PromiseOrValue::Value(U128::from(100))
                // let prepaid_gas = env::prepaid_gas();
                // let account_id = env::current_account_id();
                // Self::ext(account_id)
                //     .with_static_gas(prepaid_gas - GAS_FOR_FT_ON_TRANSFER)
                //     .value_please(msg)
                //     .into()
            }
        }
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

        // let res = pool.get_status(alice);
        // println!("{:?}", res.near_apr_history[0].apr);
    }
}