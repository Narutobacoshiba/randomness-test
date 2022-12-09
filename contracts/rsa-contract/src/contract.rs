#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use cw2::set_contract_version;

use crate::state::{Generator, GENERATORS, RandomState, RANDOM_STATE_HISTORY};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::verify::VerifyRandomOrgSig;
use crate::hash::sha512_hash;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rsa-contract";
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
        block_height: _env.block.height,
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
        ExecuteMsg::Push{random_obj,signature} => push(_deps,_info,_env,random_obj,signature),
        ExecuteMsg::GetRandomValue{} => get_random_value(_deps,_info),
    }
}

fn register(_deps: DepsMut, _info: MessageInfo, moniker: String) -> Result<Response, ContractError>{
    
    
    if GENERATORS.has(_deps.storage, _info.sender.clone()) {
        return Err(ContractError::CustomError{val:"Address has been registered !".to_string()}); 
    }

    GENERATORS.save(_deps.storage, _info.sender.clone(), &Generator{
        addr: _info.sender.clone(),
        moniker: moniker,
        reward: Vec::new(),
    })?;

    return Ok(Response::new()
        .add_attribute("action", "register".to_string())
    );
}

fn push(_deps: DepsMut, _info: MessageInfo, _env: Env, random_obj_base64:String, signature_base64:String) -> Result<Response, ContractError> {
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
        let signature = base64::decode(signature_base64.clone()).unwrap();
        let random_obj = base64::decode(random_obj_base64.clone()).unwrap();

        let verify = VerifyRandomOrgSig(random_obj.clone(), signature.clone());
        
        if !verify {
            return Err(ContractError::CustomError{val:"Verification failed!".to_string()});
        }else{
            let out_randomness = sha512_hash(random_obj.clone());

            RANDOM_STATE_HISTORY.push_back(
                        _deps.storage, &RandomState{
                                        round: last_random_state.round + 1,
                                        randomness: hex::encode(&out_randomness),
                                        origin_data: hex::encode(&random_obj),
                                        signature: hex::encode(&signature), 
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
            let mut generator = GENERATORS.load(_deps.storage, last_random_state.generator.unwrap())?;
                    
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
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_slice, Binary, OwnedDeps, RecoverPubkeyError, StdError, VerificationError, Env,
    };

    const CREATOR: &str = "creator";

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info(CREATOR, &[]);
        let mut env = mock_env();
        env.block.height = 0;
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        deps
    }

    #[test]
    fn instantiate_works() {
        setup();
    }

    #[test]
    fn user_register_success() {
        let mut deps = setup();

        let moniker: String = "test".to_string();

        let register_msg = ExecuteMsg::Register{
            moniker: moniker
        };

        let raw = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), register_msg).unwrap();

        assert_eq!(raw, Response::new().add_attribute("action", "register".to_string()))
    }

    #[test]
    fn user_register_fail() {
        let mut deps = setup();

        let first_moniker: String = "test 1".to_string();
        let second_moniker: String = "test 2".to_string();

        let register_msg_1 = ExecuteMsg::Register{
            moniker: first_moniker
        };

        let register_msg_2 = ExecuteMsg::Register{
            moniker: second_moniker
        };

        let first_raw = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), register_msg_1).unwrap();

        let second_raw = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), register_msg_2).unwrap_err();
        

        match second_raw {
            ContractError::CustomError{val:a} => {assert_eq!(a, "Address has been registered !".to_string());},
            _ => panic!("")
        }
    }

    #[test]
    fn push_with_state_history_error_fail(){
        let mut deps = mock_dependencies();

        let moniker: String = "test".to_string();

        let register_msg = ExecuteMsg::Register{
            moniker: moniker
        };

        let register_response = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), register_msg).unwrap();

        assert_eq!(register_response, Response::new().add_attribute("action", "register".to_string()));

        let random_obj = "".to_string();
        let signature = "".to_string();

        let push_msg = ExecuteMsg::Push{
            random_obj, signature
        };

        let push_response = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), push_msg).unwrap_err();
        match push_response {
            ContractError::CustomError{val:a} => {assert_eq!(a, "State history error!".to_string());},
            _ => panic!("")
        }
    }

    #[test]
    fn push_with_block_height_not_reach_fail(){
        let mut deps = setup();

        let moniker: String = "test".to_string();

        let register_msg = ExecuteMsg::Register{
            moniker: moniker
        };

        let register_response = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), register_msg).unwrap();

        assert_eq!(register_response, Response::new().add_attribute("action", "register".to_string()));

        let random_obj = "".to_string();
        let signature = "".to_string();

        let push_msg = ExecuteMsg::Push{
            random_obj, signature
        };

        let mut env = mock_env();
        env.block.height = 1;

        let push_response = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), push_msg).unwrap_err();
        match push_response {
            ContractError::CustomError{val:a} => {assert_eq!(a, "Block height not reach!".to_string());},
            _ => panic!("")
        }
    }

    #[test]
    fn push_with_invalid_signature_fail(){
        let mut deps = setup();

        let moniker: String = "test".to_string();

        let register_msg = ExecuteMsg::Register{
            moniker: moniker
        };
        
        let register_response = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), register_msg).unwrap();

        assert_eq!(register_response, Response::new().add_attribute("action", "register".to_string()));

        let random_obj = "".to_string();
        let signature = "".to_string();

        let push_msg = ExecuteMsg::Push{
            random_obj, signature
        };

        let mut env = mock_env();
        env.block.height = 31;
        
        let push_response = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), push_msg).unwrap_err();
        match push_response {
            ContractError::CustomError{val:a} => {assert_eq!(a, "Verification failed!".to_string());},
            _ => panic!("")
        }
    }

    #[test]
    fn push_with_address_has_not_been_registered_fail(){
        let mut deps = setup();

        let moniker: String = "test".to_string();

        let register_msg = ExecuteMsg::Register{
            moniker: moniker
        };
        
        let register_response = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), register_msg).unwrap();

        assert_eq!(register_response, Response::new().add_attribute("action", "register".to_string()));

        let random_obj = "".to_string();
        let signature = "".to_string();

        let push_msg = ExecuteMsg::Push{
            random_obj, signature
        };

        let mut env = mock_env();
        env.block.height = 31;
        // /Address has't been registerd!
        let push_response = execute(deps.as_mut(), env, mock_info("", &[]), push_msg).unwrap_err();
        match push_response {
            ContractError::CustomError{val:a} => {assert_eq!(a, "Address has't been registerd!".to_string());},
            _ => panic!("")
        }
    }

    fn push_randome_value_success() -> OwnedDeps<MockStorage, MockApi, MockQuerier>{
        let mut deps = setup();

        let moniker: String = "test".to_string();

        let register_msg = ExecuteMsg::Register{
            moniker: moniker
        };
        
        let register_response = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), register_msg).unwrap();

        assert_eq!(register_response, Response::new().add_attribute("action", "register".to_string()));

        let random_obj = "eyJtZXRob2QiOiJnZW5lcmF0ZVNpZ25lZEludGVnZXJzIiwiaGFzaGVkQXBpS2V5IjoiSUVicnY4NzFLZnBsdjNmdkFaWG9rRFg2S1o1N0pES2wyajhLNktRZkxnRk12MDF6ZktsWnJweXFnQ3MyRE9rRVg4LzcvQ2xIZm0yZHFveFh3VVJkTHc9PSIsIm4iOjMyLCJtaW4iOjAsIm1heCI6MjU1LCJyZXBsYWNlbWVudCI6dHJ1ZSwiYmFzZSI6MTAsInByZWdlbmVyYXRlZFJhbmRvbWl6YXRpb24iOm51bGwsImRhdGEiOlszNCwxNTIsMTIyLDEyLDExOCw1MywxOTAsMzcsMjQsMTAsMCwxMDEsNjAsMTQ0LDE4NywxMSwxNzcsMTE0LDM3LDIxNywxNDksMjI0LDI2LDg3LDI5LDE0OSwxNTAsNzQsMTM2LDQ1LDE5OSwyMzJdLCJsaWNlbnNlIjp7InR5cGUiOiJkZXZlbG9wZXIiLCJ0ZXh0IjoiUmFuZG9tIHZhbHVlcyBsaWNlbnNlZCBzdHJpY3RseSBmb3IgZGV2ZWxvcG1lbnQgYW5kIHRlc3Rpbmcgb25seSIsImluZm9VcmwiOm51bGx9LCJsaWNlbnNlRGF0YSI6bnVsbCwidXNlckRhdGEiOm51bGwsInRpY2tldERhdGEiOm51bGwsImNvbXBsZXRpb25UaW1lIjoiMjAyMi0xMi0wOCAwMjo1MjoxNVoiLCJzZXJpYWxOdW1iZXIiOjl9".to_string();
        let signature = "0K510lwXPxj8AHPV+cQoYuW4snOtjd8NTytz16XC8PHSOMXJNOW3yVynSiuVf20mc1fLHbmKjP08//TfqPyIYWd40A9OA+iJcHz+VXRgwCzSH/RK2nnxqN7uuah2xCXXerfcW5g/sRkRHrPZIjoTPVR/adXdjZBQ6q4Wb0JXItYpFv5aUCEBQWa2izq7Ax+ZNZI0PjifI5zQacPheVxoyEGYB2TtsWWYIHDI+M5afK0E0yyOjiR+emozmD3M3KgLpYq8UkaGR4rSNNgNsrLTyupDebOouRlyevXmKZURWmXZnJlW8sJKrvvPGnUQrRSDbpxBOuaBpg0SPozIr8Avv1CJCngcaDumjQFCuesQvTjQACBwrsqGZoSSHtw3QgWQdcfPnZBWOQ3jlaVk897fCEI6TYOnT+U9spvFmVdtmSVhaeftmQ5+yDoYhe5YHf2AQcmUxlikyhBmob4Fv6VDmKgpy5Ke30zlaNhFdXonvQZk+wlqlsaYk7cnmtaxrMcHlQcpVHZRRLNc5FHg0nFepe0z/T30XmFEyyOQlrAmpwZ6tKwksXDykQW5AyPUY6+esCl3rDXdt3GFis8D6/WldOKuMiGKW/JN7w9zR8W7NGxJ4INv3eO7Er8yJoxyMvD6eQ3STO3pAjBZ37e43mx7F/pnxaFOPPFrk9dMcWdCPmw=".to_string();

        let push_msg = ExecuteMsg::Push{
            random_obj, signature
        };

        let mut env = mock_env();
        env.block.height = 31;
        // /Address has't been registerd!
        let push_response = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), push_msg).unwrap();
        
        assert_eq!(push_response, Response::new().add_attribute("action", "push".to_string()));
        deps
    }

    #[test]
    fn push_success() {
        push_randome_value_success();
    }

    #[test]
    fn get_random_value_success() {
        let mut deps = push_randome_value_success();

        let get_randome_value_msg = ExecuteMsg::GetRandomValue{};
        let get_response = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), get_randome_value_msg).unwrap();
        
        let random_obj_base64 = "eyJtZXRob2QiOiJnZW5lcmF0ZVNpZ25lZEludGVnZXJzIiwiaGFzaGVkQXBpS2V5IjoiSUVicnY4NzFLZnBsdjNmdkFaWG9rRFg2S1o1N0pES2wyajhLNktRZkxnRk12MDF6ZktsWnJweXFnQ3MyRE9rRVg4LzcvQ2xIZm0yZHFveFh3VVJkTHc9PSIsIm4iOjMyLCJtaW4iOjAsIm1heCI6MjU1LCJyZXBsYWNlbWVudCI6dHJ1ZSwiYmFzZSI6MTAsInByZWdlbmVyYXRlZFJhbmRvbWl6YXRpb24iOm51bGwsImRhdGEiOlszNCwxNTIsMTIyLDEyLDExOCw1MywxOTAsMzcsMjQsMTAsMCwxMDEsNjAsMTQ0LDE4NywxMSwxNzcsMTE0LDM3LDIxNywxNDksMjI0LDI2LDg3LDI5LDE0OSwxNTAsNzQsMTM2LDQ1LDE5OSwyMzJdLCJsaWNlbnNlIjp7InR5cGUiOiJkZXZlbG9wZXIiLCJ0ZXh0IjoiUmFuZG9tIHZhbHVlcyBsaWNlbnNlZCBzdHJpY3RseSBmb3IgZGV2ZWxvcG1lbnQgYW5kIHRlc3Rpbmcgb25seSIsImluZm9VcmwiOm51bGx9LCJsaWNlbnNlRGF0YSI6bnVsbCwidXNlckRhdGEiOm51bGwsInRpY2tldERhdGEiOm51bGwsImNvbXBsZXRpb25UaW1lIjoiMjAyMi0xMi0wOCAwMjo1MjoxNVoiLCJzZXJpYWxOdW1iZXIiOjl9".to_string();
        let signature_obj_base64 = "0K510lwXPxj8AHPV+cQoYuW4snOtjd8NTytz16XC8PHSOMXJNOW3yVynSiuVf20mc1fLHbmKjP08//TfqPyIYWd40A9OA+iJcHz+VXRgwCzSH/RK2nnxqN7uuah2xCXXerfcW5g/sRkRHrPZIjoTPVR/adXdjZBQ6q4Wb0JXItYpFv5aUCEBQWa2izq7Ax+ZNZI0PjifI5zQacPheVxoyEGYB2TtsWWYIHDI+M5afK0E0yyOjiR+emozmD3M3KgLpYq8UkaGR4rSNNgNsrLTyupDebOouRlyevXmKZURWmXZnJlW8sJKrvvPGnUQrRSDbpxBOuaBpg0SPozIr8Avv1CJCngcaDumjQFCuesQvTjQACBwrsqGZoSSHtw3QgWQdcfPnZBWOQ3jlaVk897fCEI6TYOnT+U9spvFmVdtmSVhaeftmQ5+yDoYhe5YHf2AQcmUxlikyhBmob4Fv6VDmKgpy5Ke30zlaNhFdXonvQZk+wlqlsaYk7cnmtaxrMcHlQcpVHZRRLNc5FHg0nFepe0z/T30XmFEyyOQlrAmpwZ6tKwksXDykQW5AyPUY6+esCl3rDXdt3GFis8D6/WldOKuMiGKW/JN7w9zR8W7NGxJ4INv3eO7Er8yJoxyMvD6eQ3STO3pAjBZ37e43mx7F/pnxaFOPPFrk9dMcWdCPmw=".to_string();

        let random_obj = base64::decode(random_obj_base64.clone()).unwrap();
        let signature = base64::decode(signature_obj_base64.clone()).unwrap();

        assert_eq!(get_response, Response::new()
                    .add_attribute("round", 1.to_string())
                    .add_attribute("randomness", hex::encode(&sha512_hash(random_obj.clone())))
                    .add_attribute("origin_data", hex::encode(&random_obj))
                    .add_attribute("signature", hex::encode(&signature)))
    }

    #[test]
    fn get_random_value_with_empty_data_success() {
        let mut deps = setup();

        let get_randome_value_msg = ExecuteMsg::GetRandomValue{};
        let get_response = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), get_randome_value_msg).unwrap();

        assert_eq!(get_response, Response::new()
        .add_attribute("round", "".to_string())
        .add_attribute("randomness", "".to_string())
        .add_attribute("origin_data", "".to_string())
        .add_attribute("signature", "".to_string()))
    }

    #[test]
    fn get_random_value_with_state_history_error_fail() {
        let mut deps = mock_dependencies();

        let get_randome_value_msg = ExecuteMsg::GetRandomValue{};
        let get_response = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), get_randome_value_msg).unwrap_err();

        match get_response {
            ContractError::CustomError{val:a} => {assert_eq!(a, "State history error!".to_string());},
            _ => panic!("")
        }
    }

}
