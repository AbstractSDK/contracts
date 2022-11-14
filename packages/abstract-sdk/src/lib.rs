//! # Abstract SDK
//!
//! An SDK for writing Abstract OS smart-contracts.
//!
//! ## Description
//! The internal lay-out and state management of Abstract OS allows smart-contract engineers to write deployment-generic code.
//! The functions provided by this SDK can be used to quickly write and test your unique CosmWasm application.

pub extern crate abstract_os;

pub mod base;
pub mod features;
mod ans_host_traits;
mod cw20;
mod manager;
mod module_traits;
mod apis;

pub use apis::{
    applications::ApplicationInterface, execution::Execution, transfer::TransferInterface,
    vault::VaultInterface, verify::Verification,
};

pub mod ans_host {
    pub use abstract_os::objects::ans_host::AnsHost;
}



pub use abstract_os::{
    objects::common_namespace::{ADMIN_NAMESPACE, BASE_STATE, CONTRACT_VERSION},
    registry::*,
};

pub use ans_host_traits::Resolve;

