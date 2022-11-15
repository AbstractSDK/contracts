use abstract_os::{
    objects::{
        module::{Module, ModuleInfo},
        module_reference::ModuleReference,
    },
    version_control::{state::MODULE_LIBRARY, ModuleResponse, QueryMsg},
};
use cosmwasm_std::{to_binary, Deps, QueryRequest, StdError, WasmQuery};

use cosmwasm_std::StdResult;

use super::RegisterAccess;

pub trait VersionRegisterInterface: RegisterAccess {
    fn version_register<'a>(&'a self, deps: Deps<'a>) -> VersionRegister<Self> {
        VersionRegister { base: self, deps }
    }
}

impl<T> VersionRegisterInterface for T where T: RegisterAccess {}

#[derive(Clone)]
pub struct VersionRegister<'a, T: VersionRegisterInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: VersionRegisterInterface> VersionRegister<'a, T> {
    pub fn get_module_reference_raw(&self, module_info: ModuleInfo) -> StdResult<ModuleReference> {
        let registry_addr = self.base.registry(self.deps)?;
        MODULE_LIBRARY
            .query(
                &self.deps.querier,
                registry_addr.clone(),
                module_info.clone(),
            )?
            .ok_or_else(|| {
                StdError::generic_err(format!(
                    "Module {} can not be found in registry {}.",
                    module_info, registry_addr
                ))
            })
    }
    /// Smart query
    pub fn get_module(&self, module_info: ModuleInfo) -> StdResult<Module> {
        let registry_addr = self.base.registry(self.deps)?;
        let resp: ModuleResponse =
            self.deps
                .querier
                .query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: registry_addr.into_string(),
                    msg: to_binary(&QueryMsg::Module {
                        module: module_info,
                    })?,
                }))?;
        Ok(resp.module)
    }
}