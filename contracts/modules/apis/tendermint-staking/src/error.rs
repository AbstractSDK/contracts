use abstract_api::ApiError;
use abstract_sdk::SdkError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TendermintStakeError {
    #[error("{0}")]
    Std(#[from] StdError),
    //
    // #[error("{0}")]
    // Abstract(#[from] AbstractError),
    #[error("{0}")]
    AbstractSdk(#[from] SdkError),

    #[error("{0}")]
    ApiError(#[from] ApiError),
}
