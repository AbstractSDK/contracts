use abstract_sdk::MemoryOperation;
use cosmwasm_std::{StdResult, Storage};
use serde::{de::DeserializeOwned, Serialize};

use crate::HostContract;

impl<T: Serialize + DeserializeOwned> MemoryOperation for HostContract<'_, T> {
    fn load_memory(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}
