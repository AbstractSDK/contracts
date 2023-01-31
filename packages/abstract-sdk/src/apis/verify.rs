//! # Verification
//! The `Verify` struct provides helper functions that enable the contract to verify if the sender is an OS, OS admin, etc.
use super::AbstractRegistryAccess;
use abstract_os::{
    manager::state::OS_ID,
    version_control::{state::OS_ADDRESSES, Core},
};
use cosmwasm_std::{Addr, Deps, StdError, StdResult};

/// A trait enabling the verification of addresses associated with an OS.
pub trait OsVerification: AbstractRegistryAccess {
    fn os_registry<'a>(&'a self, deps: Deps<'a>) -> OsRegistry<Self> {
        OsRegistry { base: self, deps }
    }
}

impl<T> OsVerification for T where T: AbstractRegistryAccess {}

/// Endpoint for OS address verification
#[derive(Clone)]
pub struct OsRegistry<'a, T: OsVerification> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: OsVerification> OsRegistry<'a, T> {
    /// Verify if the provided manager address is indeed a user.
    pub fn assert_manager(&self, maybe_manager: &Addr) -> StdResult<Core> {
        let os_id = OS_ID
            .query(&self.deps.querier, maybe_manager.clone())
            .map_err(|_| StdError::generic_err("Caller must be an OS manager."))?;
        let vc_address = self.base.abstract_registry(self.deps)?;
        let maybe_os = OS_ADDRESSES.query(&self.deps.querier, vc_address, os_id)?;
        match maybe_os {
            None => Err(StdError::generic_err(format!(
                "OS with id {os_id} is not active."
            ))),
            Some(core) => {
                if &core.manager != maybe_manager {
                    Err(StdError::generic_err(
                        "Proposed manager is not the manager of this OS.",
                    ))
                } else {
                    Ok(core)
                }
            }
        }
    }

    /// Verify if the provided proxy address is indeed a user.
    pub fn assert_proxy(&self, maybe_proxy: &Addr) -> StdResult<Core> {
        let os_id = OS_ID
            .query(&self.deps.querier, maybe_proxy.clone())
            .map_err(|_| StdError::generic_err("Caller must be an OS proxy."))?;

        let vc_address = self.base.abstract_registry(self.deps)?;
        let maybe_os = OS_ADDRESSES.query(&self.deps.querier, vc_address, os_id)?;
        match maybe_os {
            None => Err(StdError::generic_err(format!(
                "OS with id {os_id} is not active."
            ))),
            Some(core) => {
                if &core.proxy != maybe_proxy {
                    Err(StdError::generic_err(
                        "Proposed proxy is not the proxy of this OS.",
                    ))
                } else {
                    Ok(core)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::*;

    use crate::ModuleRegistryInterface;
    use abstract_testing::{
        mock_querier, MockQuerierBuilder, TEST_OS_ID, TEST_PROXY, TEST_VERSION_CONTROL,
    };
    use speculoos::prelude::*;

    struct MockRegistry;

    impl AbstractRegistryAccess for MockRegistry {
        fn abstract_registry(&self, _deps: Deps) -> StdResult<Addr> {
            Ok(Addr::unchecked(TEST_VERSION_CONTROL))
        }
    }

    mod assert_manager {
        use super::*;

        #[test]
        fn not_proxy_fails() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            let registry = MockRegistry;

            let res = registry
                .os_registry(deps.as_ref())
                .assert_proxy(&Addr::unchecked("not_proxy"));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, StdError::GenericErr { .. }))
                .matches(|e| e.to_string().contains("OS proxy"));
        }

        #[test]
        fn inactive_os_fails() {
            let mut deps = mock_dependencies();
            deps.querier = MockQuerierBuilder::default()
                // .with_raw_handler(TEST_VERSION_CONTROL, |msg| {})
                .with_contract_item(TEST_PROXY, OS_ID, &TEST_OS_ID)
                .build();

            let registry = MockRegistry;

            let res = registry
                .os_registry(deps.as_ref())
                .assert_proxy(&Addr::unchecked(TEST_PROXY));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, StdError::GenericErr { .. }))
                .matches(|e| e.to_string().contains("OS with id 0 is not active"));
        }
    }
}
