use cosmwasm_schema::{cw_serde, QueryResponses};
use serde::{Deserialize, Serialize};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    Register{moniker:String},
    Push{random_obj:String,signature:String},
    GetRandomValue{}
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // This example query variant indicates that any client can query the contract
    // using `YourQuery` and it will return `YourQueryResponse`
    // This `returns` information will be included in contract's schema
    // which is used for client code generation.
    //
    // #[returns(YourQueryResponse)]
    // YourQuery {},
}

// We define a custom struct for each query response
// #[cw_serde]
// pub struct YourQueryResponse {}

/*
#[derive(Debug, Deserialize, Serialize)]
pub struct RandomObjectLicense {
    type: String,
    text: String,
    infoUrl: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RandomObject {
    method: String,
    hashedApiKey: String,
    n: u32,
    min: u32,
    max: u32,
    replacement: bool,
    base: u32,
    pregeneratedRandomization: String,
    data: [u8:32],
    license: RandomObjectLicense,
    licenseData: String,
    userData: String,
    ticketData: String,
    completionTime: String,
    serialNumber: u32,
}*/