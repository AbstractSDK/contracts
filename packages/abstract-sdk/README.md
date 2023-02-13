# Abstract-SDK

<!-- [![](https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github)](https://github.com/Abstract-OS/contracts)   -->

[![](https://docs.rs/abstract-sdk/badge.svg)](https://docs.rs/abstract-sdk) [![](https://img.shields.io/crates/v/abstract-sdk)](https://crates.io/crates/abstract-sdk)

This crate provides a set of modular APIs for developers to use in their [CosmWasm](https://cosmwasm.com/) smart-contracts.

## Getting started

To get started with the Abstract SDK you first need to understand the basic features that we provide and how we use those features to create composable smart-contract APIs in our SDK.  

### Features

Abstract features are traits that can be implemented on a struct. Depending on the use-case that struct can represent a smart-contract or it can be a simple struct that just implements a single feature.  

> [These are all the available features and their functions.](https://docs.rs/abstract-sdk/latest/abstract_sdk/base/features)

### APIs

The APIs are objects that can only be retrieved if a contract or feature-object implements the required features/api traits. If the trait constraints for the API is met it is automatically implemented on the object and allows you to retrieve the API object.  

#### Example

The [`Bank`](https://docs.rs/abstract-sdk/latest/abstract_sdk/apis/bank) API allows developers to transfer assets from and to the OS through their module object. We now want to use this API to create a `Splitter` API that splits the transfer of some amount of funds between a set of receivers.

```rust
use super::{execution::Execution};
use crate::{ans_resolve::Resolve, base::features::AbstractNameService};
use abstract_os::objects::AnsAsset;
use cosmwasm_std::{Addr, CosmosMsg, Deps, StdResult};
use os::objects::AssetEntry;

// Trait to retrieve the Splitter object
// Depends on the ability to transfer funds
pub trait SplitterInterface: TransferInterface {
    fn splitter<'a>(&'a self, deps: Deps<'a>) -> Splitter<Self> {
        Splitter { base: self, deps }
    }
}

// Implement for every object that can transfer funds
impl<T> SplitterInterface for T where T: TransferInterface {}

#[derive(Clone)]
pub struct Splitter<'a, T: SplitterInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: SplitterInterface> Splitter<'a, T> {
    /// Get the balances of the provided **assets**.
    pub fn split(&self, assets: &[AnsAsset], receivers: &[Addr]) -> StdResult<Vec<CosmosMsg>> {
        let bank = self.base.bank(self.deps.clone());
        let transfer_msgs = assets.

        Ok(transfer_msgs)
    }
}

```
### Abstract Base

To use an API either construct a [`feature object`](https://docs.rs/abstract-sdk/latest/abstract_sdk/feature_objects/index.html) or use an Abstract base contract as the starting-point of your application.  
The available base contracts are:

> - [App](https://crates.io/crates/abstract-app)
> - [API](https://crates.io/crates/abstract-api)
> - [IBC-host](https://crates.io/crates/abstract-ibc-host)

```rust,no_run
use abstract_sdk::{feature_objects::VersionControlContract, base::features::{Identification, AbstractNameService, ModuleIdentification}};
use cosmwasm_std::{StdResult, Deps, MessageInfo, CosmosMsg, Addr};
use abstract_sdk::feature_objects::AnsHost;

pub struct MyContract {}

impl Identification for MyContract {
    fn proxy_address(&self, _deps: Deps) -> cosmwasm_std::StdResult<Addr> {
        Ok(Addr::unchecked("just_an_example"))
    }
}
impl ModuleIdentification for MyContract {
    fn module_id(&self) -> &'static str { "my_contract" }
}
impl AbstractNameService for MyContract {
    fn ans_host(&self, _deps: Deps) -> cosmwasm_std::StdResult<AnsHost> {
        Ok(AnsHost{address: Addr::unchecked("just_an_example")})
    }
}
use abstract_sdk::TransferInterface;

fn forward_deposit(deps: Deps, my_contract: MyContract, message_info: MessageInfo) -> StdResult<CosmosMsg> {
    let send_deposit_to_vault_msg = my_contract.bank(deps).deposit_coins(message_info.funds)?;
    Ok(send_deposit_to_vault_msg)
}
```


## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
abstract-sdk = "0.1.0"
```