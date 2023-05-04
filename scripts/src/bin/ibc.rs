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
    deploy_contracts(&cw1, &host, &controller)?;
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

    // test the ica implementation
    test_ica(rt.handle().clone(), &controller, &juno)?;

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
    host.instantiate(
        &ibc_host::InstantiateMsg {
            base: ibc_host::BaseInstantiateMsg {
                ans_host_address: osmo_abstr.ans_host,
                account_factory_address: osmo_abstr.account_factory,
            },
        },
        admin,
        coins,
    );

    Ok(())
}

/// Test the cw-ica contract
fn test_ica(
    rt: Handle,
    // controller on osmosis
    controller: &Controller<Daemon>,
    juno: &Daemon,
) -> anyhow::Result<()> {
    // get the information about the remote account
    let remote_accounts: controller_msgs::ListAccountsResponse =
        controller.query(&controller_msgs::QueryMsg::ListAccounts {})?;
    assert_that!(remote_accounts.accounts.len()).is_equal_to(1);

    // get the account information
    let remote_account = remote_accounts.accounts[0].clone();
    let remote_addr = remote_account.remote_addr.unwrap();

    // send some funds to the remote account
    rt.block_on(
        juno.sender
            .bank_send(&remote_addr, vec![cosmwasm_std::coin(100u128, "ujuno")]),
    )?;

    // assert that the remote account got funds
    let balance = rt.block_on(juno.query::<Bank>().coin_balance(&remote_addr, "ujuno"))?;
    assert_that!(&balance.amount).is_equal_to(100u128.to_string());

    // burn the juno remotely
    controller.execute(
        &controller_msgs::ExecuteMsg::SendMsgs {
            channel_id: "channel-1".to_string(),
            msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
                amount: vec![cosmwasm_std::coin(100u128, "ujuno")],
            })],
            callback_id: None,
        },
        None,
    )?;

    // wait a bit
    std::thread::sleep(std::time::Duration::from_secs(30));
    // check that the balance became 0
    let balance = rt.block_on(juno.query::<Bank>().coin_balance(&remote_addr, "ujuno"))?;
    assert_that!(&balance.amount).is_equal_to(0u128.to_string());
    Ok(())
}

// Contract interface definitions

#[contract(
    controller_msgs::InstantiateMsg,
    controller_msgs::ExecuteMsg,
    controller_msgs::QueryMsg,
    Empty
)]
struct Controller;

impl<Chain: CwEnv> Controller<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let contract = Contract::new(name, chain);
        Self(contract)
    }
}

impl Uploadable for Controller<Daemon> {
    fn wasm(&self) -> <Daemon as TxHandler>::ContractSource {
        WasmPath::new(format!(
            "{CRATE_PATH}/examples/wasms/simple_ica_controller.wasm"
        ))
        .unwrap()
    }
}

#[contract(host_msgs::InstantiateMsg, Empty, host_msgs::QueryMsg, Empty)]
struct Host;
impl<Chain: CwEnv> Host<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let contract = Contract::new(name, chain);
        Self(contract)
    }
}

impl Uploadable for Host<Daemon> {
    fn wasm(&self) -> <Daemon as TxHandler>::ContractSource {
        WasmPath::new(format!("{CRATE_PATH}/examples/wasms/simple_ica_host.wasm")).unwrap()
    }
}

// just for uploading
#[contract(Empty, Empty, Empty, Empty)]
struct Cw1;
impl<Chain: CwEnv> Cw1<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let contract = Contract::new(name, chain);
        Self(contract)
    }
}

impl Uploadable for Cw1<Daemon> {
    fn wasm(&self) -> <Daemon as TxHandler>::ContractSource {
        WasmPath::new(format!("{CRATE_PATH}/examples/wasms/cw1_whitelist.wasm")).unwrap()
    }
}
