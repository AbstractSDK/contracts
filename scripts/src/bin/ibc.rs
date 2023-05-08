use abstract_boot::{Abstract, IbcClient, OsmosisHost};
use abstract_core::{ibc_client, ibc_host, IBC_CLIENT};
use cosmwasm_std::{CosmosMsg, Empty};
use cw_orch::{
    ibc_tracker::{CwIbcContractState, IbcTracker, IbcTrackerConfigBuilder},
    networks::{
        osmosis::{self, OSMO_2},
        JUNO_1,
    },
    queriers::Bank,
    *,
};

use tokio::runtime::Handle;

const CRATE_PATH: &str = env!("CARGO_MANIFEST_DIR");
const JUNO_MNEMONIC: &str = "dilemma imitate split detect useful creek cart sort grow essence fish husband seven hollow envelope wedding host dry permit game april present panic move";
const OSMOSIS_MNEMONIC: &str = "settle gas lobster judge silk stem act shoulder pluck waste pistol word comfort require early mouse provide marine butter crowd clock tube move wool";
const JUNO: &str = "juno-1";
const OSMOSIS: &str = "osmosis-2";

pub fn script() -> anyhow::Result<()> {
    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let interchain = InterchainInfrastructure::new(
        rt.handle(),
        vec![(JUNO_1, JUNO_MNEMONIC), (OSMO_2, OSMOSIS_MNEMONIC)],
    )?;

    let juno = interchain.daemon(JUNO)?;
    let osmosis = interchain.daemon(OSMOSIS)?;

    let cw1 = Cw1::new("cw1", juno.clone());
    let host = Host::new("host", juno.clone());
    let controller = Controller::new("controller", osmosis.clone());

    // ### SETUP ###
    deploy_contracts(&juno, &osmosis)?;
    interchain
        .hermes
        .create_channel(&rt, "connection-0", "simple-ica-v2", &controller, &host);

    // wait for channel creation to complete
    std::thread::sleep(std::time::Duration::from_secs(30));

    // Track IBC on JUNO
    let juno_channel = juno.channel();
    let tracker = IbcTrackerConfigBuilder::default()
        .ibc_state(CwIbcContractState::new(
            "connection-0",
            format!("wasm.{}", host.addr_str()?),
        ))
        .build()?;
    // spawn juno logging on a different thread.
    rt.spawn(async move {
        juno_channel.cron_log(tracker).await.unwrap();
    });

    // Track IBC on OSMOSIS
    let osmosis_channel = osmosis.channel();
    let tracker = IbcTrackerConfigBuilder::default()
        .ibc_state(CwIbcContractState::new(
            "connection-0",
            format!("wasm.{}", controller.addr_str()?),
        ))
        .build()?;
    // spawn osmosis logging on a different thread.
    rt.spawn(async move {
        osmosis_channel.cron_log(tracker).await.unwrap();
    });

    Ok(())
}

fn main() {
    dotenv().ok();
    use dotenv::dotenv;

    if let Err(ref err) = script() {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));
        ::std::process::exit(1);
    }
}

fn deploy_contracts(juno: &Daemon, osmosis: &Daemon) -> anyhow::Result<()> {
    let juno_abstr = Abstract::deploy_on(juno.clone(), "1.0.0".parse().unwrap())?;

    // now deploy IBC stuff
    let client = IbcClient::new(IBC_CLIENT, juno.clone());
    let host = OsmosisHost::new("host", juno.clone());
    client.upload()?;
    host.upload()?;

    client.instantiate(
        &ibc_client::InstantiateMsg {
            ans_host_address: juno_abstr.ans_host.addr_str()?,
            chain: "juno-1".to_string(),
            version_control_address: juno_abstr.version_control.addr_str()?,
        },
        None,
        None,
    )?;

    let osmo_abstr = Abstract::deploy_on(osmosis.clone(), "1.0.0".parse().unwrap())?;

    Ok(())
}
