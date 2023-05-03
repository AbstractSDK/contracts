use abstract_core::module_factory::*;

// use crate::api::get_api_init_msgs;
use cw_orch::{ArtifactsDir, Contract, CwEnv, TxResponse};

pub use abstract_core::module_factory::{
    ExecuteMsgFns as MFactoryExecFns, QueryMsgFns as MFactoryQueryFns,
};
use cw_orch::{contract, CwOrcExecute};

#[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct ModuleFactory<Chain>;

impl<Chain: CwEnv> ::cw_orch::Uploadable for ModuleFactory<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper(&self) -> <::cw_orch::Mock as ::cw_orch::TxHandler>::ContractSource {
        Box::new(
            cw_orch::ContractWrapper::new_with_empty(
                ::module_factory::contract::execute,
                ::module_factory::contract::instantiate,
                ::module_factory::contract::query,
            )
            .with_migrate(::module_factory::contract::migrate)
            .with_reply(::module_factory::contract::reply),
        )
    }
    fn wasm(&self) -> cw_orch::WasmPath {
        ArtifactsDir::env()
            .find_wasm_path("module_factory")
            .unwrap()
    }
}

impl<Chain: CwEnv> ModuleFactory<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }

    pub fn change_ans_host_addr(
        &self,
        mem_addr: String,
    ) -> Result<TxResponse<Chain>, crate::AbstractBootError> {
        self.execute(
            &ExecuteMsg::UpdateConfig {
                ans_host_address: Some(mem_addr),
                version_control_address: None,
            },
            None,
        )
        .map_err(Into::into)
    }

    // pub  fn save_init_binaries(&self, mem_addr: String, version_control_addr: String) -> Result<(), crate::AbstractBootError> {
    //     let msgs = get_api_init_msgs(mem_addr,version_control_addr);
    //     // TODO: Add version management support
    //     let binaries = msgs
    //         .iter()
    //         .map(|(name, msg)| ((name.clone(), "v0.1.0".to_string()), msg.clone()))
    //         .collect::<Vec<_>>();
    //     self.0
    //         .execute(
    //             &ExecuteMsg::UpdateFactoryBinaryMsgs {
    //                 to_add: binaries,
    //                 to_remove: vec![(LIQUIDITY_INTERFACE.to_string(), "v0.1.0".to_string())],
    //             },
    //             &vec![],
    //         )
    //         ?;
    //     Ok(())
    // }
}
