use common::msg::ProposeMemberData;
use cosmwasm_std::{
    to_json_binary, Addr, DepsMut, Empty, Order, Response, StdError, StdResult, SubMsgResponse,
};
use cw_utils::parse_instantiate_response_data;

use crate::{
    error::ContractError,
    msg::InstantiationData,
    state::{AWAITING_INITIAL_RESPS, MEMBERS},
};

// summarize: we will have the proxy contract address from the reply and store it into MEMBERS
pub fn initial_proxy_instantiated(
    deps: DepsMut,
    reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let response = reply.map_err(StdError::generic_err)?;
    let data = response.data.ok_or(ContractError::MissingData)?; // CosmWasm executor uses this data field to add information of created proxy contract
    let response = parse_instantiate_response_data(&data)?;
    let proxy_addr = Addr::unchecked(response.contract_address);
    MEMBERS.save(deps.storage, &proxy_addr, &cosmwasm_std::Empty {})?;

    // means we have one less reply to wait for
    let awaiting = AWAITING_INITIAL_RESPS.load(deps.storage)? - 1;
    if awaiting > 0 {
        AWAITING_INITIAL_RESPS.save(deps.storage, &awaiting)?; // github repo saved 0 which is wrong and be fixed in the next commit

        let resp = Response::new().add_attribute("proxy_addr", proxy_addr);

        return Ok(resp);
    }

    // the final reply will execute the code below

    let members = MEMBERS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|member| -> StdResult<_> {
            let (member, _) = member?;
            // there is no way to update other contract's state but we can query it from another contract
            let owner = proxy::state::OWNER.query(&deps.querier, member.clone())?;
            let data = ProposeMemberData {
                owner_addr: owner.into(),
                proxy_addr: member.into(),
            };
            Ok(data)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let data = InstantiationData { members };
    let resp = Response::new()
        .add_attribute("proxy addr", proxy_addr.as_str())
        .set_data(to_json_binary(&data)?);

    Ok(resp)
}

pub fn proxy_instantiated(
    deps: DepsMut,
    reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let response = reply.map_err(StdError::generic_err)?;
    let data = response.data.ok_or(ContractError::MissingData)?;
    let response = parse_instantiate_response_data(&data)?;
    let addr = Addr::unchecked(response.contract_address); // proxy contract address

    let owner = proxy::state::OWNER.query(&deps.querier, addr.clone())?;

    MEMBERS.save(deps.storage, &addr, &Empty {})?;

    let data = ProposeMemberData {
        owner_addr: owner.into(),
        proxy_addr: addr.to_string(),
    };

    let resp = Response::new()
        .add_attribute("proxy_addr", addr.as_str())
        .set_data(to_json_binary(&data)?);

    Ok(resp)
}
