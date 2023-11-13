use cosmwasm_std::{Addr, Deps, StdResult};

use crate::{msg::IsMemberResponse, state::MEMBERS};

pub fn is_member(deps: Deps, addr: String) -> StdResult<IsMemberResponse> {
    let is_member = MEMBERS.has(deps.storage, &Addr::unchecked(addr));
    Ok(IsMemberResponse { is_member })
}
