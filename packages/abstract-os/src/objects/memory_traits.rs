//! # Memory Entry
//! An entry (value) in the memory key-value store.

use std::{
    convert::{TryFrom, TryInto},
    fmt::Display,
};

use cosmwasm_std::{Addr, Deps, StdError, StdResult};
use cw_asset::AssetInfo;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{asset_entry::AssetEntry, contract_entry::ContractEntry, memory::Memory};

pub trait Resolve {
    type Output;
    fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Self::Output>;
}

impl Resolve for AssetEntry {
    type Output = AssetInfo;
    fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Self::Output> {
        memory.query_asset(deps, self)
    }
}

impl Resolve for ContractEntry {
    type Output = Addr;
    fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Self::Output> {
        memory.query_contract(deps, self)
    }
}
