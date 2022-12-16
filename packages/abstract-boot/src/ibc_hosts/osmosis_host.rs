use abstract_os::ibc_host::*;
use boot_core::{prelude::boot_contract, BootEnvironment, Contract};
use cosmwasm_std::Empty;

#[boot_contract(InstantiateMsg, Empty, QueryMsg, MigrateMsg)]
pub struct OsmosisHost<Chain>;

impl<Chain: BootEnvironment> OsmosisHost<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("osmosis_host");
        #[cfg(feature = "testing")]
        contract.set_mock(Box::new(cw_multi_test::ContractWrapper::new_with_empty(
            ::osmosis_host::contract::execute,
            ::osmosis_host::contract::instantiate,
            ::osmosis_host::contract::query,
        )));
        Self(contract)
    }
}
