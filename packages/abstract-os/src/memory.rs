//! # Memory
//!
//! `abstract_os::memory` stores chain-specific contract addresses.
//!
//! ## Description
//! Contract and asset addresses are stored on the memory contract and are retrievable trough smart or raw queries.

use cosmwasm_schema::QueryResponses;
use cw_asset::{AssetInfo, AssetInfoUnchecked};
use crate::memory::state::DexPoolData;

use crate::objects::dex_pool_entry::UncheckedDexPoolEntry;
use crate::objects::pool_id::{PoolId, UncheckedPoolId};
use crate::objects::{
    asset_entry::AssetEntry,
    contract_entry::{ContractEntry, UncheckedContractEntry},
    ChannelEntry, DexPoolEntry, UncheckedChannelEntry,
};

/// Memory state details
pub mod state {
    use cosmwasm_std::Addr;
    use cw_asset::AssetInfo;
    use cw_controllers::Admin;
    use cw_storage_plus::{Index, IndexList, IndexedMap, Map, MultiIndex, UniqueIndex};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::objects::dex_pool_entry::DexPoolEntry;
    use crate::objects::pool_id::{PoolId, UncheckedPoolId};
    use crate::objects::pool_info::{Pool, UncheckedPool};
    use crate::objects::{asset_entry::AssetEntry, contract_entry::ContractEntry, ChannelEntry};

    /// Admin address store
    pub const ADMIN: Admin = Admin::new("admin");
    /// Stores name and address of tokens and pairs
    /// LP token pairs are stored alphabetically
    pub const ASSET_ADDRESSES: Map<AssetEntry, AssetInfo> = Map::new("assets");

    /// Stores contract addresses
    pub const CONTRACT_ADDRESSES: Map<ContractEntry, Addr> = Map::new("contracts");

    /// stores channel-ids
    pub const CHANNELS: Map<ChannelEntry, String> = Map::new("channels");

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
    pub struct DexPoolData {
        pub dex: String,
        // TODO: checked (Pool)
        pub info: UncheckedPool,
    }

    pub struct DexPoolDataIndexes<'a> {
        pub dex: MultiIndex<'a, String, DexPoolData, String>,
        pub assets: MultiIndex<'a, String, DexPoolData, String>,
        // TODO: checked (PoolId)
        pub pool_id: MultiIndex<'a, UncheckedPoolId, DexPoolData, String>,
        // // should be unique across the map
        // pub dex_assets: UniqueIndex<'a, (Vec<u8>, Vec<u8>), DexPoolData, String>,
    }

    impl<'a> IndexList<DexPoolData> for DexPoolDataIndexes<'a> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<DexPoolData>> + '_> {
            let v: Vec<&dyn Index<DexPoolData>> = vec![&self.dex, &self.pool_id, &self.assets];
            Box::new(v.into_iter())
        }
    }

    // TODO: when cw_storage_plus supports const indexed maps, switch to that
    pub fn dex_pools<'a>() -> IndexedMap<'a, DexPoolEntry, DexPoolData, DexPoolDataIndexes<'a>> {
        let indexes = DexPoolDataIndexes {
            assets: MultiIndex::new(
                |_pk, pool| pool.info.assets.clone(),
                "data",
                "data__assets",
            ),
            dex: MultiIndex::new(|_pk, pool| pool.dex.clone(), "data", "data__dex"),
            pool_id: MultiIndex::new(|_pk, pool| pool.info.id.clone(), "data", "data__pool_id"),
            // dex_assets:  UniqueIndex::new(
            // |d| (d.dex.as_bytes().to_vec(), d.assets.as_bytes().to_vec()),
            // "data__dex_assets",
            // ),
        };
        IndexedMap::new("dex_pools", indexes)
    }
}

/// Memory Instantiate msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

/// Memory Execute msg
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
    /// Updates the dex pairs
    UpdateDexPools {
        /// Pairs to update or add
        to_add: Vec<(UncheckedDexPoolEntry, UncheckedPoolId)>,
        /// Pairs to remove
        to_remove: Vec<UncheckedDexPoolEntry>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

/// Memory smart-query
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
    /// Queries channels based on name
    /// returns [`ChannelsResponse`]
    #[returns(ChannelsResponse)]
    Channels {
        /// Project and channel names of channels to query
        names: Vec<ChannelEntry>,
    },
    /// Page over contracts
    /// returns [`ChannelListResponse`]
    #[returns(ChannelListResponse)]
    ChannelList {
        page_token: Option<ChannelEntry>,
        page_size: Option<u8>,
    },
    /// Queries dex_pools based on dex and asset_pair
    /// returns [`ChannelsResponse`]
    #[returns(DexPoolsResponse)]
    DexPools {
        /// [{ dex, asset_pair }] of dex_pools to query
        entries: Option<Vec<DexPoolEntry>>,
        /// name of the dex to query
        dex: Option<String>
    },
    /// Page over dex pairs
    /// returns [`DexPoolListResponse`]
    #[returns(DexPoolListResponse)]
    DexPoolList {
        page_token: Option<DexPoolEntry>,
        page_size: Option<u8>,
    },
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
pub struct DexPoolsResponse {
    pub pairs: Vec<(DexPoolEntry, DexPoolData)>,
}

#[cosmwasm_schema::cw_serde]
pub struct DexPoolListResponse {
    pub pairs: Vec<(DexPoolEntry, PoolId)>,
}
