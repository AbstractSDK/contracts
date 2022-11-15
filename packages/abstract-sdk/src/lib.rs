//! # Abstract SDK
//!
//! An SDK for writing Abstract OS smart-contracts.
//!
//! ## Description
//! The internal lay-out and state management of Abstract OS allows smart-contract engineers to write deployment-generic code.
//! The functions provided by this SDK can be used to quickly write and test your unique CosmWasm application.

pub extern crate abstract_os;

mod ans_resolve;
pub mod apis;
pub mod base;
pub mod feature_objects;


pub use ans_resolve::Resolve;
pub use crate::apis::{
    ans::AnsInterface, applications::ApplicationInterface, execution::Execution, ibc::IbcInterface,
    transfer::TransferInterface, vault::VaultInterface, verify::Verification,
    version_register::VersionRegisterInterface,
};

pub use abstract_os::{
    objects::common_namespace::{ADMIN_NAMESPACE, BASE_STATE, CONTRACT_VERSION},
    registry::*,
};
