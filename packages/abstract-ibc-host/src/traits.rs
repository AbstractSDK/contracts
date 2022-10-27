use abstract_sdk::{MemoryOperation, OsExecute, os_module_action};
use cosmwasm_std::{StdResult, Storage, Deps, Response};
use serde::{de::DeserializeOwned, Serialize};

use crate::{Host, HostError};

impl<T: Serialize + DeserializeOwned> MemoryOperation for Host<'_, T> {
    fn load_memory(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}

/// Execute a set of CosmosMsgs on the proxy contract of an OS.
impl<T: Serialize + DeserializeOwned> OsExecute for Host<'_, T> {
    type Err = HostError;

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
