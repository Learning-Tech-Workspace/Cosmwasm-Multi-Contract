use anyhow::Result as AnyResult;
use common::msg::ProposeMemberData;
use cosmwasm_std::{from_json, to_json_binary, Addr, Decimal, WasmMsg};
use cw_multi_test::{App, ContractWrapper, Executor};
use cw_utils::{parse_execute_response_data, parse_instantiate_response_data};

use crate::{
    execute, instantiate,
    msg::{ExecMsg, InstantiateMsg, InstantiationData, IsMemberResponse, QueryMsg},
    query, reply,
};

mod test;

#[derive(Clone, Copy, Debug)]
pub struct CodeId(u64);

impl CodeId {
    pub fn store_code(app: &mut App) -> Self {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        CodeId(app.store_code(Box::new(contract)))
    }

    #[track_caller]
    pub fn instantiate(
        self,
        app: &mut App,
        sender: &str,
        starting_weight: u64,
        denom: &str,
        direct_part: Decimal,
        halftime: u64,
        minimal_acceptance: u64,
        proxy_code_id: proxy::multitest::CodeId,
        //distribution_code_id: distribution::multitest::CodeId,
        initial_members: &[&str],
        label: &str,
    ) -> AnyResult<(Contract, InstantiationData)> {
        Contract::instantiate(
            app,
            self,
            sender,
            starting_weight,
            denom,
            direct_part,
            halftime,
            minimal_acceptance,
            proxy_code_id,
            initial_members,
            label,
        )
    }
}

impl From<CodeId> for u64 {
    fn from(value: CodeId) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct Contract(Addr);

impl Contract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: CodeId,
        sender: &str,

        starting_weight: u64,
        denom: &str,
        direct_part: Decimal,
        halftime: u64,
        minimal_acceptance: u64,
        proxy_code_id: proxy::multitest::CodeId,
        //distribution_code_id: distribution::multitest::CodeId,
        initial_members: &[&str],
        label: &str,
    ) -> AnyResult<(Self, InstantiationData)> {
        let init_msg = InstantiateMsg {
            starting_weight,
            denom: denom.to_owned(),
            direct_part,
            halftime,
            proxy_code_id: proxy_code_id.into(), // can use this because From<CodeId> for u64
            distribution_code_id: 0,             // cause we don't have distribution contract yet
            minimal_acceptance,
            initial_members: initial_members // need to find out this
                .iter()
                .map(|addr| addr.to_string())
                .collect(),
        };

        // instantiate membership contract
        let init_msg = WasmMsg::Instantiate {
            admin: None,
            code_id: code_id.0,
            msg: to_json_binary(&init_msg)?,
            funds: vec![],
            label: label.into(),
        };

        // execute can get the the whole AppResponse which contain data and events emitted on execution
        // instantiate_contract will return directly the address of the contract so we can not get our custom data
        let resp = app.execute(Addr::unchecked(sender), init_msg.into())?;

        let data = parse_instantiate_response_data(resp.data.unwrap_or_default().as_slice())?;

        let contract = Self(Addr::unchecked(data.contract_address));
        let data = from_json(&data.data.unwrap_or_default())?;

        Ok((contract, data))
    }

    pub fn propose_member(
        &self,
        app: &mut App,
        sender: &str,
        addr: &str,
    ) -> AnyResult<Option<ProposeMemberData>> {
        let propose_member_msg = ExecMsg::ProposeMember {
            addr: addr.to_owned(),
        };

        let resp = app.execute_contract(
            Addr::unchecked(sender),
            self.0.clone(),
            &propose_member_msg,
            &[],
        )?;

        // this is my code
        let data = parse_execute_response_data(resp.data.unwrap_or_default().as_slice())?;

        let data = from_json(&data.data.unwrap_or_default())?;

        // this is code from repo
        // resp.data
        //     .map(|data| parse_execute_response_data(&data))
        //     .transpose()?
        //     .and_then(|data| data.data)
        //     .map(|data| from_json(&data))
        //     .transpose()
        //     .map_err(Into::into);

        // so look like my code and repo code are doing the same result but my code is more readable

        Ok(data)
    }

    pub fn is_member(&self, app: &App, addr: &str) -> AnyResult<IsMemberResponse> {
        // this is my code different from repo
        let resp = app.wrap().query_wasm_smart(
            self.0.clone(),
            &QueryMsg::IsMember {
                addr: addr.to_owned(),
            },
        );
        Ok(resp?)
    }
}
