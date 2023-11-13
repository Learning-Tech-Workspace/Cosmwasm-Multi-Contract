use anyhow::Result as AnyResult;
use cosmwasm_std::{from_json, Addr, Coin, Decimal};
use cw_multi_test::{App, ContractWrapper, Executor};
use cw_utils::parse_execute_response_data;

use crate::{
    execute, instantiate,
    msg::{ExecMsg, InstantiateMsg, MembershipExecMsg, ProposeMemberData},
    query, reply,
};

#[derive(Clone, Copy, Debug)]
pub struct CodeId(u64);

impl CodeId {
    pub fn store_code(app: &mut App) -> Self {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);

        CodeId(app.store_code(Box::new(contract)))
    } // return instance of Self
      // remember need to declare all entry point in ContractWrapper

    #[track_caller]
    pub fn instantiate(
        self,
        app: &mut App,
        sender: &str,
        owner: &str,
        weight: u64,
        denom: &str,
        direct_part: Decimal,
        distribution_contract: &str,
        membership_contract: &str,
        halftime: u64,
        label: &str,
    ) -> AnyResult<Contract> {
        // this is static function?
        Contract::instantiate(
            app,
            self,
            sender,
            owner,
            weight,
            denom,
            direct_part,
            distribution_contract,
            membership_contract,
            halftime,
            label,
        )
    }
}

impl From<CodeId> for u64 {
    fn from(value: CodeId) -> Self {
        value.0
    }
}

// tuple struct
#[derive(Debug)]
pub struct Contract(Addr);

impl Contract {
    // membership contract response will contain proxy address => this function take it as argument and create a new instance of Contract - for testing
    pub fn from_addr(addr: Addr) -> Self {
        Self(addr)
    }

    // this one is get the address of the contract
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: CodeId,
        sender: &str,
        owner: &str,
        weight: u64,
        denom: &str,
        direct_part: Decimal,
        distribution_contract: &str,
        membership_contract: &str,
        halftime: u64,
        label: &str,
    ) -> AnyResult<Self> {
        let init_msg = InstantiateMsg {
            owner: owner.to_owned(),
            weight,
            denom: denom.to_owned(),
            direct_part,
            distribution_contract: distribution_contract.to_owned(),
            membership_contract: membership_contract.to_owned(),
            halftime,
        };

        app.instantiate_contract(
            code_id.0,
            Addr::unchecked(sender),
            &init_msg,
            &[],
            label,
            None,
        )
        .map(Self) // not understand this => need to re-watch the video of previous course
                   // i think .map will create a new instance of Self => and return it
                   // Self in this case is a tuple struct
                   // self is instance of Self
                   // .map take closure
                   // |contract_addr| Contract(contract_addr)
    }

    #[track_caller]
    pub fn donate(&self, app: &mut App, sender: &str, funds: &[Coin]) -> AnyResult<()> {
        let donate_msg = ExecMsg::Donate {};
        app.execute_contract(Addr::unchecked(sender), self.0.clone(), &donate_msg, funds)?;

        Ok(())
    }

    #[track_caller]
    pub fn propose_member(
        &self,
        app: &mut App,
        sender: &str,
        addr: &str,
    ) -> AnyResult<Option<ProposeMemberData>> {
        let propose_member_msg = MembershipExecMsg::ProposeMember {
            addr: addr.to_owned(),
        };
        let resp = app.execute_contract(
            Addr::unchecked(sender),
            self.0.clone(),
            &propose_member_msg,
            &[],
        )?;

        // response from the reply handler (fn propose_member) of proxy contract
        // not understand
        resp.data
            .map(|data| parse_execute_response_data(&data))
            .transpose()?
            .and_then(|data| data.data)
            .map(|data| from_json(&data))
            .transpose()
            .map_err(Into::into)
    }
}
