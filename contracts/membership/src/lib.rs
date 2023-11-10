use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Reply, Response};
use error::ContractError;
use msg::InstantiateMsg;

mod contract;
pub mod error;
pub mod msg;
pub mod state;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    contract::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    contract::reply(deps, env, reply)
}
