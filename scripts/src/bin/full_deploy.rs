use boot_core::networks::{ChainInfo, NetworkInfo, NetworkKind};
use boot_core::prelude::*;
use clap::Parser;
use semver::Version;
use std::sync::Arc;
use tokio::runtime::Runtime;

use abstract_boot::{Abstract, OS};

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn full_deploy(network: NetworkInfo) -> anyhow::Result<()> {
    let abstract_os_version: Version = ABSTRACT_VERSION.parse().unwrap();

    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    // log::info!("Your balance is: {}", );

    let mut os_core = OS::new(chain.clone(), None);

    let mut deployment = Abstract::new(chain, abstract_os_version);

    deployment.deploy(&mut os_core)?;
    //
    // let _dex = DexApi::new("dex", chain);
    //
    // deployment.deploy_modules()?;

    let ans_host = deployment.ans_host;
    ans_host.update_all()?;

    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_id: String,
}

use boot_core::networks;

pub const INJECTIVE_CHAIN: ChainInfo = ChainInfo {
    chain_id: "injective",
    pub_address_prefix: "inj",
    coin_type: 60u32,
};

// https://testnet.status.injective.network/
pub const INJECTIVE_888: NetworkInfo = NetworkInfo {
    kind: NetworkKind::Testnet,
    id: "injective-888",
    gas_denom: "inj",
    gas_price: 0.025,
    grpc_urls: &["https://testnet.grpc.injective.network:443"],
    chain_info: INJECTIVE_CHAIN,
    lcd_url: None,
    fcd_url: None,
};

pub const KUJIRA_CHAIN: ChainInfo = ChainInfo {
    chain_id: "kujira",
    pub_address_prefix: "kujira",
    coin_type: 118u32,
};

pub const HARPOON_4: NetworkInfo = NetworkInfo {
    kind: NetworkKind::Testnet,
    id: "harpoon-4",
    gas_denom: "ukuji",
    gas_price: 0.025,
    grpc_urls: &["https://kujira-testnet-grpc.polkachu.com:11890"],
    chain_info: KUJIRA_CHAIN,
    lcd_url: None,
    fcd_url: None,
};

pub fn parse_network(net_id: &str) -> NetworkInfo {
    match net_id {
        "uni-5" => networks::UNI_5,
        "juno-1" => networks::JUNO_1,
        "pisco-1" => networks::terra::PISCO_1,
        "injective-888" => INJECTIVE_888,
        "harpoon-4" => HARPOON_4,
        _ => panic!("unexpected network"),
    }
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let network = parse_network(&args.network_id);

    if let Err(ref err) = full_deploy(network) {
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
