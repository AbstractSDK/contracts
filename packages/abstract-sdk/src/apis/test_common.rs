use crate::apis::{AbstractNameService, Dependencies, Identification};
pub use cosmwasm_std::testing::*;
pub use cosmwasm_std::*;
use os::objects::ans_host::AnsHost;
use os::objects::dependency::StaticDependency;
use os::{api, app};
pub use speculoos::prelude::*;

pub struct MockModule {}

pub const TEST_PROXY: &str = "proxy_address";
pub const TEST_MANAGER: &str = "manager_address";

impl Identification for MockModule {
    fn proxy_address(&self, _deps: Deps) -> Result<Addr, StdError> {
        Ok(Addr::unchecked(TEST_PROXY))
    }
}

impl AbstractNameService for MockModule {
    fn ans_host(&self, _deps: Deps) -> StdResult<AnsHost> {
        Ok(AnsHost {
            address: Addr::unchecked("ans"),
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

pub const fn mock_module() -> MockModule {
    MockModule {}
}
