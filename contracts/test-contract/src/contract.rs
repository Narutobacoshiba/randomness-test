#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
     Response, StdResult, Addr, WasmMsg};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, CallBackMsg};
use crate::state::{RANDOMNESS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:test-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // With `Response` type, it is possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Handling contract migration
/// To make a contract migratable, you need
/// - this entry_point implemented
/// - only contract admin can migrate, so admin has to be set at contract initiation time
/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
        ExecuteMsg::RequestRandomness{} => handle_request_randomness(),
        ExecuteMsg::ReturnRandomness{randomness} => handle_return_randomness(_deps,_info,randomness),
    }
}

fn handle_request_randomness() -> Result<Response, ContractError> {
    let random_source = "aura1gjfcv5gvqd0j5x5rc6t9qdsjp4mj2s4msk0huk2yfqcfvndz49es7pzzpd";

    let request_randomness_msg  = CallBackMsg::RequestRandomness{
        key_hash: "aabbccddeeff".to_string(),
        time_set: 1000,
    };

    WasmMsg::Execute {
        contract_addr: random_source.to_string(),
        msg: to_binary(&request_randomness_msg)?,
        funds: vec![],
    };
    return Ok(Response::new().add_attribute("action","request randomness".to_string()));
}

fn handle_return_randomness(_deps: DepsMut, _info: MessageInfo, randomness: String) -> Result<Response, ContractError> {
    RANDOMNESS.save(_deps.storage, &randomness);
    return Ok(Response::new().add_attribute("action","return randomness"));
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // Find matched incoming message variant and query them your custom logic
        // and then construct your query response with the type usually defined
        // `msg.rs` alongside with the query message itself.
        //
        // use `cosmwasm_std::to_binary` to serialize query response to json binary.
        QueryMsg::GetRandomness{} => to_binary(&query_randomness(_deps)?),
    }
}

fn query_randomness(_deps: Deps) -> StdResult<String>{
    let randomness = RANDOMNESS.load(_deps.storage)?;
    return Ok(randomness);
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // With `Response` type, it is still possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages

    todo!()
}
