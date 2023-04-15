use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::{StdError, Uint128};
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ProxyError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error(transparent)]
    Admin(#[from] ::cw_controllers::AdminError),

    #[error("Module with address {0} is already allowlisted")]
    AlreadyAllowlisted(String),

    #[error("Module with address {0} not found in allowlist")]
    ModuleNotAllowlisted(String),

    #[error("Sender is not allowlisted")]
    SenderNotAllowlisted {},

    #[error("Max amount of assets registered")]
    AssetsLimitReached,

    #[error("Max amount of modules registered")]
    ModuleLimitReached,

    #[error("no base asset registered on proxy")]
    MissingBaseAsset,

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
