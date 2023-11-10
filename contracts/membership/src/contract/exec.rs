use cosmwasm_std::{
    ensure, to_json_binary, Addr, DepsMut, Empty, Env, MessageInfo, Order, Response, SubMsg,
    WasmMsg,
};

use crate::{
    contract::PROXY_INSTANTIATION_REPLY_ID,
    error::ContractError,
    state::{CONFIG, MEMBERS, PROPOSALS, VOTES},
};

use proxy::msg::InstantiateMsg as ProxyInstantiateMsg;

pub fn propose_member(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    // check if the one who send this message is a member or not
    ensure!(
        MEMBERS.has(deps.storage, &info.sender),
        ContractError::Unauthorized
    );

    // validate address of the new member
    let addr = deps.api.addr_validate(&addr)?;

    // check if the sender is already a member
    // just think it like a normal for loop
    for member in MEMBERS.range(deps.storage, None, None, Order::Ascending) {
        let (member, _) = member?; // get proxy contract address
        let owner = proxy::state::OWNER.query(&deps.querier, member)?; // get the owner of the proxy contract
        ensure!(owner != addr, ContractError::AlreadyAMember);
    }

    // check if the sender has already voted for this new member
    ensure!(
        !VOTES.has(deps.storage, (&info.sender, &addr)),
        ContractError::AlreadyVoted
    );

    // if pass through all the checks, then store to VOTES and update PROPOSALS

    // use may_load here because maybe this the first time that the new member is proposed
    let number_votes_of_new_member = PROPOSALS.may_load(deps.storage, &addr)?.unwrap_or(0) + 1;
    VOTES.save(deps.storage, (&info.sender, &addr), &Empty {})?;

    // get the minimal acceptances from CONFIG
    let config = CONFIG.load(deps.storage)?;
    let minimal_acceptances = config.minimal_acceptances;

    // it means that the new member need more votes to be accepted
    if number_votes_of_new_member < minimal_acceptances {
        PROPOSALS.save(deps.storage, &addr, &number_votes_of_new_member)?;

        let resp = Response::new()
            .add_attribute("action", "propose member")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("new_member", addr.as_str())
            .add_attribute("number_of_votes", number_votes_of_new_member.to_string());

        return Ok(resp);
    }

    // so if below code is executed, it means that the new member is accepted
    // then we create for him a proxy contract

    // what happen if not delete?
    PROPOSALS.remove(deps.storage, &addr);

    let proxy_init_msg = ProxyInstantiateMsg {
        owner: addr.to_string(),
        weight: config.starting_weight,
        denom: config.denom,
        direct_part: config.direct_part,
        distribution_contract: config.distribution_contract.into_string(),
        membership_contract: env.contract.address.to_string(),
        halftime: config.halftime,
    };

    let proxy_init_msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.into_string()),
        code_id: config.proxy_code_id,
        msg: to_json_binary(&proxy_init_msg)?,
        funds: vec![],
        label: format!("{} proxy", addr),
    };

    let proxy_init_msg = SubMsg::reply_on_success(proxy_init_msg, PROXY_INSTANTIATION_REPLY_ID);

    let resp = Response::new()
        .add_submessage(proxy_init_msg)
        .add_attribute("action", "propose member")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("new_member", addr.as_str());
    Ok(resp)
}
