//! [![github]](https://github.com/Abstract-OS/contracts)&ensp;[![crates-io]](https://crates.io/crates/abstract-sdk)&ensp;[![docs-rs]](https://docs.rs/abstract-sdk)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//! <br>  
//! </br>
//! This crate provides a set of modular APIs for developers to use in their [CosmWasm](https://cosmwasm.com/) smart-contracts.
//! 
//! # Details
//! To use an API either construct a [`feature object`](crate::feature_objects) or use an Abstract base contract as the starting-point of your application.  
//! The available base contracts are: 
//! > - [Add-on](https://crates.io/crates/abstract-add-on) ([Template](https://github.com/Abstract-OS/addon-module-template))
//! > - [API](https://crates.io/crates/abstract-api) ([Template (WIP)]())
//! > - [IBC-host](https://crates.io/crates/abstract-ibc-host) ([Template (WIP)]())
//! 
//! ```
//!   # use crate::feature_objects::VersionControlContract;
//!   #
//!   # pub struct MyContract {
//!   #     
//!   # }
//!   # 
//!   # impl Identification for MyContract {
//!   #     fn proxy_address(&self, _deps: Deps) -> cosmwasm_std::StdResult<Addr> {
//!   #         Ok(Addr::unchecked("just_an_example".into()))
//!   #     }
//!   # }
//! 
//!   use anyhow::Result;
//!
//!   fn forward_deposit(my_contract: MyContract,deps: Deps, deposit: AnsAsset) -> StdResult<CosmosMsg> {
//!       let transfers = my_contract.transfer()
//!       Ok(map)
//!   }
//!   #
//!   # fn main() {}
//!   ```

pub extern crate abstract_os as os;

mod ans_resolve;
mod apis;
pub mod base;
pub mod feature_objects;

pub use crate::apis::{
    ans::AnsInterface, applications::ApplicationInterface, execution::Execution, ibc::IbcInterface,
    bank::TransferInterface, vault::VaultInterface, verify::Verification,
    version_register::VersionRegisterInterface,
};
pub use ans_resolve::Resolve;

pub mod namespaces{
    pub use abstract_os::objects::common_namespace::*;
}

pub mod register{
    pub use abstract_os::registry::*;
}