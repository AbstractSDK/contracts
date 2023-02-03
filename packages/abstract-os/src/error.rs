use cosmwasm_std::{StdError};
use thiserror::Error;
use cw_asset::AssetError;

#[derive(Error, Debug, PartialEq)]
pub enum AbstractError {
    #[error("Std error encountered while handling os object: {0}")]
    Std(#[from] StdError),

    #[error("Asset error encountered while handling assets: {0}")]
    Overflow(#[from] AssetError),
}