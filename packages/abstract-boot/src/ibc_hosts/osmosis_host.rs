use cw_orch::prelude::{WasmPath, ArtifactsDir};
use abstract_core::ibc_host::*;
use cw_orch::{interface, Contract, CwEnv, Daemon, Uploadable};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct OsmosisHost;

impl Uploadable for OsmosisHost<Daemon> {
    fn wasm(&self) -> WasmPath {
        ArtifactsDir::env().find_wasm_path("ibc_host").unwrap()
    }
}

impl<Chain: CwEnv> OsmosisHost<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}
