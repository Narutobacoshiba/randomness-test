// use `cw_storage_plus` to create ORM-like interface to storage
// see: https://crates.io/crates/cw-storage-plus

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Deque,Map};

#[cw_serde]
pub struct Generator {
    pub addr: Addr,
    pub public_key: String,
}

#[cw_serde]
pub struct RandomnessRequest {
    pub user: Addr,
    pub key_hash: String,
    pub time: u128,
}

pub const GENERATORS: Map<Addr,Generator> = Map::new("generators");
pub const RANDOMNESS_REQUEST_STATE: Deque<RandomnessRequest> = Deque::new("randomness_request_state");
