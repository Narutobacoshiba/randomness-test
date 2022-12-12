#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use cw2::set_contract_version;

use crate::state::{Generator, GENERATORS, RandomState, RANDOM_STATE_HISTORY};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, DrandCallBack};
use crate::verify::{VerifyDrandSignature, derive_randomness_from_signature};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:bls-contract";
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
    
    let init_random_state = RandomState {
        round: 0,
        randomness: String::from(""),
        origin_data: String::from(""),
        signature:  String::from(""),
        generator: None,
        block_height: _env.block.height + 1,
    };

    RANDOM_STATE_HISTORY.push_back(deps.storage, &init_random_state)?;

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
        ExecuteMsg::Register{moniker} => register(_deps,_info,moniker),
        ExecuteMsg::Push{round,signature,previous_signature} => push(_deps,_info,_env,round,previous_signature,signature),
        ExecuteMsg::GetRandomValue{} => get_random_value(_deps,_info),
        ExecuteMsg::Recive{callback} => recive(_deps,_info,callback),
    }
}

fn recive(_deps: DepsMut, _info: MessageInfo, callback: DrandCallBack){
    
}

fn register(_deps: DepsMut, _info: MessageInfo, moniker: String) -> Result<Response, ContractError>{
    if GENERATORS.has(_deps.storage, _info.sender.clone()) {
        return Err(ContractError::CustomError{val:"Address has been registered !".to_string()}); 
    }

    GENERATORS.save(_deps.storage, _info.sender.clone(), &Generator{
        addr: _info.sender.clone(),
        moniker: moniker,
        reward: Vec::new(),
    });

    return Ok(Response::new()
        .add_attribute("action", "register".to_string())
    );
}

fn push(_deps: DepsMut, _info: MessageInfo, _env: Env, round: u64, previous_signature: String, signature: String) -> Result<Response, ContractError> {
    let last_random_state_op = RANDOM_STATE_HISTORY.back(_deps.storage)?;

    if !last_random_state_op.is_some(){
        return Err(ContractError::CustomError{val:"State history error!".to_string()})
    }

    let last_random_state = last_random_state_op.unwrap();

    let current_block_height = _env.block.height;
    if current_block_height <= last_random_state.block_height + 30 {
        return Err(ContractError::CustomError{val:"Block height not reach!".to_string()}); 
    }

    if GENERATORS.has(_deps.storage, _info.sender.clone()) {
        let signature_bytes = hex::decode(signature.clone()).unwrap();
        let previous_signature_bytes = hex::decode(previous_signature.clone()).unwrap();

        let verify = VerifyDrandSignature(round, signature_bytes, previous_signature_bytes);
        
        if !verify {
            return Err(ContractError::CustomError{val:"Verification failed!".to_string()});
        }else{
            let randomness = derive_randomness_from_signature(signature_bytes);

            RANDOM_STATE_HISTORY.push_back(
                        _deps.storage, &RandomState{
                                        round: last_random_state.round + 1,
                                        randomness: hex::encode(randomness),
                                        origin_data: "".to_string(),
                                        signature: previous_signature, 
                                        generator: Some(_info.sender),
                                        block_height: current_block_height + 1,
                                    })?;
        }
    }else{
        return Err(ContractError::CustomError{val:"Address has't been registerd!".to_string()});
    }

    return Ok(Response::new()
        .add_attribute("action", "push".to_string())
    );
}

fn get_random_value(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    let last_random_state_op = RANDOM_STATE_HISTORY.back(_deps.storage)?;
    if !last_random_state_op.is_some() {
        return Err(ContractError::CustomError{val:"State history error!".to_string()}); 
    }
    
    let last_random_state = last_random_state_op.unwrap();

    if last_random_state.generator.is_some() && 
        GENERATORS.has(_deps.storage, last_random_state.generator.clone().unwrap()) {
            let mut generator = GENERATORS.load(_deps.storage, last_random_state.generator.expect("error"))?;
                    
            for fund in _info.funds {
                generator.reward.push(fund);
            }

            return Ok(Response::new()
                    .add_attribute("round", last_random_state.round.to_string())
                    .add_attribute("randomness", last_random_state.randomness)
                    .add_attribute("origin_data", last_random_state.origin_data)
                    .add_attribute("signature", last_random_state.signature)
                );
    }

    return Ok(Response::new()
                .add_attribute("round", "".to_string())
                .add_attribute("randomness", "".to_string())
                .add_attribute("origin_data", "".to_string())
                .add_attribute("signature", "".to_string())
            );
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
    }
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // With `Response` type, it is still possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages

    todo!()
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_info,
    };

    #[test]
    fn register_success() {  

        assert_eq!("","");
    }

}
