use cosmwasm_std::{coins, BankMsg, DepsMut, Env, Response};

use crate::{
    error::ContractError,
    state::{CONFIG, PENDING_WITHDRAWAL},
};

// distribution contract send reply to proxy contract when it finish in handle the withdraw message sent from proxy contract (that mean the distribution contract has already send token to proxy contract)

// in the flow this is the last step that proxy contract send token to receiver
pub fn withdraw(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // this is the use of PENDING_WITHDRAWAL
    // when the execution entry point finished (withdraw handler proxy contract) and message is processed as part of transaction (in this case is the withdraw message sent from proxy contract to distribution contract) then we lost all the information that we pass with withdraw message to proxy contract
    let withdraw_info = PENDING_WITHDRAWAL.load(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    // remember the distribution contract may has already send distributed token of owner to proxy contract => so we just need query balance of proxy contract
    // balance of proxy contract consists of 2 part
    let total_amount = deps
        .querier
        .query_balance(env.contract.address, &config.denom)?;

    let amount = withdraw_info.amount.unwrap_or(total_amount.amount);

    // send token to receiver

    let bank_msg = BankMsg::Send {
        to_address: withdraw_info.receiver.into_string(),
        amount: coins(amount.u128(), &config.denom),
    };

    let resp = Response::new()
        .add_message(bank_msg)
        .add_attribute("amount", amount.to_string());

    Ok(resp)
}
