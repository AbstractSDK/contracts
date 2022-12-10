//! # AnsHost
//!
//! `abstract_os::ans_host` stores chain-specific contract addresses.
//!
//! ## Description
//! Contract and asset addresses are stored on the ans_host contract and are retrievable trough smart or raw queries.

use cosmwasm_schema::QueryResponses;
use cw_asset::{AssetInfo, AssetInfoUnchecked};

use crate::objects::pool_id::{PoolId, UncheckedPoolId};
use crate::objects::{
    asset_entry::AssetEntry,
    contract_entry::{ContractEntry, UncheckedContractEntry},
    pool_info::PoolMetadata,
    ChannelEntry, UncheckedChannelEntry,
};

pub type UniqueId = u64;
pub type AssetPair = (String, String);
type DexName = String;
pub type DexAssetPairing = (String, String, DexName);
pub type CompoundPoolId = (UniqueId, PoolId);

/// AnsHost state details
pub mod state {
    use crate::ans_host::{CompoundPoolId, DexAssetPairing, UniqueId};
    use cosmwasm_std::Addr;
    use cw_asset::AssetInfo;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    use crate::objects::{
        asset_entry::AssetEntry, common_namespace::ADMIN_NAMESPACE, contract_entry::ContractEntry,
        pool_info::PoolMetadata, ChannelEntry,
    };

    /// Ans host configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub next_unique_pool_id: UniqueId,
    }

    pub const CONFIG: Item<Config> = Item::new("config");

    /// Admin address store
    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    /// Stores name and address of tokens and pairs
    /// LP token pairs are stored alphabetically
    pub const ASSET_ADDRESSES: Map<AssetEntry, AssetInfo> = Map::new("assets");

    /// Stores contract addresses
    pub const CONTRACT_ADDRESSES: Map<ContractEntry, Addr> = Map::new("contracts");

    /// stores channel-ids
    pub const CHANNELS: Map<ChannelEntry, String> = Map::new("channels");

    /// Stores the registered dex names
    pub const REGISTERED_DEXES: Item<Vec<String>> = Item::new("registered_dexes");

    /// Stores (asset1, asset2, dex_name) -> (uniqueId, poolId)
    pub const PAIR_TO_POOL_ID: Map<DexAssetPairing, Vec<CompoundPoolId>> = Map::new("pool_ids");

    /// Stores the metadata for the pools
    pub const POOL_METADATA: Map<UniqueId, PoolMetadata> = Map::new("pools");
}

/// AnsHost Instantiate msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

/// AnsHost Execute msg
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Updates the contract addressbook
    UpdateContractAddresses {
        /// Contracts to update or add
        to_add: Vec<(UncheckedContractEntry, String)>,
        /// Contracts to remove
        to_remove: Vec<UncheckedContractEntry>,
    },
    /// Updates the Asset addressbook
    UpdateAssetAddresses {
        /// Assets to update or add
        to_add: Vec<(String, AssetInfoUnchecked)>,
        /// Assets to remove
        to_remove: Vec<String>,
    },
    /// Updates the Asset addressbook
    UpdateChannels {
        /// Assets to update or add
        to_add: Vec<(UncheckedChannelEntry, String)>,
        /// Assets to remove
        to_remove: Vec<UncheckedChannelEntry>,
    },
    /// Registers a dex
    RegisterDex {
        /// Name of the dex
        name: String,
    },
    /// Update the pools
    UpdatePools {
        /// Pools to update or add
        to_add: Vec<(UncheckedPoolId, PoolMetadata)>,
        /// Pools to remove
        to_remove: Vec<UniqueId>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

/// AnsHost smart-query
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Queries assets based on name
    /// returns [`AssetsResponse`]
    #[returns(AssetsResponse)]
    Assets {
        /// Names of assets to query
        names: Vec<String>,
    },
    /// Page over assets
    /// returns [`AssetListResponse`]
    #[returns(AssetListResponse)]
    AssetList {
        page_token: Option<String>,
        page_size: Option<u8>,
    },
    /// Queries contracts based on name
    /// returns [`ContractsResponse`]
    #[returns(ContractsResponse)]
    Contracts {
        /// Project and contract names of contracts to query
        names: Vec<ContractEntry>,
    },
    /// Page over contracts
    /// returns [`ContractListResponse`]
    #[returns(ContractListResponse)]
    ContractList {
        page_token: Option<ContractEntry>,
        page_size: Option<u8>,
    },
    /// Queries contracts based on name
    /// returns [`ChannelsResponse`]
    #[returns(ChannelsResponse)]
    Channels {
        /// Project and contract names of contracts to query
        names: Vec<ChannelEntry>,
    },
    /// Page over contracts
    /// returns [`ChannelListResponse`]
    #[returns(ChannelListResponse)]
    ChannelList {
        page_token: Option<ChannelEntry>,
        page_size: Option<u8>,
    },
    /// Retrieve the registered dexes
    /// returns [`RegisteredDexesResponse`]
    #[returns(RegisteredDexesResponse)]
    RegisteredDexes {},
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
/// Query response
#[cosmwasm_schema::cw_serde]
pub struct AssetsResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<(AssetEntry, AssetInfo)>,
}

/// Query response
#[cosmwasm_schema::cw_serde]
pub struct AssetListResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<(AssetEntry, AssetInfo)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ContractsResponse {
    /// Contracts (name, address)
    pub contracts: Vec<(ContractEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ContractListResponse {
    /// Contracts (name, address)
    pub contracts: Vec<(ContractEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChannelsResponse {
    pub channels: Vec<(ChannelEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChannelListResponse {
    pub channels: Vec<(ChannelEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct RegisteredDexesResponse {
    pub dexes: Vec<String>,
}
