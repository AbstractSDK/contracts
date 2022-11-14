//! # Module Features
//! Features are traits that are implemented on the base layer of a module. Implementing a feature unlocks the API objects that are dependent on it.  
//! You can easily create and provide your own API for other smart-contract developers by using these features as trait bounds.
//!
//! Example APIs can be found in [`crate::apis`].

mod abstract_name_service;
mod contract_deps;
mod identification;
mod versioning;

pub use {
    abstract_name_service::AbstractNameSystem, identification::Identification,
    versioning::Versioning,contract_deps::ContractDeps,
};
