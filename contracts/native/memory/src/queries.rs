use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, Record, StdResult};

use abstract_os::memory::state::{dex_pools, DexPoolData};
use abstract_os::memory::{DexPoolListResponse, DexPoolsResponse};
use abstract_os::objects::pool_id::PoolId;
use abstract_os::objects::DexPoolEntry;
use abstract_os::{
    memory::{
        state::{ASSET_ADDRESSES, CHANNELS, CONTRACT_ADDRESSES},
        AssetListResponse, AssetsResponse, ChannelListResponse, ChannelsResponse,
        ContractListResponse, ContractsResponse,
    },
    objects::{AssetEntry, ChannelEntry, ContractEntry},
};
use cw_asset::AssetInfo;
use cw_storage_plus::Bound;
use abstract_os::objects::pool_info::UncheckedPool;

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

pub fn query_dex_pools(deps: Deps, _env: Env, dex: String) -> StdResult<Binary> {
    let res: Result<Vec<Record<DexPoolData>>, _> = dex_pools()
        .idx
        .dex
        .prefix(dex)
        .range_raw(deps.storage, None, None, Order::Ascending)
        .collect();

    let pools: Vec<UncheckedPool> = res?.into_iter().map(|(_, data)| data.info).collect();

    to_binary(&DexPoolsResponse { pools })
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

// pub fn query_dex_pool_list(
//     deps: Deps,
//     last_pool: Option<DexPoolEntry>,
//     limit: Option<u8>,
// ) -> StdResult<Binary> {
//     let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
//     let start_bound = last_pool.map(Bound::exclusive);
//
//     let pairs_res: Result<Vec<(DexPoolEntry, PoolId)>, _> = dex_pools()
//         .range(deps.storage, start_bound, None, Order::Ascending)
//         .take(limit)
//         .collect();
//     to_binary(&DexPoolListResponse { pairs: pairs_res? })
// }
