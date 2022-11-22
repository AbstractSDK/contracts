use boot_core::prelude::boot_contract;
use boot_core::{BootEnvironment, Contract};
use cosmwasm_std::Empty;

use abstract_sdk::os::{extension, nois};

type NoisExtensionInstantiateMsg = extension::InstantiateMsg<nois::NoisInstantiateMsg>;
type NoisExtensionExecuteMsg = extension::ExecuteMsg<nois::NoisRequestMsg>;
type NoisExtensionQueryMsg = extension::QueryMsg<nois::NoisQueryMsg>;

#[boot_contract(
    NoisExtensionInstantiateMsg,
    NoisExtensionExecuteMsg,
    NoisExtensionQueryMsg,
    Empty
)]
pub struct NoisExtension<Chain>;

impl<Chain: BootEnvironment> NoisExtension<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("nois"),
            // .with_mock(Box::new(
            //     ContractWrapper::new_with_empty(
            //         ::contract::execute,
            //         ::contract::instantiate,
            //         ::contract::query,
            //     ),
            // ))
        )
    }
}
