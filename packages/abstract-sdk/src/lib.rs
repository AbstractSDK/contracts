//! # Abstract SDK
//!
//! An SDK for writing Abstract OS smart-contracts.
//!
//! ## Description
//! The internal lay-out and state management of Abstract OS allows smart-contract engineers to write deployment-generic code.
//! The functions provided by this SDK can be used to quickly write and test your unique CosmWasm application.

pub extern crate abstract_os;

mod ans_resolve;
mod apis;
pub mod base;

pub mod feature_objects;
mod manager;
mod module_traits;

pub use ans_resolve::Resolve;

pub use crate::apis::{
    applications::ApplicationInterface, execution::Execution, transfer::TransferInterface,
    vault::VaultInterface, verify::Verification, version_register::VersionRegisterInterface,ans::AnsInterface,
};

pub use abstract_os::{
    objects::common_namespace::{ADMIN_NAMESPACE, BASE_STATE, CONTRACT_VERSION},
    registry::*,
};