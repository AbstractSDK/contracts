use abstract_interface::Abstract;

use clap::Parser;
use cw_orch::{
    deploy::Deploy,
    prelude::{
        networks::{parse_network, ChainInfo},
        *,
    },
};
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const ANS_SCRAPE_URL: &str =
    "https://raw.githubusercontent.com/AbstractSDK/ans-scraper/mainline/out/";

mod ans_merge {
    use super::*;
    use abstract_core::ans_host::*;
    use abstract_interface::{AbstractInterfaceError, AnsHost};
    use cw_asset::{AssetInfo, AssetInfoBase, AssetInfoUnchecked};
    use cw_orch::state::ChainState;
    use reqwest::Client;
    use serde_json::{from_reader, from_value, Value};
    use std::{
        collections::{HashMap, HashSet},
    };
    use std::str::FromStr;

    pub fn update_assets(ans_host: &AnsHost<Daemon>) -> Result<(), AbstractInterfaceError> {
        let scraped_entries = get_scraped_json_data("assets");
        let chain_name = &ans_host.get_chain().state().chain_data.chain_name;
        let chain_id = ans_host.get_chain().state().chain_data.chain_id.to_string();

        println!("scraped_entries: {:?}", scraped_entries[chain_name][chain_id.clone()]);

        let scraped_entries: Vec<Vec<Value>> =
            from_value(scraped_entries[chain_name][chain_id].clone()).unwrap();

        let scraped_entries_vec: Vec<(String, String)> = scraped_entries.into_iter().map(|v| {
            let asset_info: AssetInfo = from_value(v[1].clone()).unwrap();
            (v[0].as_str().unwrap().to_owned(), asset_info.to_string())
        }).collect::<Vec<_>>();
            
        println!("scraped_entries: {:?}", scraped_entries_vec[0]);
        let scraped_entries = HashMap::<String, String>::from_iter(scraped_entries_vec.into_iter());

        // get all the assets
        let mut on_chain_entries = HashMap::new();
        let mut last_asset = None;
        loop {
            let AssetListResponse { assets } = ans_host.asset_list(None, None, last_asset)?;
            if assets.is_empty() {
                break;
            }
            last_asset = assets.last().map(|(entry, _)| entry.to_string());
            on_chain_entries.extend(assets.into_iter().map(|(a, b)| (a.to_string(), b)));
        }

        // Merge the keys of the two stores.
        let on_chain_binding = on_chain_entries.keys().collect::<HashSet<_>>();
        let scraped_binding = scraped_entries.keys().collect::<HashSet<_>>();
        let union_keys = on_chain_binding
            .union(&scraped_binding)
            .to_owned()
            .collect::<Vec<_>>();

        let mut assets_to_remove: Vec<String> = vec![];
        let mut assets_to_add: Vec<(String, cw_asset::AssetInfoBase<String>)> = vec![];

        for entry in union_keys {
            if !scraped_entries.contains_key(entry.as_str()) {
                // remove the key
                assets_to_remove.push(entry.to_string())
            }

            if !on_chain_entries.contains_key(*entry) {
                // add the key
                assets_to_add.push((
                    (*entry).to_owned(),
                    AssetInfoUnchecked::from_str(scraped_entries.get(*entry).unwrap()).unwrap(),
                ))
            }
        }

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

    /// get some json  
    fn get_scraped_json_data(suffix: &str) -> Value {
        let client = Client::new();
        let url = format!("{}{}.json", ANS_SCRAPE_URL, suffix);
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let response = client.get(&url).send().await.unwrap();
            let json: Value = response.json().await.unwrap();
            return json;
        })
    }
}

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn full_deploy(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    for network in networks {
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network)
            .build()?;

        let deployment = Abstract::load_from(chain)?;
        // Take the assets, contracts, and pools from resources and upload them to the ans host
        let ans_host = deployment.ans_host;
        ans_merge::update_assets(&ans_host)?;
    }
    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_ids: Vec<String>,
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let networks = args.network_ids.iter().map(|n| parse_network(n)).collect();

    if let Err(ref err) = full_deploy(networks) {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        // The backtrace is not always generated. Try to run this example
        // with `$env:RUST_BACKTRACE=1`.
        //    if let Some(backtrace) = e.backtrace() {
        //        log::debug!("backtrace: {:?}", backtrace);
        //    }

        ::std::process::exit(1);
    }
}
