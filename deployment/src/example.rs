use crate::contract_instances::memory::Memory;
use secp256k1::Secp256k1;
use terra_rust_api::{core_types::Coin, messages::MsgSend, GasOptions, Message, PrivateKey, Terra};

async fn demo() -> anyhow::Result<()> {
    // set up the LCD client
    let gas_opts = GasOptions::create_with_gas_estimate("50ukrw", 1.4)?;
    let terra = Terra::lcd_client(
        "https://bombay-lcd.terra.dev/",
        "bombay-12",
        &gas_opts,
        None,
    );
    // generate a private key
    let secp = Secp256k1::new();
    let from_key = PrivateKey::from_words(&secp, "your secret words", 0, 0)?;
    let from_public_key = from_key.public_key(&secp);
    // generate the message SEND 1000 uluna from your private key to someone else
    let coin: Coin = Coin::parse("1000uluna")?.unwrap();
    let from_account = from_public_key.account()?;
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
