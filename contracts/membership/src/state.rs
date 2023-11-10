use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Empty};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub starting_weight: u64,
    pub denom: String,
    pub direct_part: Decimal,
    pub halftime: u64,
    pub proxy_code_id: u64,
    pub distribution_contract: Addr,
    pub minimal_acceptances: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const MEMBERS: Map<&Addr, Empty> = Map::new("members"); // proxy contracts addresses
pub const PROPOSALS: Map<&Addr, u64> = Map::new("proposals");
// candidate want to join group, and number of their proposals

pub const VOTES: Map<(&Addr, &Addr), Empty> = Map::new("votes");
// member in system, candidate want to join group

pub const AWAITING_INITIAL_RESPS: Item<u64> = Item::new("awaiting_initial_resps");
