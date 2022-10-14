use cosmwasm_std::{Binary, Coin, CosmosMsg, Empty, QueryRequest, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use simple_ica::StdAck;

use self::state::AccountData;

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
    pub struct AccountData {
        /// last block balance was updated (0 is never)
        pub last_update_time: Timestamp,
        /// In normal cases, it should be set, but there is a delay between binding
        /// the channel and making a query and in that time it is empty.
        ///
        /// Since we do not have a way to validate the remote address format, this
        /// must not be of type `Addr`.
        pub remote_addr: Option<String>,
        pub remote_balance: Vec<Coin>,
    }

    pub const CONFIG: Item<Config> = Item::new("config");
    pub const ACCOUNTS: Map<&str, AccountData> = Map::new("accounts");
    pub const LATEST_QUERIES: Map<&str, LatestQueryResponse> = Map::new("querys");
}

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub memory_address: String,
    pub chain: String,
}

#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Changes the admin
    UpdateAdmin {
        admin: String,
    },
    SendMsgs {
        /// Chain we want to send request to
        host_chain: String,
        /// Note: we don't handle custom messages on remote chains
        /// Use Stargate instead
        msg: Binary,
        /// If set, the original caller will get a callback with of the result, along with this id
        callback_id: Option<String>,
        /// Contract on which callback will be called
        callback_receiver: Option<String>,
    },
    CheckRemoteBalance {
        host_chain: String,
    },
    IbcQuery {
        host_chain: String,
        msgs: Vec<QueryRequest<Empty>>,
        /// If set, the original caller will get a callback with of the result, along with this id
        callback_id: Option<String>,
        /// Contract on which callback will be called
        callback_receiver: Option<String>,
    },
    /// If you sent funds to this contract, it will attempt to ibc transfer them
    /// to the account on the remote side of this channel.
    /// If we don't have the address yet, this fails.
    SendFunds {
        /// The channel id we use above to send the simple-ica query on
        host_chain: String,
        /// The channel to use for ibctransfer. This is bound to a different
        /// port and handled by a different module.
        /// It should connect to the same chain as the ica_channel_id does
        transfer_channel_id: String,
    },
}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg {
    // Returns current admin
    Admin {},
    // Shows all open accounts (incl. remote info)
    ListAccounts {},
    // Get account for one channel
    Account { channel_id: String },
    // Get latest query
    LatestQueryResult { channel_id: String },
}

#[cosmwasm_schema::cw_serde]
pub struct AdminResponse {
    pub admin: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct LatestQueryResponse {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    pub response: StdAck,
}

#[cosmwasm_schema::cw_serde]
pub struct AccountInfo {
    pub channel_id: String,
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    /// in normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty
    pub remote_addr: Option<String>,
    pub remote_balance: Vec<Coin>,
}

impl AccountInfo {
    pub fn convert(channel_id: String, input: AccountData) -> Self {
        AccountInfo {
            channel_id,
            last_update_time: input.last_update_time,
            remote_addr: input.remote_addr,
            remote_balance: input.remote_balance,
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct AccountResponse {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    /// in normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty
    pub remote_addr: Option<String>,
    pub remote_balance: Vec<Coin>,
}

impl From<AccountData> for AccountResponse {
    fn from(input: AccountData) -> Self {
        AccountResponse {
            last_update_time: input.last_update_time,
            remote_addr: input.remote_addr,
            remote_balance: input.remote_balance,
        }
    }
}
