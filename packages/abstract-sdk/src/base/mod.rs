//! Base of an Abstract Module and its features.  
//!
//! Is used by the `abstract-api`, `abstract-ibc-host` and `abstract-app` crates.

mod contract_base;
pub mod endpoints;
pub mod features;
mod handler;

pub use contract_base::ContractName;
pub use contract_base::VersionString;
pub use contract_base::{
    AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
    MigrateHandlerFn, QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn,
};
pub use endpoints::{
    migrate::MigrateEndpoint, ExecuteEndpoint, IbcCallbackEndpoint, InstantiateEndpoint,
    QueryEndpoint, ReceiveEndpoint, ReplyEndpoint,
};
pub use handler::Handler;
