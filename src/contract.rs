use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise, PromiseOrValue, require, log, 
    Gas};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use serde_json::json;

use crate::msg::{AprInfo, UserInfo, AmountInfo, FarmInfo, PotInfo, Status, DepositParam};
use crate::util::{Check};

const BASE_GAS: u64 = 5_000_000_000_000;
const PROMISE_CALL: u64 = 5_000_000_000_000;
const GAS_FOR_FT_ON_TRANSFER: Gas = Gas(BASE_GAS + PROMISE_CALL);
const FARM_AMOUNT: u128 = 114_000_000;
const FARM_PERIOD: u64 = 5_184_000_000; //60 days in msecond
const REWARD_TIME: u64 = 600_000; //10minutes //24 hours for reward in msecond

pub const COIN_COUNT: usize = 7;
const COINS: [&str; 7] = ["USDC", "USDT", "DAI", "USN", "wBTC", "ETH", "wNEAR"];
const DECIMALS: [u32; 7] = [6, 6, 18, 18, 8, 18, 24];

pub fn getcoin_id(coin: String) -> usize{
    let res = match coin.as_str(){
        "USDC" => 0,
        "USDT" => 1,
        "DAI" => 2,
        "USN" => 3,
        "wBTC" => 4,
        "ETH" => 5,
        "wNEAR" => 6,
        _ => env::panic_str("Not correct coin type")
    };
    res
}


#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Pool {
    owner: AccountId,
    treasury: AccountId,
    apr: Vec<u32>,
    user_infos: UnorderedMap<AccountId, Vec<UserInfo>>, // for all coin
    total_rewards: Vec<u128>,
    
    amount_history: Vec<AmountInfo>,
    //--------farm-----------------
    farm_starttime: u64,
    farm_price: u128,
    farm_infos: UnorderedMap<AccountId, FarmInfo>,
    total_farmed: u128,
    
    //--------qualify----------------------
    pot_infos: UnorderedMap<AccountId, Vec<PotInfo>>,

    //-------_token address--------------------
    token_address: Vec<AccountId>
}

#[near_bindgen]
impl Pool {
    #[init]
    pub fn new(owner: Option<AccountId>, treasury: AccountId ) -> Self {
        let wnear = AccountId::new_unchecked("ft.alenzertest.testnet".to_string());
        Self {
            owner: match owner{
                Some(_owner) => _owner,
                None => env::current_account_id()
            },
            treasury: treasury,
            apr: vec![1487, 1487, 1487, 1487, 987, 987, 987],
            user_infos: UnorderedMap::new(b"n"),
            total_rewards: vec![0; COIN_COUNT],
            amount_history: Vec::new(),
            farm_starttime: env::block_timestamp_ms(),
            farm_price: 25,
            farm_infos: UnorderedMap::new(b"f"),
            total_farmed: 0,
            pot_infos: UnorderedMap::new(b"p"),
            token_address: vec![wnear; COIN_COUNT]
        }
    }
    pub fn delete_all(&mut self){
        self.amount_history.clear();
        self.user_infos.clear();
        self.pot_infos.clear();
    }
    pub fn set_config(&mut self, owner: Option<AccountId>, treasury: Option<AccountId>){
        self.check_onlyowner();
        if let Some(account) = owner {
            self.owner = account;
        }
        if let Some(account) = treasury {
            self.treasury = account;
        }
    }

    pub fn set_tokenaddress(&mut self, token: [AccountId; COIN_COUNT]){
        self.check_onlyowner();
        for i in 0..COIN_COUNT{
            self.token_address[i] = token[i].clone();
        }
    }

    pub fn set_apr(&mut self, coin: String, apr: u32 ){
        self.check_onlyowner();

        self.apr[getcoin_id(coin)] = apr;
    }
    pub fn withdraw_reserve(&mut self, coin: String, amount: U128){
        let _amount: u128 = amount.into();
        let account = env::signer_account_id();
        let mut user_info = self.user_infos.get(&account).unwrap();
        let coin_id = getcoin_id(coin.clone());

        if user_info[coin_id].amount + user_info[coin_id].reward_amount < _amount {
            return env::panic_str("Not enough balance")
        }

        user_info[coin_id].withdraw_reserve = _amount;
        self.user_infos.insert(&account, &user_info);
    }
    pub fn withdraw(&mut self, account: AccountId, coin: String, amount: U128, price: [U128; COIN_COUNT]){
        self.check_onlytreasury();

        let _amount: u128 = amount.into();
        let mut user_info = self.user_infos.get(&account).unwrap();
        let coin_id = getcoin_id(coin.clone());
        
        if user_info[coin_id].withdraw_reserve < _amount {
            return env::panic_str("Not enough reserved")
        }

        if user_info[coin_id].amount + user_info[coin_id].reward_amount < _amount {
            return env::panic_str("Not enough balance")
        }

        let remain;
        if user_info[coin_id].amount >= _amount {
            remain = _amount;
            user_info[coin_id].amount -= _amount;
        } else {
            remain = user_info[coin_id].amount;
            user_info[coin_id].amount = 0;
            user_info[coin_id].reward_amount -= _amount - remain;

            self.total_rewards[coin_id] -= _amount - remain;
        }
        user_info[coin_id].withdraw_reserve = 0;

        self.append_amount_history(coin.clone(), remain, false);
        self.withdraw_potinfo(account.clone(), coin.clone(), remain);
        self.farm_withdraw(account.clone(), coin.clone(), remain, price);

        self.user_infos.insert(&account.clone(), &user_info);
    }

    fn get_user_info(&self, owner_id: &AccountId) -> Vec<UserInfo> {
        self.user_infos.get(owner_id).unwrap()
    }

    pub fn rewards(&mut self){
        self.check_onlytreasury();

        let available_time = env::block_timestamp_ms() - REWARD_TIME;
        let keys = self.user_infos.to_vec();
        let mut bmodified = false;
        for i in 0..keys.len()
        {
            let key = keys[i].0.clone();
            let mut user_info = self.get_user_info(&key);
            for coin in COINS {   
                let coin_id = getcoin_id(coin.to_string());

                if user_info[coin_id].deposit_time < available_time { 
                    let apr = self.apr[coin_id];
                    let rewards = (user_info[coin_id].amount + user_info[coin_id].reward_amount) * (apr as u128) / 10_000 / 365;
                    user_info[coin_id].reward_amount += rewards;
                    self.total_rewards[coin_id] += rewards;
                    if rewards > 0{
                        bmodified = true;
                    }
                }
            }
            self.user_infos.insert(&key, &user_info);
        }

        if bmodified && self.amount_history.len() > 0 {
            let last_index = self.amount_history.len() - 1;
            let mut info = self.amount_history[last_index].clone();
            for i in 0..COIN_COUNT{
                info.reward[i] = self.total_rewards[i];
            }
            self.amount_history.push(info);

            if last_index > 10 {
                let mut retain = vec![true; self.amount_history.len()];
                retain[0] = false;

                let mut iter = retain.iter();
                self.amount_history.retain(|_| *iter.next().unwrap());
            }
        }
    }

    pub fn farm(&mut self, price: [U128; COIN_COUNT]){
        self.check_onlytreasury();
    
        let current_time = env::block_timestamp_ms();
        let farm_starttime = self.farm_starttime;
        let farm_endtime = farm_starttime + FARM_PERIOD;

    //-----------------condition check------------------------------
        if farm_starttime == 0 || current_time < farm_starttime  {
            return env::panic_str("NotStartedFarming")
        }
log!("farm starttime {} current_time{} endtime {}", farm_starttime, current_time, farm_endtime);
        let mut total_farm = self.total_farmed;
        if farm_endtime < current_time || total_farm > FARM_AMOUNT {
            return;
        }
    //--------------------calc farming amount---------------------
        let mut total_as_usd = 0;
    
        let keys = self.user_infos.to_vec();
        for i in 0..keys.len()
        {
            let key = keys[i].0.clone();
            let user_info = self.get_user_info(&key); //(x/10^y) * (price / 10^2) /10^3 * 24 = x/(10^y)*price*24/10^5
            let mut farm = 0;
            
            for i in 0..COIN_COUNT{
                let _price: u128 = price[i].into();
                farm += user_info[i].amount * _price * 24 / (10u128).pow(DECIMALS[i]) / (10u128).pow(5);
log!("acount{} farm amount {} - useramount {} price {} Decimal {}", key, farm, user_info[i].amount, _price, DECIMALS[i]);
                total_as_usd += user_info[i].amount * _price / (10u128).pow(DECIMALS[i]) / 100;
            }

            self.update_farm_info(key, farm);
            total_farm += farm;
        }

        self.total_farmed = total_farm;
    //-------------------recalc token price ------------------------------------
        //x * (price / 10^2) / 20,000,000
        let multiple = total_as_usd / (20_000_000u128);
        //0.25*(1.2)^multiple = 25/10^2 * (12) ^ multiple) /(10^multiple) *10^2
        let price = 25 * (12u128).pow(multiple as u32) / 
                            (10u128).pow(multiple as u32);
        self.farm_price = price as u128;
    }
 
    pub fn pot_process(&mut self){
        self.check_onlytreasury();

        let keys = self.pot_infos.to_vec();

        for i in 0..keys.len()
        {
            let mut pot_info = keys[i].1.clone();

            let mut bnone = true;
            for j in 0..COIN_COUNT { 
                pot_info[j].qualified_amount = pot_info[j].amount;
                pot_info[j].amount = 0;
                
                if pot_info[j].qualified_amount != 0 {
                    bnone = false;
                }
            }
            if bnone == true{
                self.pot_infos.remove(&keys[i].0);
            }else{
                self.pot_infos.insert(&keys[i].0, &pot_info);
            }
        }
    }
    pub fn get_pot_info(self) -> Vec<Vec<PotInfo>> {
        let keys = self.pot_infos.to_vec();
        let mut infos: Vec<Vec<PotInfo>> = vec![];
        for i in 0..keys.len(){
            infos.push(keys[i].1.clone())
        }
        infos
    }
    pub fn get_farm_info(self) -> Vec<FarmInfo> {
        let keys = self.farm_infos.to_vec();
        let mut infos: Vec<FarmInfo> = vec![];
        for i in 0..keys.len(){
            infos.push(keys[i].1.clone())
        }
        infos
    }
    pub fn get_amount_history(self) -> Vec<AmountInfo> {
        self.amount_history
    }
    pub fn get_status(self, account: AccountId) -> Status{
        let userinfo = match self.user_infos.get(&account){
            Some(info) => info,
            None => vec![UserInfo{
                account: account.clone(),
                amount: 0,
                reward_amount: 0,
                deposit_time: 0,
                withdraw_reserve: 0
            }; COIN_COUNT]
        };

        let farminfo = match self.farm_infos.get(&account){
            Some(info) => info,
            None => FarmInfo{
                account: account.clone(),
                amount: 0,
            }
        };

        let potinfo = match self.pot_infos.get(&account){
            Some(info) => info,
            None => vec![PotInfo{
                account: account.clone(),
                amount: 0,
                qualified_amount: 0
            }; COIN_COUNT]
        };

        Status {
            amount_history: self.amount_history,
            user_info: userinfo,
            farm_price: self.farm_price,
            farm_info: farminfo,
            farm_starttime: self.farm_starttime,
            total_rewards: self.total_rewards,
            pot_info: potinfo
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

    fn deposit(&mut self, coin: String, amount: u128, qualified: bool) {
        let account = env::signer_account_id();
 
        let coin_id = getcoin_id(coin.clone());
        if let Some(mut user_info) = self.user_infos.get(&account){
            user_info[coin_id].amount += amount;
            user_info[coin_id].deposit_time = env::block_timestamp_ms();

            self.user_infos.insert(&account, &user_info);
        } else {
            let mut user_info = vec![UserInfo{
                account: account.clone(),
                amount: 0,
                reward_amount: 0,
                deposit_time: 0,
                withdraw_reserve: 0,
            }; 7];
            user_info[coin_id].amount = amount;
            user_info[coin_id].deposit_time = env::block_timestamp_ms();

            self.user_infos.insert(&account, &user_info);
        }

        self.append_amount_history(coin.clone(), amount, true);
        self.deposit_potinfo(account.clone(), coin.clone(), amount, qualified);

        let arguments = json!({ "receiver_id": self.treasury.to_string(), "amount": amount.to_string() }) // method arguments
                .to_string()
                .into_bytes();
        Promise::new(self.token_address[coin_id].clone())
            .function_call("ft_transfer".to_string(), arguments, 1, Gas(5_000_000_000_000));
    }

    fn append_amount_history(&mut self, coin: String, amount: u128,  bAdd: bool){
        let coin_id = getcoin_id(coin.clone());
        if self.amount_history.len() == 0 {
            let mut amounts = vec![0;7];
            let rewards = vec![0;7];
            amounts[coin_id] = amount;

            self.amount_history.push(AmountInfo{
                amount: amounts,
                reward: rewards,
                time: env::block_timestamp_ms()
            });
        } 
        else {
            let last_index = self.amount_history.len() - 1;
            let mut info = self.amount_history[last_index].clone();

            if bAdd {
                info.amount[coin_id] += amount;
            } else {
                info.amount[coin_id] -= amount;
            }
            info.time = env::block_timestamp_ms();
            info.reward[coin_id] = self.total_rewards[coin_id];

            self.amount_history.push(info);

            if last_index > 10 {
                let mut retain = vec![true; self.amount_history.len()];
                retain[0] = false;

                let mut iter = retain.iter();
                self.amount_history.retain(|_| *iter.next().unwrap());
            }
        }
    }

    fn deposit_potinfo(&mut self, account: AccountId, coin: String,  amount: u128, qualified: bool){
        let mut pot_info = if let Some(info) = self.pot_infos.get(&account) {
                info
            } 
            else {
                vec![PotInfo{
                        account: account.clone(),
                        amount: 0,
                        qualified_amount: 0
                    }; COIN_COUNT]
            };

        if qualified {
            pot_info[getcoin_id(coin)].qualified_amount += amount;
        } else {
            pot_info[getcoin_id(coin)].amount += amount;
        }
        self.pot_infos.insert(&account, &pot_info);
    }
    
    fn withdraw_potinfo(&mut self, account: AccountId, coin: String, amount: u128) {
        let res = self.pot_infos.get(&account);
        if res == None{
            return;
        }
        
        let coin_id = getcoin_id(coin);
        let mut pot_info = self.pot_infos.get(&account).unwrap();
        if pot_info[coin_id].qualified_amount >= amount {
            pot_info[coin_id].qualified_amount -= amount;
        }else {
            pot_info[coin_id].qualified_amount = 0;
            let _amount = amount - pot_info[coin_id].qualified_amount;
    
            if pot_info[coin_id].amount >= _amount {
                pot_info[coin_id].amount -= _amount;
            }else{
                pot_info[coin_id].amount = 0;
            }
        }
        self.pot_infos.insert(&account, &pot_info);
    }
    fn farm_withdraw(&mut self, account: AccountId, coin: String, amount: u128, price: [U128; COIN_COUNT])
    {
        let current_time = env::block_timestamp_ms();
        let farm_starttime = self.farm_starttime;
        let farm_endtime = farm_starttime + FARM_PERIOD;
    
    //-----------------condition check------------------------------
        if farm_starttime == 0 || current_time < farm_starttime  {
            return;
        }
    
        if farm_endtime < current_time {
            return;
        }
    //-----------------No farm yet------------------------
        let res = self.farm_infos.get(&account);
        if res == None {
            return;
        }
    
    //--------------------calc farming amount---------------------
        let coin_id = getcoin_id(coin);
        let mut farm_info = self.farm_infos.get(&account).unwrap();
        let mut total_as_usd = 0;
    
        let res = self.user_infos.get(&account);
        match res{
            Some(user_info) => {
                for i in 0..COIN_COUNT{
                    let _price: u128 = price[i].into();
                    total_as_usd += user_info[i].amount * _price;
                }
            },
            None =>{ }
        }
    
        if total_as_usd > 0 {
            let _price: u128 = price[coin_id].into();
            let mut withdraw_as_usd = amount * _price;
    
            if withdraw_as_usd > total_as_usd {
                withdraw_as_usd = total_as_usd;
            }
            
            let withdraw_amount = withdraw_as_usd * farm_info.amount/total_as_usd;
    
            farm_info.amount -= withdraw_amount;
            self.total_farmed -= withdraw_amount;
            
            self.farm_infos.insert(&account, &farm_info);
        }
    }

    fn update_farm_info( &mut self, account: AccountId, amount: u128 ){
        let res = self.farm_infos.get(&account);
        let user_info = match res{
            Some(mut info) => {
                info.amount += amount;
                info
            },
            None => FarmInfo{
                account: account.clone(),
                amount: amount,
            }
        };
        self.farm_infos.insert(&account, &user_info);
    }
    //USDC-0 USDT-1 DAI-2 USN-3 wBTC-4 ETH-5 wNEAR-6
}


#[near_bindgen]
impl FungibleTokenReceiver for Pool {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {

        log!("in {} tokens from @{} ft_on_transfer, msg = {}", amount.0, sender_id.as_ref(), msg);

        let param: DepositParam = serde_json::from_str(msg.as_str()).unwrap();
        self.deposit(param.coin, amount.into(), param.qualified);

        PromiseOrValue::Value(amount)
        // match msg.as_str() {
            // "take-my-money" => PromiseOrValue::Value(U128::from(0)),
            // _ => {
                // PromiseOrValue::Value(U128::from(100))
                // let prepaid_gas = env::prepaid_gas();
                // let account_id = env::current_account_id();
                // Self::ext(account_id)
                //     .with_static_gas(prepaid_gas - GAS_FOR_FT_ON_TRANSFER)
                //     .value_please(msg)
                //     .into()
            // }
        // }
    }
}

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
        
        let near_apr = 121;
        // Set up the testing context and unit test environment
        let mut context = get_context(alice.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            // .attached_deposit(1)
            .predecessor_account_id(alice.clone())
            .block_timestamp(12345678)
            .build());

        let mut pool = Pool::new(owner, treasury.clone() );

        testing_env!(context
            .storage_usage(env::storage_usage())
            // .attached_deposit(1)
            .predecessor_account_id(alice.clone())
            .block_timestamp(12345678)
            .build());
        
        let val = DepositParam{
            coin: "wBTC".to_string(),
            qualified: true
        };
        let arguments = json!(val) // method arguments
            .to_string();

        pool.ft_on_transfer(alice.clone(), U128::from(100_000_000), arguments);

        testing_env!(context
            .storage_usage(env::storage_usage())
            // .attached_deposit(1)
            .predecessor_account_id(treasury.clone())
            .block_timestamp(13355678)
            .build());
        pool.rewards();

        let price: [U128;7] = [U128::from(500000); 7];
        pool.withdraw(alice.clone(), "wBTC".to_string(), U128::from(50_000_000), price);

        testing_env!(context
            .storage_usage(env::storage_usage())
            // .attached_deposit(1)
            .predecessor_account_id(treasury.clone())
            .block_timestamp(14345678)
            .build());
        pool.farm(price);


        let coin_id = getcoin_id("wBTC".to_string());
        let res = pool.get_status(alice.clone());
        println!("{:?}", res.user_info);
        println!("{:?}", res.farm_info);
        
    }
}