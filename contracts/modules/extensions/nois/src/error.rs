use abstract_extension::ExtensionError;
use cosmwasm_std::{DecimalRangeExceeded, OverflowError, StdError};
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum NoisError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AdminError(#[from] AdminError),

    #[error("{0}")]
    DecimalError(#[from] DecimalRangeExceeded),

    #[error("{0}")]
    ExtensionError(#[from] ExtensionError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Proxy address is not valid")]
    InvalidProxyAddress,

    #[error("Round already present")]
    JobIdAlreadyPresent,

    //callback should only be allowed to be called by the proxy contract
    //otherwise anyone can cut the randomness workflow and cheat the randomness
    #[error("Unauthorized Receive execution")]
    UnauthorizedReceive,

    #[error("Received invalid randomness")]
    InvalidRandomness,
}
