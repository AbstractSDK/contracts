use crate::{Host, HostError};
use abstract_sdk::base::features::{AbstractNameService, Identification, ModuleIdentification};
use cosmwasm_std::{Deps, StdError, StdResult};

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > AbstractNameService
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn ans_host(&self, deps: Deps) -> StdResult<abstract_sdk::feature_objects::AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Identification
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn proxy_address(&self, _deps: Deps) -> StdResult<cosmwasm_std::Addr> {
        self.target()
            .map_err(|e| StdError::generic_err(e.to_string()))
            .map(ToOwned::to_owned)
    }
    fn manager_address(&self, _deps: Deps) -> StdResult<cosmwasm_std::Addr> {
        Err(StdError::generic_err(
            "manager address not available on stateless ibc deployment",
        ))
    }

    fn os_core(&self, _deps: Deps) -> StdResult<abstract_sdk::os::version_control::Core> {
        Err(StdError::generic_err(
            "OS core not available on stateless ibc deployment",
        ))
    }

    fn os_id(&self, _deps: Deps) -> StdResult<u32> {
        Err(StdError::generic_err(
            "os_id not available on stateless ibc deployment",
        ))
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > ModuleIdentification
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn module_id(&self) -> &'static str {
        self.contract.info().0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_common::*;
    use abstract_testing::{TEST_ANS_HOST, TEST_CHAIN, TEST_MODULE_ID, TEST_PROXY, TEST_VERSION};
    use cosmwasm_std::Addr;

    #[test]
    fn test_ans_host() {
        let mut deps = mock_init();

        let ans_host = MOCK_HOST.ans_host(deps.as_ref());

        assert_that!(ans_host)
            .is_ok()
            .map(|a| &a.address)
            .is_equal_to(Addr::unchecked(TEST_ANS_HOST));
    }

    #[test]
    fn test_proxy_address_no_target() {
        let mut deps = mock_init();

        let proxy_address = MOCK_HOST.proxy_address(deps.as_ref());

        assert_that!(proxy_address)
            .is_err()
            .matches(|e| matches!(e, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_proxy_address() {
        let mut deps = mock_init();

        let mut host = new_mock_host();
        host.proxy_address = Some(Addr::unchecked(TEST_PROXY));
        assert_that!(host.proxy_address)
            .is_some()
            .is_equal_to(Addr::unchecked(TEST_PROXY));

        let proxy_address = host.proxy_address(deps.as_ref());

        assert_that!(proxy_address)
            .is_ok()
            .is_equal_to(Addr::unchecked(TEST_PROXY));
    }

    #[test]
    fn test_no_manager_address() {
        let mut deps = mock_init();

        let manager_address = MOCK_HOST.manager_address(deps.as_ref());

        assert_that!(manager_address)
            .is_err()
            .matches(|e| matches!(e, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_no_os_core() {
        let mut deps = mock_init();

        let os_core = MOCK_HOST.os_core(deps.as_ref());

        assert_that!(os_core)
            .is_err()
            .matches(|e| matches!(e, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_no_os_id() {
        let mut deps = mock_init();

        let os_id = MOCK_HOST.os_id(deps.as_ref());

        assert_that!(os_id)
            .is_err()
            .matches(|e| matches!(e, StdError::GenericErr { .. }));
    }
}
