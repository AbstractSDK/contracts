use cw_orch::prelude::*;

use abstract_core::ans_host::*;
use abstract_core::objects::UncheckedContractEntry;
use abstract_interface::{AbstractInterfaceError, AnsHost};

use cw_orch::state::ChainState;

use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub fn update_contracts(ans_host: &AnsHost<Daemon>) -> Result<(), AbstractInterfaceError> {
    let chain_name = &ans_host.get_chain().state().chain_data.chain_name;
    let chain_id = ans_host.get_chain().state().chain_data.chain_id.to_string();

    let scraped_entries = get_scraped_entries(chain_name, &chain_id)?;
    let on_chain_entries = get_on_chain_entries(ans_host)?;

    let union_keys = get_union_keys(&scraped_entries, &on_chain_entries);

    let (contracts_to_remove, contracts_to_add) =
        get_contracts_changes(&union_keys, &scraped_entries, &on_chain_entries);

    println!("Removing {} contracts", contracts_to_remove.len());
    println!("Removing contracts: {:?}", contracts_to_remove);
    println!("Adding {} contracts", contracts_to_add.len());
    println!("Adding contracts: {:?}", contracts_to_add);

    // add the contracts
    ans_host.execute_chunked(&contracts_to_add, 25, |chunk| {
        ExecuteMsg::UpdateContractAddresses {
            to_add: chunk.to_vec(),
            to_remove: vec![],
        }
    })?;

    // remove the contracts
    ans_host.execute_chunked(&contracts_to_remove, 25, |chunk| {
        ExecuteMsg::UpdateContractAddresses {
            to_add: vec![],
            to_remove: chunk.to_vec(),
        }
    })?;

    Ok(())
}

fn get_scraped_entries(
    chain_name: &String,
    chain_id: &String,
) -> Result<HashMap<UncheckedContractEntry, String>, AbstractInterfaceError> {
    let raw_scraped_entries = crate::get_scraped_json_data("contracts");
    println!(
        "scraped_entries: {:?}",
        raw_scraped_entries[chain_name][chain_id]
    );

    let binding = raw_scraped_entries[chain_name][chain_id].clone();
    let parsed_scraped_entries: &Vec<Value> = binding.as_array().unwrap();

    let scraped_entries_vec: Vec<(UncheckedContractEntry, String)> = parsed_scraped_entries
        .iter()
        .map(|value| {
            let contract: (UncheckedContractEntry, String) =
                serde_json::from_value(value.clone()).unwrap();
            contract
        })
        .collect();

    Ok(scraped_entries_vec.into_iter().collect())
}

fn get_on_chain_entries(
    ans_host: &AnsHost<Daemon>,
) -> Result<HashMap<UncheckedContractEntry, String>, AbstractInterfaceError> {
    let mut on_chain_entries = HashMap::new();
    let mut last_asset = None;
    loop {
        let ContractListResponse { contracts } = ans_host.contract_list(None, None, last_asset)?;
        if contracts.is_empty() {
            break;
        }
        last_asset = contracts.last().map(|l| l.0.clone());
        on_chain_entries.extend(
            contracts
                .into_iter()
                .map(|(a, b)| (a.into(), b.to_string())),
        );
    }

    Ok(on_chain_entries)
}

fn get_union_keys<'a>(
    scraped_entries: &'a HashMap<UncheckedContractEntry, String>,
    on_chain_entries: &'a HashMap<UncheckedContractEntry, String>,
) -> Vec<&'a UncheckedContractEntry> {
    let on_chain_binding = on_chain_entries.keys().collect::<HashSet<_>>();
    let scraped_binding = scraped_entries.keys().collect::<HashSet<_>>();

    on_chain_binding.union(&scraped_binding).cloned().collect()
}

fn get_contracts_changes(
    union_keys: &Vec<&UncheckedContractEntry>,
    scraped_entries: &HashMap<UncheckedContractEntry, String>,
    on_chain_entries: &HashMap<UncheckedContractEntry, String>,
) -> (
    Vec<UncheckedContractEntry>,
    Vec<(UncheckedContractEntry, String)>,
) {
    let mut contracts_to_remove: Vec<UncheckedContractEntry> = vec![];
    let mut contracts_to_add: Vec<(UncheckedContractEntry, String)> = vec![];

    for entry in union_keys {
        if !scraped_entries.contains_key(entry) {
            contracts_to_remove.push((*entry).clone())
        }

        if !on_chain_entries.contains_key(*entry) {
            let val = scraped_entries.get(*entry).unwrap();

            contracts_to_add.push(((*entry).to_owned(), val.to_owned()))
        }
    }
    return (contracts_to_remove, contracts_to_add);
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
