use cosmwasm_std::{Binary, Coin, CosmosMsg, Empty, QueryRequest, Timestamp, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use simple_ica::StdAck;

use self::state::ChainData;

pub mod state {
    use serde::{Deserialize, Serialize};

    use cosmwasm_std::{Addr, Coin, Timestamp};
    use cw_storage_plus::{Item, Map};

    use super::LatestQueryResponse;

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub admin: Addr,
        pub chain: String,
    }

    #[cosmwasm_schema::cw_serde]
    pub struct Memory {
        address: Addr,
    }

    #[cosmwasm_schema::cw_serde]
    pub struct ChainData {
        /// last block balance was updated (0 is never)
        pub last_update_time: Timestamp,
    }

    pub const CONFIG: Item<Config> = Item::new("config");
    pub const CHAINS: Map<&str, ChainData> = Map::new("chains");
    pub const LATEST_QUERIES: Map<&str, LatestQueryResponse> = Map::new("queries");
}

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub memory_address: String,
    pub chain: String,
}

#[cosmwasm_schema::cw_serde]
pub struct CallbackInfo {
    pub id: String,
    pub receiver: String,
}

#[cosmwasm_schema::cw_serde]
/// Actions that get parsed into [`crate::ibc_host::PacketMsg`]
pub enum IbcAction{
    App(Binary),
    Dispatch(Vec<CosmosMsg<Empty>>),
    Query(Vec<QueryRequest<Empty>>),
    Balances,
    SendAllBack,
}

#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Changes the admin
    UpdateAdmin {
        admin: String,
    },
    SendPacket {
        /// host chain to be executed on
        /// Example: "osmosis"
        host_chain: String,
        /// Action to be performed on the host chain/ proxy account
        action: IbcAction,
        /// Optional callback to a specified contract
        callback: Option<CallbackInfo>
    },
    CheckRemoteBalance {
        host_chain: String,
    },
    /// If you sent funds to this contract, it will attempt to ibc transfer them
    /// to the account on the remote side of this channel.
    /// If we don't have the address yet, this fails.
    SendFunds {
        host_chain: String,
    },
}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg {
    // Returns current admin
    Admin {},
    // Shows all open channels (incl. remote info)
    ListChains {},
    // Get channel info for one chain
    Chain { name: String },
    // Get remote account info for a chain + OS
    RemoteProxy { chain: String, os_id: u32},
}

#[cosmwasm_schema::cw_serde]
pub struct AdminResponse {
    pub admin: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ListChainsResponse {
    pub chains: Vec<ChainInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct LatestQueryResponse {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    pub response: StdAck,
}

#[cosmwasm_schema::cw_serde]
pub struct RemoteProxyResponse {
    /// last block balance was updated (0 is never)
    pub channel_id: String,
    /// address of the remote proxy 
    pub proxy_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ChainInfo {
    pub channel_id: String,
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
}

impl ChainInfo {
    pub fn convert(channel_id: String, input: ChainData) -> Self {
        ChainInfo {
            channel_id,
            last_update_time: input.last_update_time,
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct ChainResponse {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
}

impl From<ChainData> for ChainResponse {
    fn from(input: ChainData) -> Self {
        ChainResponse {
            last_update_time: input.last_update_time,
        }
    }
}
