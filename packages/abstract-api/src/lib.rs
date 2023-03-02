//! # Abstract api
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};

pub type ApiResult<C = Empty> = Result<Response<C>, ApiError>;
// Default to Empty

pub use crate::state::ApiContract;
pub use error::ApiError;

pub mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
mod handler;
#[cfg(feature = "schema")]
mod schema;
pub mod state;

#[cfg(test)]
mod test_common {
    use crate::{ApiContract, ApiError};
    use abstract_os::api::{self, BaseInstantiateMsg, InstantiateMsg};
    use abstract_sdk::base::InstantiateEndpoint;
    use abstract_sdk::AbstractSdkError;
    use abstract_testing::{
        TEST_ADMIN, TEST_ANS_HOST, TEST_MODULE_ID, TEST_VERSION, TEST_VERSION_CONTROL,
    };
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        DepsMut, Empty, Env, MessageInfo, Response, StdError,
    };
    use thiserror::Error;

    pub const TEST_METADATA: &str = "test_metadata";
    pub const TEST_TRADER: &str = "test_trader";

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error(transparent)]
        Api(#[from] ApiError),

        #[error("{0}")]
        Abstract(#[from] abstract_os::AbstractOsError),

        #[error("{0}")]
        AbstractSdk(#[from] AbstractSdkError),
    }

    #[cosmwasm_schema::cw_serde]
    pub struct MockApiExecMsg;

    impl api::ApiExecuteMsg for MockApiExecMsg {}

    /// Mock API type
    pub type MockApi = ApiContract<MockError, Empty, MockApiExecMsg, Empty>;

    /// use for testing
    pub const MOCK_API: MockApi = MockApi::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
        .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_response".as_bytes())))
        .with_instantiate(mock_init_handler);

    pub type ApiMockResult = Result<(), MockError>;

    pub fn mock_init(deps: DepsMut) -> Result<Response, MockError> {
        let api = MOCK_API;
        let info = mock_info(TEST_ADMIN, &[]);
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: TEST_VERSION_CONTROL.into(),
            },
            app: Empty {},
        };
        api.instantiate(deps, mock_env(), info, init_msg)
    }

    fn mock_init_handler(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _api: MockApi,
        _msg: Empty,
    ) -> Result<Response, MockError> {
        Ok(Response::new().set_data("mock_response".as_bytes()))
    }
}
