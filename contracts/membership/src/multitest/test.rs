use std::collections::HashMap;

use cosmwasm_std::Decimal;
use cw_multi_test::App;

use crate::multitest::CodeId as MembershipId;
use proxy::multitest::CodeId as ProxyId;

use proxy::multitest::Contract as ProxyContract;

#[test]
pub fn adding_member() {
    let mut app = App::default(); // blockchain

    let denom = "ORAI";

    let owner = "owner"; // owner of system
    let initial_members = ["member1", "member2"];
    let candidate = "candidate";

    // deploy code to blockchain => get code_id
    let proxy_code_id = ProxyId::store_code(&mut app);
    let membership_code_id = MembershipId::store_code(&mut app);

    // the reason to have the contract instantiate fn in the CodeId Wrapper
    // instantiate membership contract (create proxy contracts for initial members)
    let (membership_contract, instantiation_data) = membership_code_id
        .instantiate(
            &mut app,
            owner,
            10,
            denom,
            Decimal::percent(15),
            3600 * 24 * 30, // 30 days => update weight
            2,
            proxy_code_id,
            &initial_members,
            "Membership",
        )
        .unwrap(); // so remember if can not use ? operator, use unwrap() instead
                   // but priority is ? operator

    let proxies: HashMap<_, _> = instantiation_data
        .members
        .into_iter()
        .map(|member| {
            (
                member.owner_addr.into_string(),
                ProxyContract::from_addr(member.proxy_addr),
            )
        })
        .collect();
    // proxies is a HashMap with key is owner_addr and value is proxy contract instance

    // 2 initial proxy contracts created
    assert_eq!(proxies.len(), 2);
    assert!(
        membership_contract
            .is_member(&app, proxies[initial_members[0]].addr().as_str()) // initial_members[0] is key => return the Proxy Wrapper then call .addr() to get the address
            .unwrap()
            .is_member
    );
    assert!(
        membership_contract
            .is_member(&app, proxies[initial_members[1]].addr().as_str())
            .unwrap()
            .is_member
    );

    let data = proxies[initial_members[0]]
        .propose_member(&mut app, initial_members[0], candidate)
        .unwrap();

    assert!(data.is_none());

    let data = proxies[initial_members[1]]
        .propose_member(&mut app, initial_members[1], candidate)
        .unwrap();

    let data = data.unwrap();

    assert_eq!(data.owner_addr, candidate);

    assert!(
        membership_contract
            .is_member(&app, data.proxy_addr.as_str())
            .unwrap()
            .is_member
    );
}
