use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult, Storage};

use abstract_os::ans_host::state::{ASSET_PAIRINGS, POOL_METADATA};
use abstract_os::ans_host::{
    AssetPairingFilter, AssetPairingMapEntry, PoolIdListResponse, PoolMetadataFilter,
    PoolMetadataListResponse, PoolMetadataMapEntry, PoolMetadatasResponse, PoolsResponse,
    RegisteredDexesResponse,
};
use abstract_os::dex::DexName;
use abstract_os::objects::pool_info::PoolMetadata;
use abstract_os::objects::pool_reference::PoolReference;
use abstract_os::objects::{DexAssetPairing, UniquePoolId};
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

pub(crate) const DEFAULT_LIMIT: u8 = 15;
pub(crate) const MAX_LIMIT: u8 = 25;

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

pub fn list_pool_entries(
    deps: Deps,
    filter: Option<AssetPairingFilter>,
    page_token: Option<DexAssetPairing>,
    page_size: Option<u8>,
) -> StdResult<Binary> {
    let page_size = page_size.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let (asset_pair_filter, dex_filter) = match filter {
        Some(AssetPairingFilter { asset_pair, dex }) => (asset_pair, dex),
        None => (None, None),
    };

    let full_key_provided = asset_pair_filter.is_some() && dex_filter.is_some();

    let entry_list: Vec<AssetPairingMapEntry> = if full_key_provided {
        // We have the full key, so load the entry
        let (asset_x, asset_y) = asset_pair_filter.unwrap();
        let key = DexAssetPairing::new(&asset_x, &asset_y, &dex_filter.unwrap());
        let entry = load_asset_pairing_entry(deps.storage, key)?;
        // Add the result to a vec
        vec![entry]
    } else if let Some((asset_x, asset_y)) = asset_pair_filter {
        let start_bound = page_token.map(|pairing| Bound::exclusive(pairing.dex()));

        // We can use the prefix to load all the entries for the asset pair
        let res: Result<Vec<(DexName, Vec<PoolReference>)>, _> = ASSET_PAIRINGS
            .prefix((asset_x.clone(), asset_y.clone()))
            .range(deps.storage, start_bound, None, Order::Ascending)
            .take(page_size)
            .collect();

        // Re add the key prefix, since only the dex is returned as a key
        let matched: Vec<AssetPairingMapEntry> = res?
            .into_iter()
            .map(|(dex, ids)| (DexAssetPairing::new(&asset_x, &asset_y, &dex), ids))
            .collect();

        matched
    } else {
        let start_bound: Option<Bound<DexAssetPairing>> = page_token.map(Bound::exclusive);

        // We have no filter, so load all the entries
        let res: Result<Vec<AssetPairingMapEntry>, _> = ASSET_PAIRINGS
            .range(deps.storage, start_bound, None, Order::Ascending)
            .filter(|e| {
                let pairing = &e.as_ref().unwrap().0;
                dex_filter.as_ref().map_or(true, |f| f == pairing.dex())
            })
            // TODO: is this necessary?
            .map(|e| e.map(|(k, v)| (k, v)))
            .collect();
        res?
    };

    to_binary(&PoolIdListResponse { pools: entry_list })
}

/// Query the pool ids based on the actual keys
pub fn query_pool_entries(deps: Deps, keys: Vec<DexAssetPairing>) -> StdResult<Binary> {
    let mut entries: Vec<AssetPairingMapEntry> = vec![];
    for key in keys.into_iter() {
        let entry = load_asset_pairing_entry(deps.storage, key)?;

        entries.push(entry);
    }

    to_binary(&PoolsResponse { pools: entries })
}

/// Loads a given key from the asset pairings store and returns the ENTRY
fn load_asset_pairing_entry(
    storage: &dyn Storage,
    key: DexAssetPairing,
) -> StdResult<AssetPairingMapEntry> {
    let value = ASSET_PAIRINGS.load(storage, key.clone())?;
    Ok((key, value))
}

pub fn query_pool_metadatas(deps: Deps, keys: Vec<UniquePoolId>) -> StdResult<Binary> {
    let mut entries: Vec<PoolMetadataMapEntry> = vec![];
    for key in keys.into_iter() {
        let entry = load_pool_metadata_entry(deps.storage, key)?;

        entries.push(entry);
    }

    to_binary(&PoolMetadatasResponse { metadatas: entries })
}

pub fn list_pool_metadata_entries(
    deps: Deps,
    filter: Option<PoolMetadataFilter>,
    page_token: Option<UniquePoolId>,
    page_size: Option<u8>,
) -> StdResult<Binary> {
    let page_size = page_size.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = page_token.map(Bound::exclusive);

    let pool_type_filter = match filter {
        Some(PoolMetadataFilter { pool_type }) => pool_type,
        None => None,
    };

    let res: Result<Vec<(UniquePoolId, PoolMetadata)>, _> = POOL_METADATA
        // If the asset_pair_filter is provided, we must use that prefix...
        .range(deps.storage, start_bound, None, Order::Ascending)
        .filter(|e| {
            let pool_type = &e.as_ref().unwrap().1.pool_type;
            pool_type_filter.as_ref().map_or(true, |f| f == pool_type)
        })
        .take(page_size)
        .map(|e| e.map(|(k, v)| (k, v)))
        .collect();

    to_binary(&PoolMetadataListResponse { metadatas: res? })
}

/// Loads a given key from the asset pairings store and returns the ENTRY
fn load_pool_metadata_entry(
    storage: &dyn Storage,
    key: UniquePoolId,
) -> StdResult<PoolMetadataMapEntry> {
    let value = POOL_METADATA.load(storage, key)?;
    Ok((key, value))
}
#[cfg(test)]
mod test {
    use abstract_os::ans_host::{InstantiateMsg, QueryMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, DepsMut};

    use crate::contract;
    use crate::contract::{instantiate, AnsHostResult};
    use crate::error::AnsHostError;

    use super::*;

    type AnsHostTestResult = Result<(), AnsHostError>;

    const TEST_CREATOR: &str = "creator";

    fn mock_init(mut deps: DepsMut) -> AnsHostResult {
        let info = mock_info(TEST_CREATOR, &[]);

        instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})
    }

    fn query_helper(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
        let res = contract::query(deps, mock_env(), msg)?;
        Ok(res)
    }

    fn query_asset_list_msg(token: String, size: usize) -> QueryMsg {
        let msg = QueryMsg::AssetList {
            page_token: (Some(token.to_string())),
            page_size: (Some(size as u8)),
        };
        msg
    }

    mod test_query_responses {
        use abstract_os::objects::UncheckedContractEntry;

        use super::*;
        use cw_asset::AssetInfoUnchecked;
        use speculoos::assert_that;
        #[test]
        fn test_query_assets() -> AnsHostTestResult {
            // arrange mocks
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let api = deps.api;

            // create test query data
            let test_assets: Vec<(String, AssetInfoUnchecked)> = vec![
                (
                    "test1".to_string(),
                    AssetInfoUnchecked::native("1234".to_string()),
                ),
                (
                    "test2".to_string(),
                    AssetInfoUnchecked::native("5678".to_string()),
                ),
            ];
            for (test_asset_name, test_asset_info) in test_assets.clone().into_iter() {
                let insert = |_| -> StdResult<AssetInfo> { test_asset_info.check(&api, None) };
                ASSET_ADDRESSES.update(&mut deps.storage, test_asset_name.into(), insert)?;
            }

            // create msg
            let msg = QueryMsg::Assets {
                names: vec!["test1".to_string(), "test2".to_string()],
            };
            // send query message
            let res: AssetsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // Stage data for equality test
            let expected = abstract_os::ans_host::AssetsResponse {
                assets: test_assets
                    .iter()
                    .map(|item| {
                        (
                            item.0.clone().into(),
                            item.1.clone().check(&api, None).unwrap().into(),
                        )
                    })
                    .collect(),
            };

            // Assert
            assert_that!(&res).is_equal_to(&expected);

            Ok(())
        }

        #[test]
        fn test_query_contract() -> AnsHostTestResult {
            // arrange mocks
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            // create test query data
            let to_add: Vec<(UncheckedContractEntry, String)> = vec![(
                UncheckedContractEntry {
                    protocol: "foo".to_string().to_ascii_lowercase(),
                    contract: "1234".to_string().to_ascii_lowercase(),
                },
                "1234".to_string(),
            )];

            for (key, new_address) in to_add.into_iter() {
                let key = key.check();
                let addr = deps.as_ref().api.addr_validate(&new_address)?;
                let insert = |_| -> StdResult<Addr> { Ok(addr) };
                CONTRACT_ADDRESSES.update(&mut deps.storage, key, insert)?;
            }

            // create msg
            let msg = QueryMsg::Contracts {
                names: vec![
                    (ContractEntry {
                        protocol: "foo".to_string().to_ascii_lowercase(),
                        contract: "1234".to_string().to_ascii_lowercase(),
                    }),
                ],
            };
            // send query message
            let res: ContractsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // Stage data for equality test
            let expected = abstract_os::ans_host::ContractsResponse {
                contracts: vec![(
                    ContractEntry {
                        protocol: "foo".to_string().to_ascii_lowercase(),
                        contract: "1234".to_string().to_ascii_lowercase(),
                    },
                    "1234".to_string(),
                )],
            };

            // Assert
            assert_that!(&res).is_equal_to(&expected);

            Ok(())
        }

        #[test]
        fn test_query_channels() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            // create test query data
            let to_add: Vec<(ChannelEntry, String)> = vec![(
                ChannelEntry {
                    connected_chain: "test1".to_string().to_ascii_lowercase(),
                    protocol: "1234".to_string().to_ascii_lowercase(),
                },
                "1234".to_string(),
            )];
            for (key, new_channel) in to_add.into_iter() {
                // Update function for new or existing keys
                let insert = |_| -> StdResult<String> { Ok(new_channel) };
                CHANNELS.update(&mut deps.storage, key, insert)?;
            }

            // create msg
            let msg = QueryMsg::Channels {
                names: vec![
                    (ChannelEntry {
                        connected_chain: "test1".to_string().to_ascii_lowercase(),
                        protocol: "1234".to_string().to_ascii_lowercase(),
                    }),
                ],
            };
            // send query message
            let res: ChannelsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // Stage data for equality test
            let expected = abstract_os::ans_host::ChannelsResponse {
                channels: vec![(
                    ChannelEntry {
                        connected_chain: "test1".to_string(),
                        protocol: "1234".to_string(),
                    },
                    "1234".to_string(),
                )],
            };
            // Assert
            assert_that!(&res).is_equal_to(&expected);

            Ok(())
        }

        #[test]
        fn test_query_asset_list() -> AnsHostTestResult {
            // arrange mocks
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let api = deps.api;

            // create test query data
            let to_add: Vec<(String, AssetInfoUnchecked)> = vec![
                (
                    "bar".to_string(),
                    AssetInfoUnchecked::native("1234".to_string()),
                ),
                (
                    "foo".to_string(),
                    AssetInfoUnchecked::native("5678".to_string()),
                ),
            ];
            for (test_asset_name, test_asset_info) in to_add.clone().into_iter() {
                let insert = |_| -> StdResult<AssetInfo> { test_asset_info.check(&api, None) };
                ASSET_ADDRESSES.update(&mut deps.storage, test_asset_name.into(), insert)?;
            }
            // create second entry
            let to_add1: Vec<(String, AssetInfoUnchecked)> = vec![(
                "foobar".to_string(),
                AssetInfoUnchecked::native("1234".to_string()),
            )];
            for (test_asset_name, test_asset_info) in to_add1.clone().into_iter() {
                let insert = |_| -> StdResult<AssetInfo> { test_asset_info.check(&api, None) };
                ASSET_ADDRESSES.update(&mut deps.storage, test_asset_name.into(), insert)?;
            }

            // create msgs
            let msg = query_asset_list_msg("".to_string(), 2);
            let res: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // limit response to 1 result
            let msg = query_asset_list_msg("".to_string(), 1);
            let res_singular: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // results after specified entry
            let msg = query_asset_list_msg("foo".to_string(), 2);
            let res_of_foobar: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // Stage data for equality test
            let expected = abstract_os::ans_host::AssetListResponse {
                assets: to_add
                    .iter()
                    .map(|item| {
                        (
                            item.0.clone().into(),
                            item.1.clone().check(&api, None).unwrap().into(),
                        )
                    })
                    .collect(),
            };

            let expected_of_one = abstract_os::ans_host::AssetListResponse {
                assets: vec![(
                    "bar".to_string().into(),
                    AssetInfoUnchecked::native("1234".to_string()).check(&api, None)?,
                )],
            };
            let expected_foobar = abstract_os::ans_host::AssetListResponse {
                assets: vec![(
                    "foobar".to_string().into(),
                    AssetInfoUnchecked::native("1234".to_string()).check(&api, None)?,
                )],
            };

            // Assert
            assert_that!(&res).is_equal_to(&expected);
            assert_that!(res_singular).is_equal_to(expected_of_one);
            assert_that!(&res_of_foobar).is_equal_to(&expected_foobar);
            assert_that!(&res).is_not_equal_to(&expected_foobar);

            Ok(())
        }

        #[test]
        fn test_query_contract_list() -> AnsHostTestResult {
            // arrange mocks
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            // create test query data
            let to_add: Vec<(ContractEntry, String)> = vec![(
                ContractEntry {
                    protocol: "foo".to_string().to_ascii_lowercase(),
                    contract: "1234".to_string().to_ascii_lowercase(),
                },
                "1234".to_string(),
            )];
            for (key, new_address) in to_add.into_iter() {
                let addr = deps.as_ref().api.addr_validate(&new_address)?;
                let insert = |_| -> StdResult<Addr> { Ok(addr) };
                CONTRACT_ADDRESSES.update(&mut deps.storage, key, insert)?;
            }
            // create second entry
            let to_add1: Vec<(ContractEntry, String)> = vec![(
                ContractEntry {
                    protocol: "bar".to_string().to_ascii_lowercase(),
                    contract: "1234".to_string().to_ascii_lowercase(),
                },
                "1234".to_string(),
            )];
            for (key, new_address) in to_add1.into_iter() {
                let addr = deps.as_ref().api.addr_validate(&new_address)?;
                let insert = |_| -> StdResult<Addr> { Ok(addr) };
                CONTRACT_ADDRESSES.update(&mut deps.storage, key, insert)?;
            }

            // create msgs
            let msg = QueryMsg::ContractList {
                page_token: None,
                page_size: Some(2 as u8),
            };
            let res: ContractListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            let msg = QueryMsg::ContractList {
                page_token: Some(ContractEntry {
                    protocol: "bar".to_string().to_ascii_lowercase(),
                    contract: "1234".to_string().to_ascii_lowercase(),
                }),
                page_size: Some(2 as u8),
            };
            let res_of_bar_as_token: ContractListResponse =
                from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // Stage data for equality test
            let expected = abstract_os::ans_host::ContractListResponse {
                contracts: vec![
                    (
                        ContractEntry {
                            protocol: "bar".to_string().to_ascii_lowercase(),
                            contract: "1234".to_string().to_ascii_lowercase(),
                        },
                        "1234".to_string(),
                    ),
                    (
                        ContractEntry {
                            protocol: "foo".to_string().to_ascii_lowercase(),
                            contract: "1234".to_string().to_ascii_lowercase(),
                        },
                        "1234".to_string(),
                    ),
                ],
            };

            let expected_foo = abstract_os::ans_host::ContractListResponse {
                contracts: vec![(
                    ContractEntry {
                        protocol: "foo".to_string().to_ascii_lowercase(),
                        contract: "1234".to_string().to_ascii_lowercase(),
                    },
                    "1234".to_string(),
                )],
            };

            // Assert
            assert_that!(&res).is_equal_to(&expected);
            assert_that!(&res_of_bar_as_token).is_equal_to(&expected_foo);

            Ok(())
        }
        #[test]
        fn test_query_channel_list() -> AnsHostTestResult {
            // arrange mocks
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            // create test query data
            let to_add: Vec<(ChannelEntry, String)> = vec![
                (
                    ChannelEntry {
                        connected_chain: "foo".to_string(),
                        protocol: "foo".to_string(),
                    },
                    "foo".to_string(),
                ),
                (
                    ChannelEntry {
                        connected_chain: "bar".to_string(),
                        protocol: "bar".to_string(),
                    },
                    "bar".to_string(),
                ),
            ];
            for (key, new_channel) in to_add.into_iter() {
                // Update function for new or existing keys
                let insert = |_| -> StdResult<String> { Ok(new_channel) };
                CHANNELS.update(&mut deps.storage, key, insert)?;
            }
            // create second entry
            let to_add1: Vec<(ChannelEntry, String)> = vec![(
                ChannelEntry {
                    connected_chain: "foobar".to_string(),
                    protocol: "foobar".to_string(),
                },
                "foobar".to_string(),
            )];
            for (key, new_channel) in to_add1.into_iter() {
                // Update function for new or existing keys
                let insert = |_| -> StdResult<String> { Ok(new_channel) };
                CHANNELS.update(&mut deps.storage, key, insert)?;
            }

            // create msgs
            // No token filter - should return up to `page_size` entries
            let msg = QueryMsg::ChannelList {
                page_token: None,
                page_size: Some(3 as u8),
            };
            let res_all = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // Filter for entries after `Foo` - Alphabetically
            let msg = QueryMsg::ChannelList {
                page_token: Some(ChannelEntry {
                    connected_chain: "foo".to_string(),
                    protocol: "foo".to_string(),
                }),
                page_size: Some(1 as u8),
            };
            let res_foobar = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // Return first entry - Alphabetically
            let msg = QueryMsg::ChannelList {
                page_token: None,
                page_size: Some(1 as u8),
            };
            let res_bar = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // Stage data for equality test
            // Return all
            let expected_all = abstract_os::ans_host::ChannelListResponse {
                channels: vec![
                    (
                        ChannelEntry {
                            connected_chain: "bar".to_string(),
                            protocol: "bar".to_string(),
                        },
                        "bar".to_string(),
                    ),
                    (
                        ChannelEntry {
                            connected_chain: "foo".to_string(),
                            protocol: "foo".to_string(),
                        },
                        "foo".to_string(),
                    ),
                    (
                        ChannelEntry {
                            connected_chain: "foobar".to_string(),
                            protocol: "foobar".to_string(),
                        },
                        "foobar".to_string(),
                    ),
                ],
            };
            // Filter from `Foo`
            let expected_foobar = abstract_os::ans_host::ChannelListResponse {
                channels: vec![(
                    ChannelEntry {
                        connected_chain: "foobar".to_string(),
                        protocol: "foobar".to_string(),
                    },
                    "foobar".to_string(),
                )],
            };
            // Return first entry (alphabetically)
            let expected_bar = abstract_os::ans_host::ChannelListResponse {
                channels: vec![(
                    ChannelEntry {
                        connected_chain: "bar".to_string(),
                        protocol: "bar".to_string(),
                    },
                    "bar".to_string(),
                )],
            };
            // Assert
            assert_that!(&res_all).is_equal_to(expected_all);
            assert_that!(&res_foobar).is_equal_to(expected_foobar);
            assert_that!(&res_bar).is_equal_to(expected_bar);

            Ok(())
        }

        #[test]
        fn test_query_registered_dexes() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            // Create test data
            let to_add: Vec<String> = vec!["test_dex1".to_string(), "test_dex2".to_string()];
            for _dex in to_add.clone() {
                let register_dex = |mut dexes: Vec<String>| -> StdResult<Vec<String>> {
                    for _dex in to_add.clone() {
                        if !dexes.contains(&_dex) {
                            dexes.push(_dex.to_ascii_lowercase());
                        }
                    }
                    Ok(dexes)
                };
                REGISTERED_DEXES.update(&mut deps.storage, register_dex)?;
            }
            // create msg
            let msg = QueryMsg::RegisteredDexes {};
            // deserialize response
            let res = from_binary(&query_helper(deps.as_ref(), msg)?)?;

            // comparisons
            let expected = RegisteredDexesResponse {
                dexes: vec!["test_dex1".to_string(), "test_dex2".to_string()],
            };
            let not_expected = RegisteredDexesResponse {
                dexes: vec!["test_dex3".to_string(), "test_dex2".to_string()],
            };
            // tests
            assert_that!(&res).is_equal_to(expected);
            assert_that!(&res).is_not_equal_to(not_expected);
            Ok(())
        }
    }
}
