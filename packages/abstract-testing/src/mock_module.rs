use crate::{
    TEST_ANS_HOST, TEST_MANAGER, TEST_MODULE_ADDRESS, TEST_MODULE_ID, TEST_PROXY,
    TEST_VERSION_CONTROL,
};
use abstract_os::objects::ans_host::AnsHost;
use abstract_os::version_control::Core;
use abstract_os::{api, app};
use abstract_sdk::base::features::{AbstractNameService, Identification};
use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    to_binary, Addr, Binary, ContractResult, Deps, Empty, QuerierWrapper, StdError, StdResult,
    SystemResult, WasmQuery,
};
use std::collections::HashMap;

pub struct MockModule {}

impl MockModule {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Identification for MockModule {
    fn proxy_address(&self, _deps: Deps) -> Result<Addr, StdError> {
        Ok(Addr::unchecked(TEST_PROXY))
    }
}

impl AbstractNameService for MockModule {
    fn ans_host(&self, _deps: Deps) -> StdResult<AnsHost> {
        Ok(AnsHost {
            address: Addr::unchecked(TEST_ANS_HOST),
        })
    }
}

#[cosmwasm_schema::cw_serde]
pub struct MockModuleExecuteMsg {}

#[cosmwasm_schema::cw_serde]
pub struct MockModuleQueryMsg {}

impl api::ApiExecuteMsg for MockModuleExecuteMsg {}

impl api::ApiQueryMsg for MockModuleQueryMsg {}

impl app::AppExecuteMsg for MockModuleExecuteMsg {}

impl app::AppQueryMsg for MockModuleQueryMsg {}
