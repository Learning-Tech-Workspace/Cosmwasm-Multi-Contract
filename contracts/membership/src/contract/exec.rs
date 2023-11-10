use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::error::ContractError;

pub fn propose_member(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    add: String,
) -> Result<Response, ContractError> {
    unimplemented!()
}
