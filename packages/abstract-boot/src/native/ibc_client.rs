use abstract_core::ibc_client::*;
use cw_orch::{ArtifactsDir, Contract, CwEnv, Daemon, Uploadable};

pub use abstract_core::ibc_client::{
    ExecuteMsgFns as IbcClientExecFns, QueryMsgFns as IbcClientQueryFns,
};
use cw_orch::contract;

#[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl Uploadable for IbcClient<Daemon> {
    fn wasm(&self) -> cw_orch::WasmPath {
        ArtifactsDir::env().find_wasm_path("ibc_client").unwrap()
    }
}

impl<Chain: CwEnv> IbcClient<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}
