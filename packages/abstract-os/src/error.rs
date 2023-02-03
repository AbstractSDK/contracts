use cosmwasm_std::{OverflowError, StdError};
use cw_asset::AssetError;
use cw_semver::Error as CwSemverError;
use semver::Error as SemverError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbstractError {
    #[error("Std error encountered while handling os object: {0}")]
    Std(#[from] StdError),

    #[error("Asset error encountered while handling assets: {0}")]
    CwAsset(#[from] AssetError),

    #[error("cw math overflow error: {0}")]
    Overflow(#[from] OverflowError),

    #[error("Semver error encountered while handling os object: {0}")]
    Semver(#[from] SemverError),

    #[error("Semver error encountered while handling os object: {0}")]
    CwSemver(#[from] CwSemverError),

    #[error("Entry {actual} should be formatted as {expected}")]
    EntryFormattingError { actual: String, expected: String },

    #[error("Object {object} should be formatted {expected} but is {actual}")]
    FormattingError {
        object: String,
        expected: String,
        actual: String,
    },

    #[error("API {0} not installed on OS")]
    ApiNotInstalled(String),

    #[error("App {0} not installed on OS")]
    AppNotInstalled(String),

    #[error("version for {0} in missing")]
    MissingVersion(String),

    #[error("Abstract storage object {object} errors with {msg}")]
    Storage { object: String, msg: String },

    #[error("assertion: {0}")]
    Assert(String),

    //fee error
    #[error("fee error: {0}")]
    Fee(String),

    // deposit error
    #[error("deposit error: {0}")]
    Deposit(String),
}
