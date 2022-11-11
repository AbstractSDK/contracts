use abstract_os::{
    manager::state::OS_ID,
    objects::module::{Module, ModuleInfo},
    version_control::{state::OS_ADDRESSES, Core, ModuleResponse, QueryMsg},
};
use cosmwasm_std::{Addr, QuerierWrapper, StdError};

use cosmwasm_std::StdResult;

use crate::os_address::OsAddress;


pub trait Verification<'a>: OsAddress + Sized {
    fn verify(&self,querier: &QuerierWrapper, version_control_address: &Addr)-> Verify<'a> {
        Verify { base: self, version_control_address }
    }
}

impl<'a, T> Verification<'a> for T
    where T: OsAddress + Sized
{}

pub struct Verify <'a> {
    base: &'a dyn OsAddress,
    version_control_address: &'a Addr,
}

impl<'a> Verify<'a> {
    
/// Verify if the provided manager address is indeed a user.
pub fn assert_os_manager(
    &self,
    querier: &QuerierWrapper,
    maybe_manager: &Addr,
) -> StdResult<Core> {
    let os_id = OS_ID.query(querier, maybe_manager.clone())?;
    let maybe_os = OS_ADDRESSES.query(querier, self.version_control_address.clone(), os_id)?;
    match maybe_os {
        None => Err(StdError::generic_err(format!(
            "OS with id {} is not active.",
            os_id
        ))),
        Some(core) => {
            if &core.manager != maybe_manager {
                Err(StdError::generic_err(
                    "Proposed manager is not the manager of this instance.",
                ))
            } else {
                Ok(core)
            }
        }
    }
}

/// Verify if the provided proxy address is indeed a user.
pub fn assert_os_proxy(
    &self,
    querier: &QuerierWrapper,
    maybe_proxy: &Addr,
) -> StdResult<Core> {
    let os_id = OS_ID.query(querier, maybe_proxy.clone())?;
    let maybe_os = OS_ADDRESSES.query(querier, self.version_control_address.clone(), os_id)?;
    match maybe_os {
        None => Err(StdError::generic_err(format!(
            "OS with id {} is not active.",
            os_id
        ))),
        Some(core) => {
            if &core.proxy != maybe_proxy {
                Err(StdError::generic_err(
                    "Proposed proxy is not the proxy of this instance.",
                ))
            } else {
                Ok(core)
            }
        }
    }
}
}


/// Get the [`abstract_os::version_control::Core`] object for an os-id.
pub fn get_os_core(
    querier: &QuerierWrapper,
    os_id: u32,
    version_control_addr: &Addr,
) -> StdResult<Core> {
    let maybe_os = OS_ADDRESSES.query(querier, version_control_addr.clone(), os_id)?;
    match maybe_os {
        None => Err(StdError::generic_err(format!(
            "OS with id {} is not active.",
            os_id
        ))),
        Some(core) => Ok(core),
    }
}

