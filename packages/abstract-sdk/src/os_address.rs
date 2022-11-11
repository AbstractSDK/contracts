use abstract_os::{version_control::Core, objects::common_namespace::ADMIN_NAMESPACE};
use cosmwasm_std::{Addr, Deps, StdResult, StdError};
use cw_storage_plus::Item;

const MANAGER: Item<'_, Option<Addr>> = Item::new(ADMIN_NAMESPACE);

pub trait AnsClient {
    
}

/// Retrieve addresses related to the OS from the module
pub trait OsAddress {
    fn proxy_address(&self, deps: Deps) -> StdResult<Addr>;
    fn manager_address(&self, deps: Deps) -> StdResult<Addr> {
        let maybe_proxy_manager = MANAGER.query(&deps.querier, self.proxy_address(deps)?)?;
        maybe_proxy_manager.ok_or_else(|| StdError::generic_err("proxy admin must be manager."))
    }
    fn os_core(&self, deps: Deps) -> StdResult<Core> {
        Ok(Core{
                    manager: self.manager_address(deps)?,
                    proxy: self.proxy_address(deps)?,
                })
    }
}

pub trait Versioning {
    fn version_control_address(&self, deps: Deps) -> StdResult<Addr>;
    fn manager_address(&self, deps: Deps) -> StdResult<Addr> {
        let maybe_proxy_manager = MANAGER.query(&deps.querier, self.proxy_address(deps)?)?;
        maybe_proxy_manager.ok_or_else(|| StdError::generic_err("proxy admin must be manager."))
    }
    fn os_core(&self, deps: Deps) -> StdResult<Core> {
        Ok(Core{
                    manager: self.manager_address(deps)?,
                    proxy: self.proxy_address(deps)?,
                })
    }
}