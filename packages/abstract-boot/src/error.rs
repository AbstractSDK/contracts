use abstract_os::AbstractError;
use boot_core::BootError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbstractBootError {
    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Boot(#[from] BootError),

    #[error("JSON Conversion Error")]
    SerdeJson(#[from] ::serde_json::Error),

    #[error("{0}")]
    Std(#[from] StdError),
}
