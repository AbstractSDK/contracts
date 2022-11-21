use abstract_sdk::os::tendermint_staking::*;
use cosmwasm_std::Empty;

use boot_core::{BootEnvironment, Contract};

use boot_core::prelude::boot_contract;

#[boot_contract(
    abstract_sdk::os::extension::InstantiateMsg,
    RequestMsg,
    abstract_sdk::os::tendermint_staking::QueryMsg,
    Empty
)]
// #[boot_contract(abstract_sdk::os::extension::InstantiateMsg, ExecuteMsg<RequestMsg>, abstract_sdk::os::extension::QueryMsg<abstract_sdk::os::tendermint_staking::QueryMsg>, Empty)]
pub struct TMintStakingExtension<Chain>;

impl<Chain: BootEnvironment> TMintStakingExtension<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("tendermint_staking"),
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
