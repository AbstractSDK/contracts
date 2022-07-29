use abstract_api::ApiError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DexError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ApiError(#[from] ApiError),

    #[error("DEX {0} is not a known dex on this network.")]
    UnknownDex(String),

    #[error("Cw1155 is unsupported.")]
    Cw1155Unsupported,
}
