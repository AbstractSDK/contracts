use std::marker::PhantomData;

use abstract_os::ibc_host::PacketMsg;
use abstract_sdk::{memory::Memory, AbstractContract, Handler, ReplyHandlerFn, BASE_STATE};

use cosmwasm_std::{Addr, Binary, Empty, StdResult, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    reply::{reply_dispatch_callback, reply_init_callback, INIT_CALLBACK_ID, RECEIVE_DISPATCH_ID},
    HostError,
};

pub const TRADER_NAMESPACE: &str = "traders";

/// Store channel information for proxy contract creation reply
pub const PENDING: Item<(String, u32)> = Item::new("pending");
/// Store the processing packet information for processing in Reply along with the channel id it came from
pub const PROCESSING_PACKET: Item<(PacketMsg, String)> = Item::new("processing");
/// (channel-id,os_id) -> local_proxy_addr
pub const ACCOUNTS: Map<(&str, u32), Addr> = Map::new("accounts");
/// (channel-id,os_id) -> client_proxy_addr
pub const CLIENT_PROXY: Map<(&str, u32), String> = Map::new("client_proxy");
/// List of closed channels
/// Allows for fund recovery
pub const CLOSED_CHANNELS: Item<Vec<String>> = Item::new("closed");
// this stores all results from current dispatch
pub const RESULTS: Item<Vec<Binary>> = Item::new("results");

/// The state variables for our host contract.
pub struct Host<
    Error: From<cosmwasm_std::StdError> + From<HostError> + 'static,
    CustomExecMsg: 'static = Empty,
    CustomInitMsg: 'static = Empty,
    CustomQueryMsg: 'static = Empty,
    CustomMigrateMsg: 'static = Empty,
    Receive: 'static = Empty,
> {
    // Scaffolding contract that handles type safety and provides helper methods
    pub(crate) contract: AbstractContract<
        Self,
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        Receive,
    >,
    // pub admin: Admin<'static>,
    // Custom state for every Host
    pub proxy_address: Option<Addr>,
    pub(crate) base_state: Item<'static, HostState>,
    pub(crate) chain: &'static str,
}

// Constructor
impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    pub const fn new(name: &'static str, version: &'static str, chain: &'static str) -> Self {
        let contract = AbstractContract::new(name, version).with_replies([
            &[
                // add reply handlers we want to support by default
                (RECEIVE_DISPATCH_ID, reply_dispatch_callback),
                (INIT_CALLBACK_ID, reply_init_callback),
            ],
            &[],
        ]);
        Self {
            contract,
            chain,
            base_state: Item::new(BASE_STATE),
            proxy_address: None,
        }
    }

    /// add reply handler to contract
    pub const fn with_reply_handlers(
        mut self,
        reply_handlers: &'static [(u64, ReplyHandlerFn<Self, Error>)],
    ) -> Self {
        // update this to store static iterators
        let mut new_reply_handlers = self.contract.reply_handlers;
        new_reply_handlers[1] = reply_handlers;
        self.contract = self.contract.with_replies(new_reply_handlers);
        self
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<HostState> {
        self.base_state.load(store)
    }

    pub fn target(&self) -> Result<&Addr, HostError> {
        self.proxy_address.as_ref().ok_or(HostError::NoTarget)
    }
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HostState {
    /// Memory contract struct (address)
    pub memory: Memory,
    /// code id for Stargate proxy contract
    pub cw1_code_id: u64,
    /// Chain identifier
    pub chain: String,
    pub admin: Addr,
}
