//! # Abstract Api
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};

pub use error::HostError;

pub use crate::state::Host;
pub mod chains;
pub mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
mod handler;
pub(crate) mod host_commands;
pub mod os_commands;
mod schema;
pub mod state;

// Default to Empty
pub type IbcHostResult<C = Empty> = Result<Response<C>, HostError>;

#[cfg(test)]
mod test_common {
    use crate::{Host, HostError};
    use abstract_os::ibc_host;
    use abstract_sdk::base::InstantiateEndpoint;
    use abstract_testing::{
        MockDeps, TEST_ANS_HOST, TEST_CHAIN, TEST_MODULE_FACTORY, TEST_MODULE_ID, TEST_VERSION,
    };
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, Empty, StdError};
    pub use speculoos::prelude::*;
    use thiserror::Error;

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error("{0}")]
        HostError(#[from] HostError),
    }

    pub type MockIbcHost = Host<MockError>;

    pub const MOCK_HOST: MockIbcHost = new_mock_host();

    pub const fn new_mock_host() -> MockIbcHost {
        MockIbcHost::new(TEST_MODULE_ID, TEST_VERSION, TEST_CHAIN, None)
    }

    pub fn mock_init() -> MockDeps {
        let mut deps = mock_dependencies();
        let info = mock_info(TEST_MODULE_FACTORY, &[]);

        let init_msg = ibc_host::InstantiateMsg {
            base: ibc_host::BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
                cw1_code_id: 1,
            },
            app: Empty {},
        };

        MOCK_HOST
            .instantiate(deps.as_mut(), mock_env(), info, init_msg)
            .unwrap();

        deps
    }
}
