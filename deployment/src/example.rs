use crate::{
    contract_instances::memory::Memory,
    sender::{GroupConfig, Network, Sender},
};
use secp256k1::Secp256k1;
use std::env;
use terra_rust_api::{core_types::Coin, messages::MsgSend, GasOptions, Message, PrivateKey, Terra};

async fn demo() -> anyhow::Result<()> {
    let secp = Secp256k1::new();
    let client = reqwest::Client::new();

    // All configs are set here
    let private_key = PrivateKey::from_words(&secp, "your secret words", 0, 0)?;
    let group_name = "debugging".to_string();
    let config = GroupConfig::new(Network::LocalTerra, group_name, client, "uusd").await?;
    let sender = Sender::new(config, private_key, secp);

    Memory::new();

    let send: Message = MsgSend::create(
        from_account,
        String::from("terra1usws7c2c6cs7nuc8vma9qzaky5pkgvm2uag6rh"),
        vec![coin],
    )?;
    // generate the transaction & calc fees
    let messages: Vec<Message> = vec![send];
    let (std_sign_msg, sigs) = terra
        .generate_transaction_to_broadcast(&secp, &from_key, messages, None)
        .await?;
    // send it out
    let resp = terra.tx().broadcast_sync(&std_sign_msg, &sigs).await?;
    match resp.code {
        Some(code) => {
            log::error!("{}", serde_json::to_string(&resp)?);
            eprintln!("Transaction returned a {} {}", code, resp.txhash)
        }
        None => {
            println!("{}", resp.txhash)
        }
    }
    Ok(())
}
