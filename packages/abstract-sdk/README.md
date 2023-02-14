# Abstract-SDK

<!-- [![](https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github)](https://github.com/Abstract-OS/contracts)   -->

[![](https://docs.rs/abstract-sdk/badge.svg)](https://docs.rs/abstract-sdk) [![](https://img.shields.io/crates/v/abstract-sdk)](https://crates.io/crates/abstract-sdk)

This crate provides a set of modular APIs for developers to use in their [CosmWasm](https://cosmwasm.com/) smart-contracts.

## Getting started

To get started with the Abstract SDK you first need to understand the basic features that we provide and how you can use those features to create composable smart-contract APIs with our SDK.  

### Features

Abstract features are traits that can be implemented on a struct. Depending on the use-case that struct can represent a smart-contract or it can be a simple struct that just implements a single feature. Each feature unlocks a function on the object
which allows you to retrieve some information. By composing these features it is possible to write advanced APIs that are automatically implemented on objects that support its required features.

### APIs

The Abstract APIs are objects that can only be retrieved if a contract or feature-object implements the required features/api traits. If the trait constraints for the API is met it is automatically implemented on the object and allows you to retrieve the API object.  

#### Example

The [`Bank`](https://docs.rs/abstract-sdk/latest/abstract_sdk/apis/bank) API allows developers to transfer assets from and to the OS through their module object. We now want to use this API to create a `Splitter` API that splits the transfer of some amount of funds between a set of receivers.

```rust,no_run
use crate::TransferInterface;
use abstract_os::objects::AnsAsset;
use cosmwasm_std::{Addr, CosmosMsg, Deps, StdResult, Uint128};

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
    /// Split an asset to multiple users
    pub fn split(&self, asset: AnsAsset, receivers: &[Addr]) -> StdResult<Vec<CosmosMsg>> {
        // split the asset between all receivers
        let receives_each = AnsAsset {
            amount: asset
                .amount
                .multiply_ratio(Uint128::one(), Uint128::from(receivers.len() as u128)),
            ..asset
        };

        // Retrieve the bank API
        let bank = self.base.bank(self.deps.clone());
        let transfer_msgs: StdResult<_> = receivers
            .iter()
            .map(|receiver| {
                // Construct the transfer message
                bank.transfer(vec![receives_each.clone()], receiver)
            })
            .collect();

        Ok(transfer_msgs?)
    }
}

```
