use abstract_sdk::SdkError;
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Abstract(#[from] SdkError),

    #[error("{0}")]
    Admin(#[from] AdminError),
}
