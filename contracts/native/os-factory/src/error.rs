use abstract_os::AbstractError;
use abstract_sdk::SdkError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OsFactoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] SdkError),

    #[error("Asset error encountered while handling assets: {0}")]
    CwAsset(#[from] AssetError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),

    #[error("module {0} is required to be of kind {1}")]
    WrongModuleKind(String, String),

    #[error("Bad subscription module configuration.")]
    UnsupportedAsset(),

    #[error("Your payment does not match the required payment {0}")]
    WrongAmount(String),

    #[error("No payment received")]
    NoPaymentReceived {},
}
