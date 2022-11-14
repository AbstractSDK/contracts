//! # Application
//! The Application interface provides helper functions to execute functions on other applications installed on the OS.

use abstract_os::{
    api::{BaseExecuteMsg, ExecuteMsg},
    manager::state::{ModuleId, OS_MODULES},
};
use cosmwasm_std::{
    wasm_execute, Addr, CosmosMsg, Empty, QueryRequest, StdError, StdResult, WasmQuery,
};
use cw2::{ContractVersion, CONTRACT};
use serde::Serialize;

use crate::features::Identification;

pub trait ApplicationInterface: Identification {
    fn applications(&self) -> Applications<Self> {
        Applications { base: self }
    }
}

impl<T> ApplicationInterface for T where T: Identification {}

pub struct Applications<'a, T: ApplicationInterface> {
    base: &'a T,
}

impl<'a, T: ApplicationInterface> Applications<'a, T> {
    pub fn app_address(&self, module_id: ModuleId) -> StdResult<Addr> {
        let manager_addr = self.base.manager_address()?;
        let maybe_module_addr = OS_MODULES.query(&self.base.querier(), manager_addr, module_id)?;
        let Some(module_addr) = maybe_module_addr else {
            return Err(StdError::generic_err(format!("Module {} not enabled on OS.",module_id)));
        };
        Ok(module_addr)
    }

    /// Construct an API request message.
    pub fn api_request<M: Serialize>(
        &self,
        api_id: ModuleId,
        message: impl Into<ExecuteMsg<M, Empty>>,
    ) -> StdResult<CosmosMsg> {
        let api_msg: ExecuteMsg<M, Empty> = message.into();
        let api_address = self.app_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// Construct an API configure message
    pub fn configure_api(&self, api_id: ModuleId, message: BaseExecuteMsg) -> StdResult<CosmosMsg> {
        let api_msg: ExecuteMsg<Empty, Empty> = message.into();
        let api_address = self.app_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// RawQuery the version of an enabled module
    pub fn app_version(&self, app_id: ModuleId) -> StdResult<ContractVersion> {
        let app_address = self.app_address(app_id)?;
        let req = QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: app_address.into(),
            key: CONTRACT.as_slice().into(),
        });
        self.base.querier().query::<ContractVersion>(&req)
    }
}
