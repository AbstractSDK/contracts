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

fn update_ans(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    for network in networks {
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network)
            .build()?;

        let deployment = Abstract::load_from(chain)?;
        // Take the assets, contracts, and pools from resources and upload them to the ans host
        let ans_host = deployment.ans_host;
        ans_helper::assets::update_assets(&ans_host)?;
        ans_helper::contracts::update_contracts(&ans_host)?;
        ans_helper::pools::update_pools(&ans_host)?;
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

    if let Err(ref err) = update_ans(networks) {
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

// pub fn update_assets(ans_host: &AnsHost<Daemon>) -> Result<(), AbstractInterfaceError> {
//     let scraped_entries = get_scraped_json_data("assets");
//     let chain_name = &ans_host.get_chain().state().chain_data.chain_name;
//     let chain_id = ans_host.get_chain().state().chain_data.chain_id.to_string();

//     println!("scraped_entries: {:?}", scraped_entries[chain_name][chain_id.clone()]);

//     let scraped_entries: Vec<Vec<Value>> =
//         from_value(scraped_entries[chain_name][chain_id].clone()).unwrap();

//     let scraped_entries_vec: Vec<(String, String)> = scraped_entries.into_iter().map(|v| {
//         let asset_info: AssetInfo = from_value(v[1].clone()).unwrap();
//         (v[0].as_str().unwrap().to_owned(), asset_info.to_string())
//     }).collect::<Vec<_>>();

//     println!("scraped_entries: {:?}", scraped_entries_vec[0]);
//     let scraped_entries = HashMap::<String, String>::from_iter(scraped_entries_vec.into_iter());

//     // get all the assets
//     let mut on_chain_entries = HashMap::new();
//     let mut last_asset = None;
//     loop {
//         let AssetListResponse { assets } = ans_host.asset_list(None, None, last_asset)?;
//         if assets.is_empty() {
//             break;
//         }
//         last_asset = assets.last().map(|(entry, _)| entry.to_string());
//         on_chain_entries.extend(assets.into_iter().map(|(a, b)| (a.to_string(), b)));
//     }

//     // Merge the keys of the two stores.
//     let on_chain_binding = on_chain_entries.keys().collect::<HashSet<_>>();
//     let scraped_binding = scraped_entries.keys().collect::<HashSet<_>>();
//     let union_keys = on_chain_binding
//         .union(&scraped_binding)
//         .to_owned()
//         .collect::<Vec<_>>();

//     let mut assets_to_remove: Vec<String> = vec![];
//     let mut assets_to_add: Vec<(String, cw_asset::AssetInfoBase<String>)> = vec![];

//     for entry in union_keys {
//         if !scraped_entries.contains_key(entry.as_str()) {
//             // remove the key
//             assets_to_remove.push(entry.to_string())
//         }

//         if !on_chain_entries.contains_key(*entry) {
//             // add the key
//             assets_to_add.push((
//                 (*entry).to_owned(),
//                 AssetInfoUnchecked::from_str(scraped_entries.get(*entry).unwrap()).unwrap(),
//             ))
//         }
//     }

//     println!("Removing {} assets", assets_to_remove.len());
//     println!("Removing assets: {:?}", assets_to_remove);
//     println!("Adding {} assets", assets_to_add.len());
//     println!("Adding assets: {:?}", assets_to_add);
