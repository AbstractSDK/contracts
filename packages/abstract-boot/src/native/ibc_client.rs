use boot_core::{BootEnvironment, Contract};

use abstract_os::ibc_client::*;

pub use abstract_os::ibc_client::{
    ExecuteMsgFns as IbcClientExecFns, QueryMsgFns as IbcClientQueryFns,
};
use boot_core::prelude::boot_contract;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl<Chain: BootEnvironment> IbcClient<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("ibc_client");
        #[cfg(feature = "testing")]
        contract.set_mock(Box::new(cw_multi_test::ContractWrapper::new_with_empty(
            ::ibc_client::contract::execute,
            ::ibc_client::contract::instantiate,
            ::ibc_client::contract::query,
        )));
        Self(contract)
    }
}
