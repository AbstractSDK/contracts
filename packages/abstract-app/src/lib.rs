// #[cfg(test)]
// mod mock_querier;
pub use crate::state::AppContract;
pub(crate) use abstract_sdk::base::*;
use cosmwasm_std::{Empty, Response};
pub use error::AppError;

mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
pub(crate) mod handler;
#[cfg(feature = "schema")]
mod schema;
pub mod state;

// #[cfg(test)]
// mod testing;
// Default to Empty
pub type AppResult<C = Empty> = Result<Response<C>, AppError>;

#[cfg(test)]
mod test_common {
    pub use abstract_os::app;
    pub use cosmwasm_std::testing::*;
    use cosmwasm_std::StdError;
    pub use speculoos::prelude::*;

    #[cosmwasm_schema::cw_serde]
    pub struct MockInitMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockExecMsg;

    impl app::AppExecuteMsg for MockExecMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct MockQueryMsg;

    impl app::AppQueryMsg for MockQueryMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct MockMigrateMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockReceiveMsg;

    use crate::{AppContract, AppError};
    use thiserror::Error;

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error("{0}")]
        DappError(#[from] AppError),
    }

    pub type MockAppContract = AppContract<
        // MockModule,
        MockError,
        MockExecMsg,
        MockInitMsg,
        MockQueryMsg,
        MockMigrateMsg,
        MockReceiveMsg,
    >;

    pub const MOCK_APP: MockAppContract = MockAppContract::new("test_contract", "0.1.0", None);
}
