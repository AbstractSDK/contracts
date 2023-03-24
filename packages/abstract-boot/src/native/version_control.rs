use crate::{deployment, AbstractAccount};
pub use abstract_interface::version_control::{
    ExecuteMsgFns as VCExecFns, QueryMsgFns as VCQueryFns,
};
use abstract_interface::{
    objects::{
        module::{Module, ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
        AccountId,
    },
    version_control::*,
    VERSION_CONTROL,
};
#[cfg(feature = "daemon")]
use boot_core::Daemon;
use boot_core::{
    boot_contract, BootEnvironment, Contract, IndexResponse, TxResponse,
    {BootQuery, ContractInstance},
};
use cosmwasm_std::Addr;
use semver::Version;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct VersionControl<Chain>;

impl<Chain: BootEnvironment> VersionControl<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("version_control");
        Self(contract)
    }

    pub fn load(chain: Chain, address: &Addr) -> Self {
        Self(Contract::new(VERSION_CONTROL, chain).with_address(Some(address)))
    }

    /// Query a single module
    pub fn module(&self, info: ModuleInfo) -> Result<Module, crate::AbstractBootError> {
        let ModulesResponse { mut modules } = self.modules(vec![info])?;

        Ok(modules.swap_remove(0))
    }

    pub fn register_base(
        &self,
        account: &AbstractAccount<Chain>,
        version: &str,
    ) -> Result<(), crate::AbstractBootError> {
        let manager = account.manager.as_instance();
        let manager_module = (
            ModuleInfo::from_id(&manager.id, ModuleVersion::Version(version.to_string()))?,
            ModuleReference::Account(manager.code_id()?),
        );
        self.add_modules(vec![manager_module])?;

        log::info!("Module {} registered", manager.id);

        let proxy = account.proxy.as_instance();
        let proxy_module = (
            ModuleInfo::from_id(&proxy.id, ModuleVersion::Version(version.to_string()))?,
            ModuleReference::Account(proxy.code_id()?),
        );
        self.add_modules(vec![proxy_module])?;

        log::info!("Module {} registered", proxy.id);
        Ok(())
    }

    /// Register core modules
    pub fn register_cores(
        &self,
        apps: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractBootError> {
        let to_register = self.contracts_into_module_entries(apps, version, |c| {
            ModuleReference::Account(c.code_id().unwrap())
        })?;
        self.add_modules(to_register)?;
        Ok(())
    }

    /// Register native modules
    pub fn register_natives(
        &self,
        natives: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractBootError> {
        let to_register = self.contracts_into_module_entries(natives, version, |c| {
            ModuleReference::Native(c.address().unwrap())
        })?;
        self.add_modules(to_register)?;
        Ok(())
    }

    pub fn register_apps(
        &self,
        apps: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractBootError> {
        let to_register = self.contracts_into_module_entries(apps, version, |c| {
            ModuleReference::App(c.code_id().unwrap())
        })?;
        self.add_modules(to_register)?;
        Ok(())
    }

    pub fn register_apis(
        &self,
        apis: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractBootError> {
        let to_register = self.contracts_into_module_entries(apis, version, |c| {
            ModuleReference::Api(c.address().unwrap())
        })?;
        self.add_modules(to_register)?;
        Ok(())
    }

    pub fn register_standalones(
        &self,
        standalones: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractBootError> {
        let to_register = self.contracts_into_module_entries(standalones, version, |c| {
            ModuleReference::Standalone(c.code_id().unwrap())
        })?;
        self.add_modules(to_register)?;
        Ok(())
    }

    pub fn register_deployment(
        &self,
        deployment: &deployment::Abstract<Chain>,
    ) -> Result<(), crate::AbstractBootError> {
        self.register_natives(deployment.contracts(), &deployment.version)?;
        Ok(())
    }

    fn contracts_into_module_entries<RefFn>(
        &self,
        modules: Vec<&Contract<Chain>>,
        version: &Version,
        ref_fn: RefFn,
    ) -> Result<Vec<(ModuleInfo, ModuleReference)>, crate::AbstractBootError>
    where
        RefFn: Fn(&&Contract<Chain>) -> ModuleReference,
    {
        let modules_to_register: Result<
            Vec<(ModuleInfo, ModuleReference)>,
            crate::AbstractBootError,
        > = modules
            .iter()
            .map(|contract| {
                Ok((
                    ModuleInfo::from_id(&contract.id, ModuleVersion::Version(version.to_string()))?,
                    ref_fn(contract),
                ))
            })
            .collect();
        modules_to_register
    }

    pub fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<AccountBase, crate::AbstractBootError> {
        let resp: AccountBaseResponse = self.query(&QueryMsg::AccountBase { account_id })?;
        Ok(resp.account)
    }

    /// Retrieves an API's address from version control given the module **id** and **version**.
    pub fn get_api_addr(
        &self,
        id: &str,
        version: ModuleVersion,
    ) -> Result<Addr, crate::AbstractBootError> {
        let module: Module = self.module(ModuleInfo::from_id(id, version)?)?;

        Ok(module.reference.unwrap_api()?)
    }

    /// Retrieves an APP's code id from version control given the module **id** and **version**.
    pub fn get_app_code(
        &self,
        id: &str,
        version: ModuleVersion,
    ) -> Result<u64, crate::AbstractBootError> {
        let module: Module = self.module(ModuleInfo::from_id(id, version)?)?;

        Ok(module.reference.unwrap_app()?)
    }
}

#[cfg(feature = "daemon")]
impl VersionControl<Daemon> {
    // pub fn update_code_ids(&self, new_version: Version) -> anyhow::Result<()> {
    //     let code_ids = self.get_chain().state().get_all_code_ids()?;
    //     for (contract_id, code_id) in code_ids {
    //         if NATIVE_CONTRACTS.contains(&contract_id.as_str()) {
    //             continue;
    //         }

    //         // Get latest code id
    //         let resp: Result<QueryCodeIdResponse, crate::AbstractBootError> = self.query(&QueryMsg::CodeId {
    //             module: ModuleInfo {
    //                 name: contract_id.clone(),
    //                 version: None,
    //             },
    //         });
    //         log::debug!("{:?}", resp);
    //         if new_version.pre.is_empty() {
    //             match resp {
    //                 Ok(resp) => {
    //                     let registered_code_id = resp.code_id.u64();
    //                     // If equal, continue
    //                     if registered_code_id == code_id {
    //                         continue;
    //                     } else {
    //                         let latest_version = resp.info.version;
    //                         version = latest_version.parse().unwrap();
    //                         // bump patch
    //                         version.patch += 1;
    //                     }
    //                 }
    //                 Err(_) => (),
    //             };
    //         }

    //         self.execute(
    //             &ExecuteMsg::AddCodeId {
    //                 module: contract_id.to_string(),
    //                 version: version.to_string(),
    //                 code_id,
    //             },
    //             None,
    //         )?;
    //     }
    //     Ok(())
    // }

    // pub fn update_apis(&self) -> anyhow::Result<()> {
    //     for contract_name in chain_state.keys() {
    //         if !API_CONTRACTS.contains(&contract_name.as_str()) {
    //             continue;
    //         }

    //         // Get local addr
    //         let address: String = chain_state[contract_name].as_str().unwrap().into();

    //         // Get latest addr
    //         let resp: Result<QueryApiAddressResponse, crate::AbstractBootError> =
    //             self.query(&QueryMsg::ApiAddress {
    //                 module: ModuleInfo {
    //                     name: contract_name.clone(),
    //                     version: None,
    //                 },
    //             });
    //         log::debug!("{:?}", resp);
    //         let mut version = self.deployment_version.clone();
    //         match resp {
    //             Ok(resp) => {
    //                 let registered_addr = resp.address.to_string();

    //                 // If equal, continue
    //                 if registered_addr == address {
    //                     continue;
    //                 } else {
    //                     let latest_version = resp.info.version;
    //                     version = latest_version.parse().unwrap();
    //                     // bump patch
    //                     version.patch += 1;
    //                 }
    //             }
    //             Err(_) => (),
    //         };

    //         self.execute(
    //             &ExecuteMsg::AddApi {
    //                 module: contract_name.to_string(),
    //                 version: version.to_string(),
    //                 address,
    //             },
    //             None,
    //         )?;
    //     }
    //     Ok(())
    // }
}
