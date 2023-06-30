use cw_orch::prelude::*;

use abstract_core::ans_host::*;
use abstract_core::objects::pool_id::{PoolAddressBase, UncheckedPoolAddress};
use abstract_core::objects::{AnsEntryConvertor, PoolMetadata, PoolReference, UniquePoolId};
use abstract_interface::{AbstractInterfaceError, AnsHost};

use cw_orch::state::ChainState;

use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub fn update_pools(ans_host: &AnsHost<Daemon>) -> Result<(), AbstractInterfaceError> {
    let chain_name = &ans_host.get_chain().state().chain_data.chain_name;
    let chain_id = ans_host.get_chain().state().chain_data.chain_id.to_string();

    let (scraped_entries, _dexes) = get_scraped_entries(chain_name, &chain_id)?;
    let on_chain_entries = get_on_chain_entries(ans_host)?;

    let union_keys = get_union_keys(&scraped_entries, &on_chain_entries);

    let (pools_to_remove, pools_to_add) =
        get_pools_changes(ans_host, union_keys, &scraped_entries, &on_chain_entries);

    println!("Removing {} pools", pools_to_remove.len());
    println!("Removing pools: {:?}", pools_to_remove);
    println!("Adding {} pools", pools_to_add.len());
    println!("Adding pools: {:?}", pools_to_add);

    // add the pools
    ans_host.execute_chunked(&pools_to_add.into_iter().collect::<Vec<_>>(), 25, |chunk| {
        ExecuteMsg::UpdatePools {
            to_add: chunk.to_vec(),
            to_remove: vec![],
        }
    })?;

    // remove the pools
    ans_host.execute_chunked(
        &pools_to_remove.into_iter().collect::<Vec<_>>(),
        25,
        |chunk| ExecuteMsg::UpdatePools {
            to_add: vec![],
            to_remove: chunk.to_vec(),
        },
    )?;

    Ok(())
}

pub type ScrapedEntries = (
    HashMap<PoolAddressBase<std::string::String>, PoolMetadata>,
    HashSet<String>,
);

fn get_scraped_entries(
    chain_name: &String,
    chain_id: &String,
) -> Result<ScrapedEntries, AbstractInterfaceError> {
    let raw_scraped_entries = crate::get_scraped_json_data("pools");
    println!(
        "scraped_entries: {:?}",
        raw_scraped_entries[chain_name][chain_id]
    );

    let binding = raw_scraped_entries[chain_name][chain_id].clone();
    let parsed_scraped_entries: &Vec<Value> = binding.as_array().unwrap();
    let mut dexes_to_register: HashSet<String> = HashSet::new();

    let scraped_entries_vec: Vec<(UncheckedPoolAddress, PoolMetadata)> = parsed_scraped_entries
        .iter()
        .map(|value| {
            let pool: (UncheckedPoolAddress, PoolMetadata) =
                serde_json::from_value(value.clone()).unwrap();

            dexes_to_register.insert(pool.1.dex.clone());

            pool
        })
        .collect();

    Ok((scraped_entries_vec.into_iter().collect(), dexes_to_register))
}

fn get_on_chain_entries(
    ans_host: &AnsHost<Daemon>,
) -> Result<HashMap<UniquePoolId, PoolMetadata>, AbstractInterfaceError> {
    let mut on_chain_entries = HashMap::new();
    let mut last_pool = None;
    loop {
        let PoolMetadataListResponse { metadatas } =
            ans_host.pool_metadata_list(None, None, last_pool)?;
        if metadatas.is_empty() {
            break;
        }
        last_pool = metadatas.last().map(|l| l.0);
        on_chain_entries.extend(metadatas);
    }

    Ok(on_chain_entries)
}
//
fn get_union_keys<'a>(
    scraped_entries: &'a HashMap<PoolAddressBase<std::string::String>, PoolMetadata>,
    on_chain_entries: &'a HashMap<UniquePoolId, PoolMetadata>,
) -> Vec<&'a PoolMetadata> {
    let on_chain_binding = on_chain_entries.values().collect::<HashSet<_>>();
    let scraped_binding = scraped_entries.values().collect::<HashSet<_>>();

    on_chain_binding.union(&scraped_binding).cloned().collect()
}

fn get_pools_changes(
    ans_host: &AnsHost<Daemon>,
    union: Vec<&PoolMetadata>,
    scraped_entries: &HashMap<PoolAddressBase<std::string::String>, PoolMetadata>,
    _on_chain_entries: &HashMap<UniquePoolId, PoolMetadata>,
) -> (
    HashSet<UniquePoolId>,
    HashSet<(PoolAddressBase<String>, PoolMetadata)>,
) {
    // first check the dexes that need to be added
    let registered_dexes = ans_host.registered_dexes().unwrap().dexes;
    let mut dexes = HashSet::new();

    for dex in scraped_entries.values().map(|v| v.dex.clone()) {
        if dexes.contains(&dex) {
            continue;
        }
        dexes.insert(dex);
    }

    let dexes_to_register: Vec<String> = dexes
        .into_iter()
        .filter(|i| !registered_dexes.contains(i) && !i.is_empty())
        .collect();

    if !dexes_to_register.is_empty() {
        ans_host.update_dexes(dexes_to_register, vec![]).unwrap();
    }

    let pools_to_remove = HashSet::<UniquePoolId>::new();
    let mut pools_to_add = HashSet::<(PoolAddressBase<String>, PoolMetadata)>::new();

    for entry in union {
        // Find if any of the on-chain pools match the address of the scraped pool. If not it needs to be added
        // Create a pair for this pool and check for it's existence
        let pair = AnsEntryConvertor::new(entry.clone())
            .dex_asset_pairing()
            .unwrap();
        let pools = ans_host.pools(vec![pair]);
        for scraped_key in scraped_entries.keys() {
            let Ok(pools_resp) = &pools else {
                    let val = scraped_entries.get(scraped_key).unwrap();
                    pools_to_add.insert((scraped_key.clone(), val.to_owned()));
                    continue;
                };

            let pools: Vec<PoolReference> = pools_resp
                .clone()
                .pools
                .into_iter()
                .flat_map(|(_, reference)| reference)
                .collect();
            if pools
                .iter()
                .any(|p| UncheckedPoolAddress::from(&p.pool_address) == *scraped_key)
            {
                continue;
            }
            // else the value is also missing
            let val = scraped_entries.get(scraped_key).unwrap();
            pools_to_add.insert((scraped_key.clone(), val.to_owned()));
        }

        // Now find any pools that are on-chain but not in the scraped data. If so they need to be removed.
        // We do this by finding the pool address of the on-chain pool and checking if it exists in the scraped data
        // for pool in pools {
        //     if on_chain_entries.has(pool.unique_id == entry.unique_id {
        //         continue;
        //     }
        //     pools_to_remove.push(**entry)
        // }
    }
    (pools_to_remove, pools_to_add)
}
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
