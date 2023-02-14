#![doc(html_logo_url = "https://raw.githubusercontent.com/Abstract-OS/assets/mainline/logo.svg")]
#![doc = include_str!("../README.md")]
#![doc(test(attr(
    warn(unused),
    deny(warnings),
    // W/o this, we seem to get some bogus warning about `extern crate zbus`.
    // allow(unused_extern_crates),
)))]

pub extern crate abstract_macros as macros;
pub extern crate abstract_os as os;

mod ans_resolve;
mod apis;
pub mod base;
pub mod cw_helpers;
pub mod feature_objects;

pub use crate::apis::{
    bank::TransferInterface, dex::DexInterface, execution::Execution, ibc::IbcInterface,
    modules::ModuleInterface, respond::AbstractResponse, vault::VaultInterface,
    verify::OsVerification, version_registry::ModuleRegistryInterface,
};
pub mod features {
    pub use crate::base::features::*;
}

pub use ans_resolve::Resolve;

/// Common state-store namespaces.
pub mod namespaces {
    pub use abstract_os::objects::common_namespace::*;
}

/// Abstract reserved version control entries.
pub mod register {
    pub use abstract_os::registry::*;
}
