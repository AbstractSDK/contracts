use crate::{AbstractAccount, Manager, Proxy};
pub use abstract_core::account_factory::{
    ExecuteMsgFns as AccountFactoryExecFns, QueryMsgFns as AccountFactoryQueryFns,
};
use abstract_core::{
    account_factory::*, objects::gov_type::GovernanceDetails, ABSTRACT_EVENT_NAME, MANAGER, PROXY,
};
use cosmwasm_std::Addr;
use cw_orch::{
    contract, ArtifactsDir, Contract, CwEnv, IndexResponse, StateInterface,
    {ContractInstance, CwOrcExecute},
};

/// A helper struct that contains fields from [`abstract_core::manager::state::AccountInfo`]
#[derive(Default)]
pub struct AccountDetails {
    name: String,
    description: Option<String>,
    link: Option<String>,
}

#[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct AccountFactory;

impl<Chain: CwEnv> ::cw_orch::Uploadable for AccountFactory<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper(&self) -> <::cw_orch::Mock as ::cw_orch::TxHandler>::ContractSource {
        Box::new(
            cw_orch::ContractWrapper::new_with_empty(
                ::account_factory::contract::execute,
                ::account_factory::contract::instantiate,
                ::account_factory::contract::query,
            )
            .with_reply_empty(::account_factory::contract::reply)
            .with_migrate(::account_factory::contract::migrate),
        )
    }

    fn wasm(&self) -> cw_orch::WasmPath {
        ArtifactsDir::env()
            .find_wasm_path("account_factor")
            .unwrap()
    }
}

impl<Chain: CwEnv> AccountFactory<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
    /// Creates a local account
    pub fn create_new_account(
        &self,
        account_details: AccountDetails,
        governance_details: GovernanceDetails<String>,
    ) -> Result<AbstractAccount<Chain>, crate::AbstractBootError> {
        let AccountDetails {
            name,
            link,
            description,
        } = account_details;

        let result = self.execute(
            &ExecuteMsg::CreateAccount {
                governance: governance_details,
                name,
                link,
                description,
                origin: None,
            },
            None,
        )?;

        let manager_address = &result.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
        self.get_chain()
            .state()
            .set_address(MANAGER, &Addr::unchecked(manager_address));
        let proxy_address = &result.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;
        self.get_chain()
            .state()
            .set_address(PROXY, &Addr::unchecked(proxy_address));
        Ok(AbstractAccount {
            manager: Manager::new(MANAGER, self.get_chain().clone()),
            proxy: Proxy::new(PROXY, self.get_chain().clone()),
        })
    }

    pub fn create_default_account(
        &self,
        governance_details: GovernanceDetails<String>,
    ) -> Result<AbstractAccount<Chain>, crate::AbstractBootError> {
        self.create_new_account(
            AccountDetails {
                name: "Default Abstract Account".into(),
                ..Default::default()
            },
            governance_details,
        )
    }
}
