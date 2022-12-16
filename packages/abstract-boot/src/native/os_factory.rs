use boot_core::{
    prelude::boot_contract, state::StateInterface, BootEnvironment, BootError, Contract,
    IndexResponse, TxResponse,
};
use cosmwasm_std::Addr;

pub use abstract_os::os_factory::{
    ExecuteMsgFns as OsFactoryExecFns, QueryMsgFns as OsFactoryQueryFns,
};
use abstract_os::{objects::gov_type::GovernanceDetails, os_factory::*};
use abstract_os::{MANAGER, PROXY};
use boot_core::interface::BootExecute;
use boot_core::interface::ContractInstance;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct OSFactory<Chain>;

impl<Chain: BootEnvironment> OSFactory<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("os_factory");
        #[cfg(feature = "testing")]
        contract.set_mock(Box::new(cw_multi_test::ContractWrapper::new_with_empty(
            ::os_factory::contract::execute,
            ::os_factory::contract::instantiate,
            ::os_factory::contract::query,
        )));
        Self(contract)
    }

    pub fn create_default_os(
        &self,
        governance_details: GovernanceDetails,
    ) -> Result<(), BootError> {
        let result = self.execute(
            &ExecuteMsg::CreateOs {
                governance: governance_details,
                description: None,
                link: None,
                name: "Test".to_string(),
            },
            None,
        )?;

        let manager_address = &result.event_attr_value("wasm", "manager_address")?;
        self.get_chain()
            .state()
            .set_address(MANAGER, &Addr::unchecked(manager_address));
        let treasury_address = &result.event_attr_value("wasm", "proxy_address")?;
        self.get_chain()
            .state()
            .set_address(PROXY, &Addr::unchecked(treasury_address));

        Ok(())
    }

    pub fn set_subscription_contract(&self, addr: String) -> Result<TxResponse<Chain>, BootError> {
        self.execute(
            &ExecuteMsg::UpdateConfig {
                admin: None,
                ans_host_contract: None,
                version_control_contract: None,
                module_factory_address: None,
                subscription_address: Some(addr),
            },
            None,
        )
    }
}
