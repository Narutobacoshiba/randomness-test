// use `cw_storage_plus` to create ORM-like interface to storage
// see: https://crates.io/crates/cw-storage-plus
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Deque,Map};

#[cw_serde]
pub struct Generator {
    pub addr: Addr,
    pub moniker: String,
    pub reward: Vec<Coin>,
}

#[cw_serde]
pub struct RandomState {
    pub round: u64,
    pub randomness: String,
    pub origin_data: String,
    pub signature: String, 
    pub generator: Option<Addr>,
    pub block_height: u64,
}


pub const GENERATORS: Map<Addr,Generator> = Map::new("generators");
pub const RANDOM_STATE_HISTORY: Deque<RandomState> = Deque::new("random_state_history");