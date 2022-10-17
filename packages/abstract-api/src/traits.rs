use abstract_os::{api::ApiRequestMsg, objects::{ContractEntry, UncheckedContractEntry}, IBC_CLIENT};
use abstract_sdk::{
    api_request, manager::query_module_address, proxy::{os_module_action, os_ibc_action}, Dependency, MemoryOperation,
    OsExecute,
};
use cosmwasm_std::{Addr, CosmosMsg, Deps, Response, StdError, StdResult, Storage, MessageInfo, Binary};
use serde::{de::DeserializeOwned, Serialize};

use crate::{ApiContract, ApiError};

impl<T: Serialize + DeserializeOwned, C: Serialize + DeserializeOwned> MemoryOperation for ApiContract<'_, T, C> {
    fn load_memory(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}

/// Execute a set of CosmosMsgs on the proxy contract of an OS.
impl<T: Serialize + DeserializeOwned, C: Serialize + DeserializeOwned> OsExecute for ApiContract<'_, T, C> {
    type Err = ApiError;

    fn os_execute(
        &self,
        _deps: Deps,
        msgs: Vec<cosmwasm_std::CosmosMsg>,
    ) -> Result<Response, Self::Err> {
        if let Some(target) = &self.target_os {
            Ok(Response::new().add_message(os_module_action(msgs, &target.proxy)?))
        } else {
            Err(ApiError::NoTargetOS {})
        }
    }
    fn os_ibc_execute(
        &self,
        _deps: Deps,
        msgs: Vec<abstract_os::ibc_client::ExecuteMsg>,
    ) -> Result<Response, Self::Err> {
        if let Some(target) = &self.target_os {
            Ok(Response::new().add_message(os_ibc_action(msgs, &target.proxy)?))
        } else {
            Err(ApiError::NoTargetOS {})
        }
    }
}

/// Implement the dependency functions for an API contract
impl<T: Serialize + DeserializeOwned, C: Serialize + DeserializeOwned> Dependency for ApiContract<'_, T, C> {
    fn dependency_address(
        &self,
        deps: Deps,
        dependency_name: &str,
    ) -> cosmwasm_std::StdResult<Addr> {
        if !self.dependencies.contains(&dependency_name) {
            return Err(StdError::generic_err("dependency not enabled on OS"));
        }
        let manager_addr = &self
            .target_os
            .as_ref()
            .ok_or_else(|| StdError::generic_err(ApiError::NoTargetOS {}.to_string()))?
            .manager;
        query_module_address(deps, manager_addr, dependency_name)
    }

    fn call_api_dependency<E: Serialize>(
        &self,
        deps: Deps,
        dependency_name: &str,
        request_msg: &E,
        funds: Vec<cosmwasm_std::Coin>,
    ) -> cosmwasm_std::StdResult<CosmosMsg> {
        let proxy = self
            .target()
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        let dep_addr = self.dependency_address(deps, dependency_name)?;
        let api_request_msg = ApiRequestMsg {
            proxy_address: Some(proxy.to_string()),
            request: request_msg,
        };
        api_request(dep_addr, api_request_msg, funds)
    }
}
