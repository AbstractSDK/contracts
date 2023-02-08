use abstract_boot::{Manager, OSFactory, VersionControl, OS};
use abstract_os::{MANAGER, OS_FACTORY, VERSION_CONTROL};

use boot_core::networks::{parse_network, NetworkInfo};
use boot_core::prelude::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

use abstract_os::objects::module::{ModuleInfo, ModuleVersion};
use abstract_os::version_control::ExecuteMsgFns;
use clap::Parser;
use semver::Version;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn migrate(network: NetworkInfo) -> anyhow::Result<()> {
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let abstract_os_version = Version::parse(VERSION)?;

    let mut vc = VersionControl::new(VERSION_CONTROL, chain.clone());

    let old_manager_info =
        ModuleInfo::from_id(MANAGER, ModuleVersion::Version("0.8.3".to_string()))?;

    // Remove the old module
    let old_manager_mod = vc.module(old_manager_info)?;
    vc.remove_module(old_manager_mod.info)?;
    // // register it under 0.8.0
    // vc.add_modules(vec![(
    //     ModuleInfo::from_id(MANAGER, ModuleVersion::Version("0.8.0".to_string()))?,
    //     old_manager_mod.reference,
    // )])?;

    let mut manager = Manager::new(MANAGER, chain.clone());
    manager.upload()?;

    /// Register the new one
    vc.register_cores(vec![manager.as_instance()], &abstract_os_version)?;

    Ok(())
}

// TODO: base arguments
#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_id: String,
}

//
fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let network = parse_network(&args.network_id);

    if let Err(ref err) = migrate(network) {
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
