use std::collections::HashMap;

use abstract_os::{objects::{ContractEntry, AssetEntry, ChannelEntry, pool_id::UncheckedPoolAddress, PoolMetadata, UniquePoolId}, dex::DexName, ans_host::state::{ASSET_ADDRESSES, CONTRACT_ADDRESSES, CHANNELS, POOL_METADATA, REGISTERED_DEXES}};
use cosmwasm_std::{Addr, testing::{mock_dependencies, MockApi}};
use cw_asset::AssetInfo;

use crate::{MockQuerierBuilder, TEST_ANS_HOST};




/// mirror ANS state
/// ```rust,ignore
/// pub const ASSET_ADDRESSES: Map<&AssetEntry, AssetInfo> = Map::new("assets");
/// pub const REV_ASSET_ADDRESSES: Map<&AssetInfo, AssetEntry> = Map::new("rev_assets");
/// pub const CONTRACT_ADDRESSES: Map<&ContractEntry, Addr> = Map::new("contracts");
/// pub const CHANNELS: Map<&ChannelEntry, String> = Map::new("channels");
/// pub const REGISTERED_DEXES: Item<Vec<DexName>> = Item::new("registered_dexes");
/// // Stores the asset pairing entries to their pool ids
/// // (asset1, asset2, dex_name) -> {id: uniqueId, pool_id: poolId}
/// pub const ASSET_PAIRINGS: Map<&DexAssetPairing, Vec<PoolReference>> = Map::new("pool_ids");
/// pub const POOL_METADATA: Map<UniquePoolId, PoolMetadata> = Map::new("pools");
/// ``` 
pub struct MockAnsHost<'a> {
    pub contracts: Vec<(&'a ContractEntry, Addr)>,
    pub assets: Vec<(&'a AssetEntry, AssetInfo)>,
    pub channels: Vec<(&'a ChannelEntry, String)>,
    pub pools: Vec<(UncheckedPoolAddress, PoolMetadata)>
}

impl Default for MockAnsHost<'_> {
    fn default() -> Self {
        Self {
            contracts: vec![],
            assets: vec![],
            channels: vec![],
            pools: vec![],
        }
    }
}

impl MockAnsHost<'_>{
    // consume to drop self as re-using is not possible
    pub fn insert_into(self, querier_builder: &mut MockQuerierBuilder) {
        querier_builder.with_contract_map_entries(TEST_ANS_HOST, ASSET_ADDRESSES, self.assets.clone());
        querier_builder.with_contract_map_entries(TEST_ANS_HOST, CONTRACT_ADDRESSES, self.contracts.clone());
        querier_builder.with_contract_map_entries(TEST_ANS_HOST, CHANNELS, self.channels.clone());

        let mut unique_id = UniquePoolId::new(0);
        let dexes = vec![];
        for (pool_addr, pool_meta) in self.pools.clone() {
            let dex = pool_meta.dex.clone();
            if !dexes.contains(&dex) {
                dexes.push(dex);
            }
            let pool_addr = pool_addr.check(&MockApi::default()).unwrap();
            querier_builder.with_contract_map_entries(TEST_ANS_HOST, POOL_METADATA, (pool_addr, pool_meta));
            unique_id.increment();
        }
        querier_builder.with_contract_item(TEST_ANS_HOST, REGISTERED_DEXES, &dexes);
    }
}


/// Execute an action on every asset pairing in the list of assets
/// Example: assets: [A, B, C] -> [A, B], [A, C], [B, C]
fn exec_on_asset_pairings<T, A, E>(assets: &[AssetEntry], mut action: A) -> StdResult<()>
where
    A: FnMut(AssetPair) -> Result<T, E>,
    StdError: From<E>,
{
    for (i, asset_x) in assets.iter().enumerate() {
        for (j, asset_y) in assets.iter().enumerate() {
            // Skip self-pairings
            if i == j || asset_x == asset_y {
                continue;
            }
            let pair: AssetPair = (asset_x.clone(), asset_y.clone());
            action(pair)?;
        }
    }
    Ok(())
}

