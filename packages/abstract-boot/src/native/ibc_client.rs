use boot_core::{BootEnvironment, Contract};
use iabstract::ibc_client::*;

use boot_core::boot_contract;
pub use iabstract::ibc_client::{
    ExecuteMsgFns as IbcClientExecFns, QueryMsgFns as IbcClientQueryFns,
};

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl<Chain: BootEnvironment> IbcClient<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("ibc_client");
        Self(contract)
    }
}
