use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VersionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error(
        "Version {} of module {} does not have a stored code id",
        version.unwrap_or_default(),
        module
    )]
    MissingCodeId { version: Option<String>, module: String },

    #[error("OS ID {} is not in version control register", id)]
    MissingOsId { id: u32 },
}
