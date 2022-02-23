use crate::{
    contract_instances::memory::Memory,
    sender::{GroupConfig, Network, Sender},
};
use secp256k1::Secp256k1;

use terra_rust_api::PrivateKey;

async fn demo() -> anyhow::Result<()> {
    let secp = Secp256k1::new();
    let client = reqwest::Client::new();
    let path = String::from("hhh");

    // All configs are set here
    let private_key = PrivateKey::from_words(&secp, "your secret words", 0, 0)?;
    let group_name = "debugging".to_string();
    let config = GroupConfig::new(Network::LocalTerra, group_name, client, "uusd", path).await?;
    let sender = Sender::new(&config, private_key, secp);

    let memory = Memory::new(config.clone());
    memory
        .add_new_assets(&sender, vec![("ust".to_string(), "uusd".to_string())])
        .await?;

    Ok(())
}
