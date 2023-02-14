use abstract_os::AbstractError;
use abstract_sdk::SdkError;
use cosmwasm_std::{StdError, Uint128};
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] SdkError),

    #[error("Asset error encountered while handling assets: {0}")]
    CwAsset(#[from] AssetError),

    #[error(transparent)]
    Admin(#[from] ::cw_controllers::AdminError),

    #[error("Module with address {0} is already whitelisted")]
    AlreadyWhitelisted(String),

    #[error("Module with address {0} not found in whitelist")]
    NotWhitelisted(String),

    #[error("Sender is not whitelisted")]
    SenderNotWhitelisted {},

    #[error("Max amount of assets registered")]
    AssetsLimitReached,

    #[error("Max amount of modules registered")]
    ModuleLimitReached,

    #[error("The proposed update resulted in a bad configuration: {0}")]
    BadUpdate(String),

    #[error(
        "Treasury balance too low, {} requested but it only has {}",
        requested,
        balance
    )]
    Broke {
        balance: Uint128,
        requested: Uint128,
    },
}

impl From<ProxyError> for StdError {
    fn from(e: ProxyError) -> Self {
        StdError::generic_err(e.to_string())
    }
}
