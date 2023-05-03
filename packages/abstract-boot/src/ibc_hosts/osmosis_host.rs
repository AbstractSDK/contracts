use abstract_core::ibc_host::*;
use cosmwasm_std::Empty;
use cw_orch::{contract, Contract, CwEnv};

#[contract(InstantiateMsg, Empty, QueryMsg, MigrateMsg)]
pub struct OsmosisHost<Chain>;

impl<Chain: CwEnv> OsmosisHost<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}
