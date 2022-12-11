//! # AnsHost Entry
//! An entry (value) in the ans_host key-value store.

use cosmwasm_std::{Addr, Deps, StdResult};
use cw_asset::AssetInfo;

use abstract_os::objects::{ans_host::AnsHost, AssetEntry, ChannelEntry, ContractEntry};

pub trait Resolve {
    type Output;
    fn resolve(&self, deps: Deps, ans_host: &AnsHost) -> StdResult<Self::Output>;
}
