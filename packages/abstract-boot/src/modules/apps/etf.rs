use abstract_os::{app::MigrateMsg, etf::*};
use boot_core::{prelude::boot_contract, BootEnvironment, Contract};

#[boot_contract(EtfInstantiateMsg, EtfExecuteMsg, EtfQueryMsg, MigrateMsg)]
pub struct ETF<Chain>;

impl<Chain: BootEnvironment> ETF<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("etf");
        #[cfg(feature = "testing")]
        contract.set_mock(Box::new(cw_multi_test::ContractWrapper::new_with_empty(
            ::etf::contract::execute,
            ::etf::contract::instantiate,
            ::etf::contract::query,
        )));
        Self(contract)
    }
}
