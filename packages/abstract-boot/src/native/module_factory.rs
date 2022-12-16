use abstract_os::module_factory::*;

// use crate::api::get_api_init_msgs;
use boot_core::{BootEnvironment, BootError, Contract, TxResponse};

pub use abstract_os::module_factory::{
    ExecuteMsgFns as MFactoryExecFns, QueryMsgFns as MFactoryQueryFns,
};
use boot_core::{interface::BootExecute, prelude::boot_contract};

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct ModuleFactory<Chain>;

impl<Chain: BootEnvironment> ModuleFactory<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("module_factory");
        #[cfg(feature = "testing")]
        contract.set_mock(Box::new(cw_multi_test::ContractWrapper::new_with_empty(
            ::module_factory::contract::execute,
            ::module_factory::contract::instantiate,
            ::module_factory::contract::query,
        )));
        Self(contract)
    }

    pub fn change_ans_host_addr(&self, mem_addr: String) -> Result<TxResponse<Chain>, BootError> {
        self.execute(
            &ExecuteMsg::UpdateConfig {
                admin: None,
                ans_host_address: Some(mem_addr),
                version_control_address: None,
            },
            None,
        )
    }

    // pub  fn save_init_binaries(&self, mem_addr: String, version_control_addr: String) -> Result<(), BootError> {
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
