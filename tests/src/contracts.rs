use cw_orch::{Uploadable,WasmPath,CwEnv,Contract,Daemon,contract, TxHandler};

const CRATE_PATH: &str = env!("CARGO_MANIFEST_DIR");

#[contract(abstract_core::ans_host::InstantiateMsg,abstract_core::ans_host::ExecuteMsg,abstract_core::ans_host::QueryMsg,abstract_core::ans_host::MigrateMsg)]
pub struct ANSHost;


impl<Chain: CwEnv> ANSHost<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let contract = Contract::new(name, chain);
        Self(contract)
    }
}

impl Uploadable for ANSHost<Daemon> {
    fn wasm(&self) -> <Daemon as TxHandler>::ContractSource {
        WasmPath::new(format!(
            "{CRATE_PATH}/artifacts/abstract_and_host.wasm"
        ))
        .unwrap()
    }
}