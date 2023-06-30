use crate::EntryDif;
use cw_asset::AssetInfoBase;

use cw_orch::prelude::*;

use abstract_core::ans_host::*;
use abstract_interface::{AbstractInterfaceError, AnsHost};

use serde_json::{from_value, Value};
use std::collections::HashMap;

/*
pub fn update_assets(ans_host: &AnsHost<Daemon>) -> Result<(), AbstractInterfaceError> {
    let chain_name = &ans_host.get_chain().state().chain_data.chain_name;
    let chain_id = ans_host.get_chain().state().chain_data.chain_id.to_string();

    let scraped_entries = get_scraped_entries(chain_name, &chain_id)?;
    let on_chain_entries = get_on_chain_entries(ans_host)?;

    let union_keys = get_union_keys(&scraped_entries, &on_chain_entries);

    let (assets_to_remove, assets_to_add) =
        get_assets_changes(&union_keys, &scraped_entries, &on_chain_entries);

    println!("Removing {} assets", assets_to_remove.len());
    println!("Removing assets: {:?}", assets_to_remove);
    println!("Adding {} assets", assets_to_add.len());
    println!("Adding assets: {:?}", assets_to_add);

    // add the assets
    ans_host.execute_chunked(&assets_to_add, 25, |chunk| {
        ExecuteMsg::UpdateAssetAddresses {
            to_add: chunk.to_vec(),
            to_remove: vec![],
        }
    })?;

    // remove the assets
    ans_host.execute_chunked(&assets_to_remove, 25, |chunk| {
        ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![],
            to_remove: chunk.to_vec(),
        }
    })?;

    Ok(())
}
*/

pub fn get_scraped_entries(
    chain_name: &String,
    chain_id: &String,
) -> Result<HashMap<String, AssetInfoBase<String>>, AbstractInterfaceError> {
    let raw_scraped_entries = crate::get_scraped_json_data("assets");
    println!(
        "scraped_entries: {:?}",
        raw_scraped_entries[chain_name][chain_id]
    );

    let parsed_scraped_entries: Vec<Vec<Value>> =
        from_value(raw_scraped_entries[chain_name][chain_id].clone()).unwrap();

    let scraped_entries_vec: Vec<(String, AssetInfoBase<String>)> = parsed_scraped_entries
        .into_iter()
        .map(|v| {
            let asset_info: AssetInfoBase<String> = from_value(v[1].clone()).unwrap();
            (v[0].as_str().unwrap().to_owned(), asset_info)
        })
        .collect();

    Ok(scraped_entries_vec.into_iter().collect())
}

pub fn get_on_chain_entries(
    ans_host: &AnsHost<Daemon>,
) -> Result<HashMap<String, AssetInfoBase<String>>, AbstractInterfaceError> {
    let mut on_chain_entries = HashMap::new();
    let mut last_asset = None;
    loop {
        let AssetListResponse { assets } = ans_host.asset_list(None, None, last_asset)?;
        if assets.is_empty() {
            break;
        }
        last_asset = assets.last().map(|(entry, _)| entry.to_string());
        on_chain_entries.extend(assets.into_iter().map(|(a, b)| (a.to_string(), b.into())));
    }

    Ok(on_chain_entries)
}

pub fn update(
    ans_host: &AnsHost<Daemon>,
    diff: EntryDif<String, AssetInfoBase<String>>,
) -> Result<(), AbstractInterfaceError> {
    println!("Removing {} assets", diff.0.len());
    println!("Removing assets: {:?}", diff.0);
    println!("Adding {} assets", diff.1.len());
    println!("Adding assets: {:?}", diff.1);

    let to_add: Vec<_> = diff.1.into_iter().collect();
    let to_remove: Vec<_> = diff.0.into_iter().collect();

    // add the assets
    ans_host.execute_chunked(&to_add, 25, |chunk| ExecuteMsg::UpdateAssetAddresses {
        to_add: chunk.to_vec(),
        to_remove: vec![],
    })?;

    // remove the assets
    ans_host.execute_chunked(&to_remove, 25, |chunk| ExecuteMsg::UpdateAssetAddresses {
        to_add: vec![],
        to_remove: chunk.to_vec(),
    })?;

    Ok(())
}

// fn update_channels(ans: &AnsHost<Daemon>) -> Result<(), crate::CwOrchError> {
//     let path = env::var("ANS_HOST_CHANNELS").unwrap();
//     let file =
//         File::open(&path).unwrap_or_else(|_| panic!("file should be present at {}", &path));
//     let json: serde_json::Value = from_reader(file)?;
//     let chain_name = &ans.get_chain().state().chain_data.chain_name;
//     let chain_id = ans.get_chain().state().chain_data.chain_id.to_string();
//     let channels = json
//         .get(chain_name)
//         .unwrap()
//         .get(chain_id)
//         .ok_or_else(|| CwOrchError::StdErr("network not found".into()))?;

//     let channels = channels.as_object().unwrap();
//     let channels_to_add: Vec<(UncheckedChannelEntry, String)> = channels
//         .iter()
//         .map(|(name, value)| {
//             let id = value.as_str().unwrap().to_owned();
//             let key = UncheckedChannelEntry::try_from(name.clone()).unwrap();
//             (key, id)
//         })
//         .collect();

//     ans.execute_chunked(&channels_to_add, 25, |chunk| ExecuteMsg::UpdateChannels {
//         to_add: chunk.to_vec(),
//         to_remove: vec![],
//     })?;

//     Ok(())
// }

// fn update_contracts(ans: &AnsHost<Daemon>) -> Result<(), crate::CwOrchError> {
//     let path = env::var("ANS_HOST_CONTRACTS").unwrap();

//     let file =
//         File::open(&path).unwrap_or_else(|_| panic!("file should be present at {}", &path));
//     let json: serde_json::Value = from_reader(file)?;
//     let chain_name = &ans.get_chain().state().chain_data.chain_name;
//     let chain_id = ans.get_chain().state().chain_data.chain_id.to_string();
//     let contracts = json
//         .get(chain_name)
//         .unwrap()
//         .get(chain_id)
//         .ok_or_else(|| CwOrchError::StdErr("network not found".into()))?;

//     /*
//       [
//     [
//       {
//         "protocol": "junoswap",
//         "contract": "staking/crab,junox"
//       },
//       "juno1vhxnvu0zh6p707auf0ltq6scse3d2fxvp4804s54q45z29vtjleqghne5g"
//     ]
//     ]
//        */
//     let contracts = contracts.as_array().unwrap();
//     let contracts_to_add: Vec<(UncheckedContractEntry, String)> = contracts
//         .iter()
//         .map(|value| {
//             let contract: (UncheckedContractEntry, String) =
//                 serde_json::from_value(value.clone()).unwrap();
//             contract
//         })
//         .collect();

//     ans.execute_chunked(&contracts_to_add, 25, |chunk| {
//         ExecuteMsg::UpdateContractAddresses {
//             to_add: chunk.to_vec(),
//             to_remove: vec![],
//         }
//     })?;

//     Ok(())
// }

// fn update_pools(ans: &AnsHost<Daemon>) -> Result<(), crate::CwOrchError> {
//     let path = env::var("ANS_HOST_POOLS").unwrap();
//     let file =
//         File::open(&path).unwrap_or_else(|_| panic!("file should be present at {}", &path));
//     let json: serde_json::Value = from_reader(file)?;
//     let chain_name = &ans.get_chain().state().chain_data.chain_name;
//     let chain_id = ans.get_chain().state().chain_data.chain_id.to_string();
//     let pools = json
//         .get(chain_name)
//         .unwrap()
//         .get(chain_id)
//         .ok_or_else(|| CwOrchError::StdErr("network not found".into()))?;

//     let mut dexes_to_register: HashSet<String> = HashSet::new();

//     let pools = pools.as_array().unwrap();
//     let pools_to_add: Vec<(UncheckedPoolAddress, PoolMetadata)> = pools
//         .iter()
//         .map(|value| {
//             let pool: (UncheckedPoolAddress, PoolMetadata) =
//                 serde_json::from_value(value.clone()).unwrap();

//             dexes_to_register.insert(pool.1.dex.clone());

//             pool
//         })
//         .collect();

//     // Register the dexes
//     ans.0.execute(
//         &ExecuteMsg::UpdateDexes {
//             to_add: Vec::from_iter(dexes_to_register),
//             to_remove: vec![],
//         },
//         None,
//     )?;

//     ans.execute_chunked(&pools_to_add, 25, |chunk| ExecuteMsg::UpdatePools {
//         to_add: chunk.to_vec(),
//         to_remove: vec![],
//     })?;

//     Ok(())
// }
