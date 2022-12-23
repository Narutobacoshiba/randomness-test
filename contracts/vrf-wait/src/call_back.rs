use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub enum CallBackMsg {
    ReturnRandomness{randomness:String},
}

