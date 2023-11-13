use std::vec;

use cosmwasm_std::{
    ensure, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, WasmMsg,
};

mod exec;
mod query;
mod reply;

use crate::error::ContractError;
use crate::msg::{ExecMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, AWAITING_INITIAL_RESPS, CONFIG};

// Get instantiate msg of proxy contract
use proxy::msg::InstantiateMsg as ProxyInstantiateMsg;

const INITIAL_PROXY_INSTANTIATION_REPLY_ID: u64 = 1;
const PROXY_INSTANTIATION_REPLY_ID: u64 = 2;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    ensure!(
        msg.minimal_acceptance >= 2,
        ContractError::NotEnoughRequiredAcceptances
    );

    ensure!(
        msg.initial_members.len() as u64 >= msg.minimal_acceptance,
        ContractError::NotEnoughInitialMembers
    );

    let config = Config {
        starting_weight: msg.starting_weight,
        denom: msg.denom.clone(),
        direct_part: msg.direct_part,
        halftime: msg.halftime,
        proxy_code_id: msg.proxy_code_id,
        distribution_contract: Addr::unchecked(""), // cause we don't have distribution contract yet
        minimal_acceptances: msg.minimal_acceptance,
    };

    CONFIG.save(deps.storage, &config)?;

    let proxy_instantiate_msgs: Vec<_> = msg
        .initial_members
        .into_iter()
        .map(|member| -> Result<_, ContractError> {
            // can use ? operator here because return Result<Response, ContractError>

            // validate address
            let addr = deps.api.addr_validate(&member)?;

            let proxy_init_msg = ProxyInstantiateMsg {
                owner: addr.to_string(),
                weight: msg.starting_weight,
                denom: msg.denom.clone(),
                direct_part: msg.direct_part,
                distribution_contract: "".to_owned(), // cause we don't have distribution contract yet
                membership_contract: env.contract.address.to_string(),
                halftime: msg.halftime,
            };

            // blockchain will instantiate proxy contract with below information
            let msg = WasmMsg::Instantiate {
                admin: Some(env.contract.address.to_string()),
                code_id: msg.proxy_code_id,
                // this one will go to entry point of proxy contract
                msg: to_json_binary(&proxy_init_msg)?, // to_binary deprecated
                funds: vec![],
                label: format!("{} Proxy", addr),
            };

            // use SubMsg so that our membership contract can get the reply with INITIAL_PROXY_INSTANTIATION_REPLY_ID
            let msg = SubMsg::reply_on_success(msg, INITIAL_PROXY_INSTANTIATION_REPLY_ID);

            Ok(msg)
        })
        .collect::<Result<_, _>>()?;

    AWAITING_INITIAL_RESPS.save(deps.storage, &(proxy_instantiate_msgs.len() as _))?;

    let resp = Response::new().add_submessages(proxy_instantiate_msgs);
    // these submessages provide reply to reply entry point of membership contract with INITIAL_PROXY_INSTANTIATION_REPLY_ID
    // and membership contract will have reply handler for those replies

    Ok(resp)
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecMsg::ProposeMember { addr } => exec::propose_member(deps, env, info, addr),
    }
}

pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        INITIAL_PROXY_INSTANTIATION_REPLY_ID => {
            reply::initial_proxy_instantiated(deps, reply.result.into_result())
        }
        PROXY_INSTANTIATION_REPLY_ID => reply::proxy_instantiated(deps, reply.result.into_result()),
        id => Err(ContractError::UnrecognizedReplyId(id)),
    }
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsMember { addr } => to_json_binary(&query::is_member(deps, addr)?),
    }
}
