use abstract_os::objects::{memory::Memory, memory_traits::Resolve};
use cosmwasm_std::{CosmosMsg, Deps, Response, StdResult, Storage};

/// execute an operation on the os
pub trait OsExecute {
    type Err: ToString;

    fn os_execute(&self, deps: Deps, msgs: Vec<CosmosMsg>) -> Result<Response, Self::Err>;
}

// easily retrieve the memory object from the contract to perform queries
pub trait MemoryOperation {
    fn load(&self, store: &dyn Storage) -> StdResult<Memory>;
    fn resolve<T: Resolve>(
        &self,
        deps: Deps,
        memory_entry: &dyn Resolve<Output = T::Output>,
    ) -> StdResult<T::Output> {
        memory_entry.resolve(deps, &self.load(deps.storage)?)
    }
}
