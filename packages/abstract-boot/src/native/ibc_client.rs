use cw_orch::prelude::{ArtifactsDir, WasmPath};
use abstract_core::ibc_client::*;
use cw_orch::{Contract, CwEnv, Daemon, Uploadable};

pub use abstract_core::ibc_client::{
    ExecuteMsgFns as IbcClientExecFns, QueryMsgFns as IbcClientQueryFns,
};
use cw_orch::interface;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl Uploadable for IbcClient<Daemon> {
    fn wasm(&self) -> WasmPath {
        ArtifactsDir::env().find_wasm_path("ibc_client").unwrap()
    }
}

impl<Chain: CwEnv> IbcClient<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}
