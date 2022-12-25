use crate::apis::{AbstractNameService, Identification};
pub use cosmwasm_std::testing::*;
pub use cosmwasm_std::*;
use os::objects::ans_host::AnsHost;
pub use speculoos::prelude::*;

pub struct MockModule {}

pub const TEST_PROXY: &str = "proxy_address";

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

pub const fn stub_module() -> MockModule {
    MockModule {}
}
