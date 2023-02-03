use crate::SdkResult;
use cosmwasm_std::{Addr, Deps};

/// Trait that enables access to a registry, like a version control contract.
pub trait AbstractRegistryAccess: Sized {
    /// Get the address of the registry.
    fn abstract_registry(&self, deps: Deps) -> SdkResult<Addr>;
}
