use std::env;

use crate::{
    contract_instances::memory::Memory,
    sender::{GroupConfig, Network, Sender},
};
use pandora_os::*;
use secp256k1::Secp256k1;

use pandora_os::memory::msg::{
    ExecuteMsg as MemExec, InstantiateMsg as MemInit, QueryMsg as MemQuery,
};

pub async fn demo() -> anyhow::Result<()> {
    let secp = Secp256k1::new();
    let client = reqwest::Client::new();
    let path = env::var("ADDRESS_JSON")?;
    let propose_on_multisig = false;

    // All configs are set here
    let group_name = "debugging".to_string();
    let config = GroupConfig::new(
        Network::LocalTerra,
        group_name,
        client,
        "uusd",
        path,
        propose_on_multisig,
        &secp,
    )
    .await?;
    let sender = &Sender::new(&config, secp)?;

    let memory = Memory::new(config.clone());
    memory.execute(
        sender,
        MemExec::update_asset_addresses(vec![], vec![]),
        vec![],
    ).await?;

    log::debug!(
        "{:?}",
        memory::msg::ExecuteMsg::set_admin("oeuaoeuaoeu".into())
    );

    // memory.0.upload(&sender, "/home/cyberhoward/Programming/Pandora/contracts/artifacts/memory.wasm").await?;
    // memory.instantiate(&sender).await?;
    // memory
    //     .add_new_assets(&sender, vec![("ust".to_string(), "uusd".to_string())])
    //     .await?;

    Ok(())
}
