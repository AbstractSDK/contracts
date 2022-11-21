use boot_core::{Contract, IndexResponse, TxHandler, TxResponse, prelude::boot_contract, BootEnvironment};


use abstract_sdk::os::ibc_client::*;

#[boot_contract( ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient;

impl<Chain: BootEnvironment> IbcClient<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("ibc_client"), // .with_mock(Box::new(
                                                                     //     ContractWrapper::new_with_empty(
                                                                     //         ::contract::execute,
                                                                     //         ::contract::instantiate,
                                                                     //         ::contract::query,
                                                                     //     ),
                                                                     // ))
        )
    }
}
