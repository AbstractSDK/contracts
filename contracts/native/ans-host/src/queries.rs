use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult, StdError, Storage};

use abstract_os::ans_host::{AssetPair, AssetPairingEntry, CompoundPoolId, AssetPairingKey, AssetPairingFilter, PoolIdListResponse, PoolsResponse, RegisteredDexesResponse, UniqueId};
use abstract_os::{
    ans_host::{
        state::{ASSET_ADDRESSES, CHANNELS, CONTRACT_ADDRESSES, REGISTERED_DEXES},
        AssetListResponse, AssetsResponse, ChannelListResponse, ChannelsResponse,
        ContractListResponse, ContractsResponse,
    },
    objects::{AssetEntry, ChannelEntry, ContractEntry},
};
use cw_asset::AssetInfo;
use cw_storage_plus::Bound;
use abstract_os::ans_host::state::{ASSET_PAIRS, POOL_METADATA};
use abstract_os::dex::DexName;
use abstract_os::objects::pool_info::PoolMetadata;
use crate::error::AnsHostError;

const DEFAULT_LIMIT: u8 = 15;
const MAX_LIMIT: u8 = 25;

pub fn query_assets(deps: Deps, _env: Env, asset_names: Vec<String>) -> StdResult<Binary> {
    let assets: Vec<AssetEntry> = asset_names
        .iter()
        .map(|name| name.as_str().into())
        .collect();
    let res: Result<Vec<(AssetEntry, AssetInfo)>, _> = ASSET_ADDRESSES
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| assets.contains(&e.as_ref().unwrap().0))
        .collect();
    to_binary(&AssetsResponse { assets: res? })
}

pub fn query_contract(deps: Deps, _env: Env, names: Vec<ContractEntry>) -> StdResult<Binary> {
    let res: Result<Vec<(ContractEntry, Addr)>, _> = CONTRACT_ADDRESSES
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| names.contains(&e.as_ref().unwrap().0))
        .collect();

    to_binary(&ContractsResponse {
        contracts: res?.into_iter().map(|(x, a)| (x, a.to_string())).collect(),
    })
}

pub fn query_channel(deps: Deps, _env: Env, names: Vec<ChannelEntry>) -> StdResult<Binary> {
    let res: Result<Vec<(ChannelEntry, String)>, _> = CHANNELS
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| names.contains(&e.as_ref().unwrap().0))
        .collect();

    to_binary(&ChannelsResponse { channels: res? })
}

pub fn query_asset_list(
    deps: Deps,
    last_asset_name: Option<String>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_asset_name.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(AssetEntry, AssetInfo)>, _> = ASSET_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    to_binary(&AssetListResponse { assets: res? })
}

pub fn query_contract_list(
    deps: Deps,
    last_contract: Option<ContractEntry>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_contract.map(Bound::exclusive);

    let res: Result<Vec<(ContractEntry, Addr)>, _> = CONTRACT_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();
    to_binary(&ContractListResponse {
        contracts: res?.into_iter().map(|(x, a)| (x, a.to_string())).collect(),
    })
}

pub fn query_channel_list(
    deps: Deps,
    last_channel: Option<ChannelEntry>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_channel.map(Bound::exclusive);

    let res: Result<Vec<(ChannelEntry, String)>, _> = CHANNELS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();
    to_binary(&ChannelListResponse { channels: res? })
}

pub fn query_registered_dexes(deps: Deps, _env: Env) -> StdResult<Binary> {
    let dexes = REGISTERED_DEXES.load(deps.storage)?;

    to_binary(&RegisteredDexesResponse { dexes })
}

//
// fn query_asset_pair_ids(deps: Deps, asset_x: &String, asset_y: &String, dex: &DexName) -> Result<Vec<AssetPairingEntry>, StdError> {
//     let dex_bound = Some(Bound::inclusive(dex));
//
//     let matched_res: Result<Vec<(DexName, Vec<CompoundPoolId>)>, _> = ASSET_PAIRS
//         .prefix((asset_x.clone(), asset_y.clone()))
//         .range(deps.storage, None, dex_bound, Order::Ascending)
//         .collect();
//
//     // Re add the key prefix
//     let matched: Vec<AssetPairingEntry> = matched_res?.into_iter()
//         .map(|(dex, ids)| ((asset_x.clone(), asset_y.clone(), dex), ids))
//         .collect();
//
//     Ok(matched)
// }


pub fn query_pool_id_list(deps: Deps, filter: Option<AssetPairingFilter>, page_token: Option<AssetPairingKey>, page_size: Option<u8>) -> StdResult<Binary> {
    let page_size = page_size.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let (asset_pair_filter, dex_filter) = match filter {
        Some(AssetPairingFilter { asset_pair, dex }) => (asset_pair, dex),
        None => (None, None)
    };

    let full_key_provided = asset_pair_filter.is_some() && dex_filter.is_some();

    let entry_list: Vec<AssetPairingEntry> = if full_key_provided {
        // We have the full key, so load the entry
        let (asset_x, asset_y) = asset_pair_filter.unwrap();
        let key = (asset_x, asset_y, dex_filter.unwrap());
        let entry = load_asset_pairing_entry_by_key(deps.storage, key)?;
        // Add the result to a vec
        vec![entry]
    } else if let Some((asset_x, asset_y)) = asset_pair_filter {
        let start_bound = page_token.map(|(_, _, dex)| Bound::exclusive(dex));

        // We can use the prefix to load all the entries for the asset pair
        let res: Result<Vec<(DexName, Vec<CompoundPoolId>)>, _> = ASSET_PAIRS
            .prefix((asset_x.clone(), asset_y.clone()))
            .range(deps.storage, start_bound, None, Order::Ascending)
            .take(page_size)
            .collect();

        // Re add the key prefix, since only the dex is returned as a key
        let matched: Vec<AssetPairingEntry> = res?.into_iter()
            .map(|(dex, ids)| ((asset_x.clone(), asset_y.clone(), dex), ids))
            .collect();

        matched
    } else {
        let start_bound = page_token.map(Bound::exclusive);

        // We have no filter, so load all the entries
        let res: Result<Vec<AssetPairingEntry>, _> = ASSET_PAIRS
            .range(deps.storage, start_bound, None, Order::Ascending)
            .filter(|e| {
                let (_, _, dex) = &e.as_ref().unwrap().0;
                dex_filter.as_ref().map_or(true, |f| f == dex)
            })
            .collect();
        res?
    };


    to_binary(&PoolIdListResponse {
        pools: entry_list,
    })
}


// let res: Result<Vec<(UniqueId, PoolMetadata)>, _> = POOL_METADATA
//     // If the asset_pair_filter is provided, we must use that prefix...
//     .range(deps.storage, start_bound, None, Order::Ascending)
//     .filter(|e| {
//         let pool_type = &e.as_ref().unwrap().1.pool_type;
//         pool_type_filter.as_ref().map_or(true, |f| f == pool_type)
//     })
//     .take(page_size)
//     .collect();

/// Query the pool ids based on the actual keys
pub fn query_asset_pairs_ids(deps: Deps, keys: Vec<AssetPairingKey>) -> StdResult<Binary> {
    let mut entries: Vec<AssetPairingEntry> = vec![];
    for key in keys.into_iter() {
        let entry = load_asset_pairing_entry_by_key(deps.storage, key)?;

        entries.push(entry);
    }

    to_binary(&PoolsResponse { pools: entries })
}

fn load_asset_pairing_entry_by_key(storage: &dyn Storage, key: AssetPairingKey) -> StdResult<AssetPairingEntry> {
    let matched = ASSET_PAIRS.load(storage, key.clone())?;
    Ok((key, matched))
}
