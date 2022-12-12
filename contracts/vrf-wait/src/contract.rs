#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, DrandResponse};
use crate::state::{Generator,RandomnessRequest,GENERATORS,RANDOMNESS_REQUEST_STATE};
use crate::hasher::sha256_hash;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vrf-wait";
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
        PushRandomness{randomness,signature,drand_response} => handle_push_randomness(_deps,_info,randomness,drand_response),
        RequestRandomness{key_hash, time_set},
    }
}

fn handle_push_randomness(_deps: DepsMut, _info: MessageInfo, randomness: String, signature: String, drand_response: DrandResponse) -> Result<Response, ContractError>{
    
    if(!GENERATORS.has(_deps.storage, _info.sender.clone())){
        return Err(ContractError::CustomError{val:"Unregistered adrress!".to_string()});
    }

    let generator = GENERATORS.load(_deps.storage, _info.sender.clone())?;

    if verify_drand_randomness(drand_response) {
        
        let signature_bytes = hex::decode(signature.clone()).unwrap();
        let randomness_bytes = hex::decode(randomness.clone()).unwrap();
        let key_bytes = hex::decode(generator.public_key.clone()).unwrap();

        let result = _deps.api.secp256k1_verify(&randomness_bytes, &signature_bytes, &key_bytes);

        match result {
            Ok(true) => {
                
                let mut count = 0u32;

                loop {
                    if count == 5 {
                        break;
                    }

                    let request = RANDOMNESS_REQUEST_STATE.pop_back(_deps.storage)?;

                    if !request.is_some() {
                        break;
                    }
                    
                    let seed = hex::decode(randomness + request.key_hash).unwrap();
                    seed.append(request.user.as_bytes());

                    let randomness = sha256_hash(&seed);
                    
                    let param  = CallBackParam {
                        randomness: hex::encode(randomness),
                    }

                    count += 1;
                }
                
            },
            Ok(false) => return Err(ContractError::CustomError{val:"some error".to_string()}),
            Err(_err) => return Err(ContractError::CustomError{val:"some error".to_string()}),
        }
    }else {
        return Err(ContractError::CustomError{val:"Invalid drand randomness!".to_string()});
    }

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
