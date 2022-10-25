#![feature(generic_associated_types)]
//! # Abstract API
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};
pub type ApiResult<C = Empty> = Result<Response<C>, ApiError>;
// Default to Empty


pub use error::ApiError;
pub use crate::state::ApiContract;

mod execute;
mod ibc_callback;
mod query;
mod receive;
pub mod error;
pub mod instantiate;
pub mod state;
/// Abstract SDK trait implementations
pub mod traits;

