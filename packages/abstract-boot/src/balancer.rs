pub use abstract_sdk::os::balancer::*;
use boot_core::{Contract, IndexResponse, TxHandler, TxResponse};
use cosmwasm_std::Empty;



#[boot_contract( ExecuteMsg, InstantiateMsg, QueryMsg, Empty)]
pub struct Balancer;

impl<Chain: BootEnvironment> Balancer<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("balancer"), // .with_mock(Box::new(
                                                                   //     ContractWrapper::new_with_empty(
                                                                   //         ::contract::execute,
                                                                   //         ::contract::instantiate,
                                                                   //         ::contract::query,
                                                                   //     ),
                                                                   // ))
        )
    }
}
