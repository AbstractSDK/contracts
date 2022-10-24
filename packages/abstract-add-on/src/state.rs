use std::marker::PhantomData;

use cosmwasm_std::{Addr, Empty, StdResult, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_controllers::Admin;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use abstract_sdk::{memory::Memory, ADMIN, BASE_STATE};

use crate::{execute::IbcHandlerFn, AddOnError};

/// The state variables for our AddOnContract.
pub struct AddOnContract<
    'a,
    Request: Serialize + DeserializeOwned,
    Error: From<cosmwasm_std::StdError> + From<AddOnError>,
    Callback: Serialize + DeserializeOwned = Empty,
> {
    // Every DApp should use the provided memory contract for token/contract address resolution
    pub base_state: Item<'a, AddOnState>,
    pub version: Item<'a, ContractVersion>,
    pub admin: Admin<'a>,
    pub dependencies: &'static [&'static str],
    pub ibc_callbacks: &'a [(&'static str, IbcHandlerFn<Request, Error, Callback>)],

    _phantom_data: PhantomData<Request>,
    _phantom_data_error: PhantomData<Error>,
    _phantom_data_callbacks: PhantomData<Callback>,
}

impl<
        'a,
        T: Serialize + DeserializeOwned,
        C: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<AddOnError>,
    > Default for AddOnContract<'a, T, E, C>
{
    fn default() -> Self {
        Self::new()
    }
}

/// Constructor
impl<
        'a,
        T: Serialize + DeserializeOwned,
        C: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<AddOnError>,
    > AddOnContract<'a, T, E, C>
{
    fn new() -> Self {
        Self {
            version: CONTRACT,
            base_state: Item::new(BASE_STATE),
            admin: Admin::new(ADMIN),
            ibc_callbacks: &[],
            dependencies: &[],
            _phantom_data: PhantomData,
            _phantom_data_callbacks: PhantomData,
            _phantom_data_error: PhantomData,
        }
    }
    /// add IBC callback handler to contract
    pub const fn with_ibc_callbacks(
        mut self,
        callbacks: &'a [(&'static str, IbcHandlerFn<T, E, C>)],
    ) -> Self {
        self.ibc_callbacks = callbacks;
        self
    }

    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [&'static str]) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<AddOnState> {
        self.base_state.load(store)
    }

    pub fn version(&self, store: &dyn Storage) -> StdResult<ContractVersion> {
        self.version.load(store)
    }
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddOnState {
    /// Proxy contract address for relaying transactions
    pub proxy_address: Addr,
    /// Memory contract struct (address)
    pub memory: Memory,
}
