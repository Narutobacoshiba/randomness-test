#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{CosmosMsg, Binary, Deps, DepsMut, Env, MessageInfo, 
    Reply, Response, StdResult, WasmMsg, to_binary, Empty};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, DrandResponse};
use crate::state::{Generator,RandomnessRequest,GENERATORS,RANDOMNESS_REQUEST_STATE};
use crate::hasher::sha256_hash;
use crate::call_back::{CallBackMsg};
use crate::drand_verify::verify_drand_randomness;

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
        ExecuteMsg::PushRandomness{randomness,signature,drand_response} => handle_push_randomness(_deps,_info,randomness,signature,drand_response),
        ExecuteMsg::RequestRandomness{key_hash, time_set} => handle_request_randomness(_deps,_info,key_hash,time_set),
        ExecuteMsg::Register{public_key} => handle_register(_deps,_info,public_key),
        ExecuteMsg::DeleteGenerator{} => handle_delete_generator(_deps,_info),
    }
}

fn handle_delete_generator(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    if !GENERATORS.has(_deps.storage, _info.sender.clone()) {
        return Err(ContractError::CustomError{val:"Unregisterd address!".to_string()}); 
    }

    GENERATORS.remove(_deps.storage, _info.sender);

    return Ok(Response::new().add_attribute("action","delete generator!".to_string()));
}


fn handle_register(_deps: DepsMut, _info: MessageInfo, public_key: String) -> Result<Response, ContractError>{

    if GENERATORS.has(_deps.storage, _info.sender.clone()) {
        return Err(ContractError::CustomError{val:"Address has been registered !".to_string()}); 
    }

    GENERATORS.save(_deps.storage, _info.sender.clone(), &Generator{
        addr: _info.sender.clone(),
        public_key: public_key,
    })?;

    return Ok(Response::new()
        .add_attribute("action", "register".to_string())
    );
}

fn handle_request_randomness(_deps: DepsMut, _info: MessageInfo, key_hash: String, 
    time_set: u128) -> Result<Response, ContractError> {
    let request = RandomnessRequest {
        user: _info.sender,
        key_hash: key_hash,
        time: time_set,
    };

    RANDOMNESS_REQUEST_STATE.push_back(_deps.storage,&request)?;

    return Ok(Response::new().add_attribute("action","request randomness"));
}


fn handle_push_randomness(_deps: DepsMut, _info: MessageInfo, randomness: String, 
    signature: String, drand_response: DrandResponse) -> Result<Response, ContractError>{
    
    if !GENERATORS.has(_deps.storage, _info.sender.clone()){
        return Err(ContractError::CustomError{val:"Unregistered adrress!".to_string()});
    }

    let generator = GENERATORS.load(_deps.storage, _info.sender.clone())?;

    if verify_drand_randomness(drand_response.round, 
                            hex::decode(drand_response.signature).unwrap(),
                            hex::decode(drand_response.previous_signature).unwrap()) {
        
        let signature_bytes = hex::decode(signature.clone()).unwrap();
        let randomness_bytes = hex::decode(randomness.clone()).unwrap();
        let key_bytes = hex::decode(generator.public_key.clone()).unwrap();

        let result = _deps.api.secp256k1_verify(&randomness_bytes, &signature_bytes, &key_bytes);

        match result {
            Ok(true) => {             
                let request_op = RANDOMNESS_REQUEST_STATE.pop_back(_deps.storage)?;

                if request_op.is_some() {
                    let request = request_op.unwrap();

                    let seed = signature + &request.key_hash + &hex::encode(request.user.as_bytes());

                    let return_randomness = sha256_hash(&hex::decode(seed).unwrap());

                    let return_randomness_msg  = CallBackMsg::ReturnRandomness{
                        randomness: hex::encode(return_randomness),
                    };

                    WasmMsg::Execute {
                        contract_addr: request.user.as_ref().to_string(),
                        msg: to_binary(&return_randomness_msg)?,
                        funds: vec![],
                    };
                }
                    
            },
            Ok(false) => return Err(ContractError::CustomError{val:"Invalid generator signature!".to_string()}),
            Err(_err) => return Err(ContractError::CustomError{val:"Invalid generator signature!".to_string()}),
        }
    }else {
        return Err(ContractError::CustomError{val:"Invalid drand randomness!".to_string()});
    }

    return Ok(Response::new().add_attribute("action","push randomness".to_string()));
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
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_slice, Binary, OwnedDeps, RecoverPubkeyError, StdError, VerificationError, Env,
    };

    const CREATOR: &str = "creator";
    const GENERATOR: &str = "generator";
    const USER: &str = "user";

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        deps
    }

    #[test]
    fn instantiate_works() {
        setup();
    }

    #[test]
    fn generator_register_success() {
        let mut deps = setup();

        let public_key: String = "03224d723fb48f1c247e105bca7853fb5c711e59fc824f5e581614d80bfbde1c99".to_string();

        let register_msg = ExecuteMsg::Register{
            public_key
        };

        let raw = execute(deps.as_mut(), mock_env(), mock_info(GENERATOR, &[]), register_msg).unwrap();

        assert_eq!(raw, Response::new().add_attribute("action", "register".to_string()))
    }

    #[test]
    fn generator_push_randomness_success() {
        let mut deps = setup();

        let public_key: String = "03224d723fb48f1c247e105bca7853fb5c711e59fc824f5e581614d80bfbde1c99".to_string();
        
        let register_msg = ExecuteMsg::Register{
            public_key
        };

        let raw = execute(deps.as_mut(), mock_env(), mock_info(GENERATOR, &[]), register_msg).unwrap();

        assert_eq!(raw, Response::new().add_attribute("action", "register".to_string()));

        let randomness = "3e145797dba4b22ace82d72ddcd2c4fede803247337ba8b11fca7e2402e235a9".to_string();
        let signature = "d92ae4131cbd4e720789ad13fef129d6043421301ba40342513a550b10b7219b6b6d916a711a660bdde1b88a55933ac324687d3b7240404146d4c13f814a99c1".to_string();
        let drand_response = DrandResponse {
            round: 2515680,
            signature: "b46015d219fb64ad572c32ae5104e3d6498ea9a7e8045d5f7b1355f20b3c05894f7c9b7af2a5d4499dc6c8f3a50f85d209b903e2c0660075a994e463c99c49582eab9d991a35fba2784979f51a5943476a3cf2096762be81f2d90a9bc1097259".to_string(),
            previous_signature: "b70285d05e8021c296f4e1fc4f7624210a5540a59628df703bcceebb7f5df9c1ce205956c23a5ec7bc31bc431535b53007d2abb32a2a84f4d1318aba0cfa9088d6a11aac135c56e77b14ed0ee2247084066a1590bec22f1e0802460ce18d21ba".to_string(),
        };

        let push_randomness_msg = ExecuteMsg::PushRandomness {
            randomness,
            signature,
            drand_response
        };

        let raw_push = execute(deps.as_mut(), mock_env(), mock_info(GENERATOR, &[]), push_randomness_msg).unwrap();

        assert_eq!(raw_push, Response::new().add_attribute("action", "push randomness".to_string()));

    }

    #[test]
    fn generator_push_randomness_fail_with_invalid_drand_randomness() {
        let mut deps = setup();

        let public_key: String = "03224d723fb48f1c247e105bca7853fb5c711e59fc824f5e581614d80bfbde1c99".to_string();
        
        let register_msg = ExecuteMsg::Register{
            public_key
        };

        let raw = execute(deps.as_mut(), mock_env(), mock_info(GENERATOR, &[]), register_msg).unwrap();

        assert_eq!(raw, Response::new().add_attribute("action", "register".to_string()));

        let randomness = "3e145797dba4b22ace82d72ddcd2c4fede803247337ba8b11fca7e2402e235a9".to_string();
        let signature = "d92ae4131cbd4e720789ad13fef129d6043421301ba40342513a550b10b7219b6b6d916a711a660bdde1b88a55933ac324687d3b7240404146d4c13f814a99c1".to_string();
        let drand_response = DrandResponse {
            round: 2515680,
            signature: "a46015d219fb64ad572c32ae5104e3d6498ea9a7e8045d5f7b1355f20b3c05894f7c9b7af2a5d4499dc6c8f3a50f85d209b903e2c0660075a994e463c99c49582eab9d991a35fba2784979f51a5943476a3cf2096762be81f2d90a9bc1097259".to_string(),
            previous_signature: "b70285d05e8021c296f4e1fc4f7624210a5540a59628df703bcceebb7f5df9c1ce205956c23a5ec7bc31bc431535b53007d2abb32a2a84f4d1318aba0cfa9088d6a11aac135c56e77b14ed0ee2247084066a1590bec22f1e0802460ce18d21ba".to_string(),
        };

        let push_randomness_msg = ExecuteMsg::PushRandomness {
            randomness,
            signature,
            drand_response
        };

        let raw_push = execute(deps.as_mut(), mock_env(), mock_info(GENERATOR, &[]), push_randomness_msg).unwrap_err();

        match raw_push {
            ContractError::CustomError{val:a} => {assert_eq!(a, "Invalid drand randomness!".to_string());},
            _ => panic!("")
        }
    }

    #[test]
    fn generator_push_randomness_fail_with_invalid_generator_signature() {
        let mut deps = setup();

        let public_key: String = "03224d723fb48f1c247e105bca7853fb5c711e59fc824f5e581614d80bfbde1c99".to_string();
        
        let register_msg = ExecuteMsg::Register{
            public_key
        };

        let raw = execute(deps.as_mut(), mock_env(), mock_info(GENERATOR, &[]), register_msg).unwrap();

        assert_eq!(raw, Response::new().add_attribute("action", "register".to_string()));

        let randomness = "3e145797dba4b22ace82d72ddcd2c4fede803247337ba8b11fca7e2402e235a9".to_string();
        let signature = "a92ae4131cbd4e720789ad13fef129d6043421301ba40342513a550b10b7219b6b6d916a711a660bdde1b88a55933ac324687d3b7240404146d4c13f814a99c1".to_string();
        let drand_response = DrandResponse {
            round: 2515680,
            signature: "b46015d219fb64ad572c32ae5104e3d6498ea9a7e8045d5f7b1355f20b3c05894f7c9b7af2a5d4499dc6c8f3a50f85d209b903e2c0660075a994e463c99c49582eab9d991a35fba2784979f51a5943476a3cf2096762be81f2d90a9bc1097259".to_string(),
            previous_signature: "b70285d05e8021c296f4e1fc4f7624210a5540a59628df703bcceebb7f5df9c1ce205956c23a5ec7bc31bc431535b53007d2abb32a2a84f4d1318aba0cfa9088d6a11aac135c56e77b14ed0ee2247084066a1590bec22f1e0802460ce18d21ba".to_string(),
        };

        let push_randomness_msg = ExecuteMsg::PushRandomness {
            randomness,
            signature,
            drand_response
        };

        let raw_push = execute(deps.as_mut(), mock_env(), mock_info(GENERATOR, &[]), push_randomness_msg).unwrap_err();

        match raw_push {
            ContractError::CustomError{val:a} => {assert_eq!(a, "Invalid generator signature!".to_string());},
            _ => panic!("")
        }
    }

    #[test]
    fn generator_push_randomness_fail_with_unregistered_address() {
        let mut deps = setup();

        let public_key: String = "03224d723fb48f1c247e105bca7853fb5c711e59fc824f5e581614d80bfbde1c99".to_string();
        
        let register_msg = ExecuteMsg::Register{
            public_key
        };

        let raw = execute(deps.as_mut(), mock_env(), mock_info(GENERATOR, &[]), register_msg).unwrap();

        assert_eq!(raw, Response::new().add_attribute("action", "register".to_string()));

        let randomness = "3e145797dba4b22ace82d72ddcd2c4fede803247337ba8b11fca7e2402e235a9".to_string();
        let signature = "d92ae4131cbd4e720789ad13fef129d6043421301ba40342513a550b10b7219b6b6d916a711a660bdde1b88a55933ac324687d3b7240404146d4c13f814a99c1".to_string();
        let drand_response = DrandResponse {
            round: 2515680,
            signature: "b46015d219fb64ad572c32ae5104e3d6498ea9a7e8045d5f7b1355f20b3c05894f7c9b7af2a5d4499dc6c8f3a50f85d209b903e2c0660075a994e463c99c49582eab9d991a35fba2784979f51a5943476a3cf2096762be81f2d90a9bc1097259".to_string(),
            previous_signature: "b70285d05e8021c296f4e1fc4f7624210a5540a59628df703bcceebb7f5df9c1ce205956c23a5ec7bc31bc431535b53007d2abb32a2a84f4d1318aba0cfa9088d6a11aac135c56e77b14ed0ee2247084066a1590bec22f1e0802460ce18d21ba".to_string(),
        };

        let push_randomness_msg = ExecuteMsg::PushRandomness {
            randomness,
            signature,
            drand_response
        };

        let raw_push = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), push_randomness_msg).unwrap_err();

        match raw_push {
            ContractError::CustomError{val:a} => {assert_eq!(a, "Unregistered adrress!".to_string());},
            _ => panic!("")
        }
    }

    #[test]
    fn user_request_randomness_success() {
        let mut deps = setup();

        let key_hash = "aabb".to_string();
        let time_set = 100;

        let request_randomness_msg = ExecuteMsg::RequestRandomness {
            key_hash: key_hash.clone(), 
            time_set: time_set,
        };

        let raw_push = execute(deps.as_mut(), mock_env(), mock_info(USER, &[]), request_randomness_msg).unwrap();
        assert_eq!(raw_push, Response::new().add_attribute("action", "request randomness".to_string()));

        let request = RANDOMNESS_REQUEST_STATE.back(&deps.storage).unwrap().unwrap();

        assert_eq!(request.key_hash, key_hash);
        assert_eq!(request.time, time_set);
        assert_eq!(request.user.as_ref(), USER);
    }
}