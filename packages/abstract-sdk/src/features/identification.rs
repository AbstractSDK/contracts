use abstract_os::{objects::common_namespace::ADMIN_NAMESPACE, version_control::Core};
use cosmwasm_std::{Addr, StdError, StdResult};
use cw_storage_plus::Item;

use super::contract_deps::ContractDeps;

const MANAGER: Item<'_, Option<Addr>> = Item::new(ADMIN_NAMESPACE);

pub trait Identification: ContractDeps {
    fn proxy_address(&self) -> StdResult<Addr>;
    fn manager_address(&self) -> StdResult<Addr> {
        let maybe_proxy_manager = MANAGER.query(self.querier(), self.proxy_address()?)?;
        maybe_proxy_manager.ok_or_else(|| StdError::generic_err("proxy admin must be manager."))
    }
    fn os_core(&self) -> StdResult<Core> {
        Ok(Core {
            manager: self.manager_address()?,
            proxy: self.proxy_address()?,
        })
    }
}
