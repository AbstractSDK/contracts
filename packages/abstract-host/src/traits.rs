use abstract_os::api::ApiRequestMsg;
use abstract_sdk::{
    api_request, manager::query_module_address, proxy::send_to_proxy, Dependency, MemoryOperation,
    OsExecute,
};
use cosmwasm_std::{Addr, CosmosMsg, Deps, Response, StdError, StdResult, Storage};
use serde::{de::DeserializeOwned, Serialize};

use crate::{HostContract, HostError};

impl<T: Serialize + DeserializeOwned> MemoryOperation for HostContract<'_, T> {
    fn load_memory(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}