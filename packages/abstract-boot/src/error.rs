use abstract_core::AbstractError;
use cosmwasm_std::StdError;
use cw_orch::CwOrcError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbstractBootError {
    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Orch(#[from] CwOrcError),

    #[error("JSON Conversion Error")]
    SerdeJson(#[from] ::serde_json::Error),

    #[error("{0}")]
    Std(#[from] StdError),
}

impl AbstractBootError {
    pub fn root(&self) -> &dyn std::error::Error {
        match self {
            AbstractBootError::Orch(e) => e.root(),
            _ => panic!("Unexpected error type"),
        }
    }
}
