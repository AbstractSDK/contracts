use abstract_core::ibc_host::*;
use cw_orch::{contract, ArtifactsDir, Contract, CwEnv, Daemon, Uploadable};

#[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct OsmosisHost<Chain>;

impl Uploadable for OsmosisHost<Daemon> {
    fn wasm(&self) -> cw_orch::WasmPath {
        ArtifactsDir::env().find_wasm_path("ibc_host").unwrap()
    }
}

impl<Chain: CwEnv> OsmosisHost<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}
