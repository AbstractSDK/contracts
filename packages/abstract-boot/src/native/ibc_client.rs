use abstract_core::ibc_client::*;
use cw_orc::{Contract, CwEnv};

pub use abstract_core::ibc_client::{
    ExecuteMsgFns as IbcClientExecFns, QueryMsgFns as IbcClientQueryFns,
};
use cw_orc::contract;

#[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl<Chain: CwEnv> IbcClient<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("abstract_ibc_client");
        Self(contract)
    }
}
