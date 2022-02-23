use std::fs::File;

use anyhow::Error;

use secp256k1::{Context, Signing};

use serde_json::{from_reader, json};
use terra_rust_api::{
    client::tx_types::TXResultSync, core_types::Coin, messages::MsgExecuteContract, Message,
};

use crate::{
    error::TerraRustScriptError,
    sender::{GroupConfig, Sender},
};
// https://doc.rust-lang.org/std/process/struct.Command.html
// RUSTFLAGS='-C link-arg=-s' cargo wasm

pub struct Interface<I, E, Q, M> {
    pub init_msg: Option<I>,
    pub execute_msg: Option<E>,
    pub query_msg: Option<Q>,
    pub migrate_msg: Option<M>,
}

impl<I, E, Q, M> Interface<I, E, Q, M> {}

impl<I, E, Q, M> Default for Interface<I, E, Q, M> {
    // Generates placeholder with type restrictions
    fn default() -> Self {
        Interface {
            init_msg: None,
            execute_msg: None,
            query_msg: None,
            migrate_msg: None,
        }
    }
}

pub struct ContractInstance<I, E, Q, M> {
    pub interface: Interface<I, E, Q, M>,
    config: GroupConfig,
}

impl<I: serde::Serialize, E: serde::Serialize, Q: serde::Serialize, M: serde::Serialize>
    ContractInstance<I, E, Q, M>
{
    pub async fn execute<C: Signing + Context>(
        &self,
        sender: &Sender<C>,
        exec_msg: E,
        coins: Vec<Coin>,
    ) -> Result<TXResultSync, TerraRustScriptError> {
        let execute_msg_json = json!(exec_msg);
        let contract = self.addresses()?;

        let send: Message = MsgExecuteContract::create_from_value(
            &sender.pub_addr()?,
            &contract,
            &execute_msg_json,
            &coins,
        )?;
        // generate the transaction & calc fees
        let messages: Vec<Message> = vec![send];
        let (std_sign_msg, sigs) = sender
            .terra
            .generate_transaction_to_broadcast(&sender.secp, &sender.private_key, messages, None)
            .await?;
        // send it out
        let resp = sender
            .terra
            .tx()
            .broadcast_sync(&std_sign_msg, &sigs)
            .await?;
        match resp.code {
            Some(code) => {
                log::error!("{}", serde_json::to_string(&resp)?);
                eprintln!("Transaction returned a {} {}", code, resp.txhash)
            }
            None => {
                println!("{}", resp.txhash)
            }
        }
        Ok(resp)
    }

    fn addresses(&self) -> Result<String, TerraRustScriptError> {
        let file = File::open(&self.config.file_path).expect(&format!(
            "file should be present at {}",
            self.config.file_path
        ));
        let json: serde_json::Value = from_reader(file).unwrap();
        let maybe_address = json[self.config.name.clone()][self.config.name.clone()].get("addr");
        match maybe_address {
            Some(addr) => {
                log::debug!("contract: {} addr: {}", self.config.name, addr);
                return Ok(addr.to_string());
            }
            None => return Err(TerraRustScriptError::AddrNotInFile()),
        }
    }

    fn code_id(&self) -> anyhow::Result<u64> {
        let file = File::open(&self.config.file_path).expect(&format!(
            "file should be present at {}",
            self.config.file_path
        ));
        let json: serde_json::Value = from_reader(file).unwrap();
        let maybe_address = json[self.config.name.clone()][self.config.name.clone()].get("code_id");
        match maybe_address {
            Some(code_id) => {
                log::debug!("contract: {} addr: {}", self.config.name, code_id);
                return Ok(code_id.as_u64().unwrap());
            }
            None => return Err(Error::msg("addr not present")),
        }
    }

    // pub fn execute(),
    // pub fn query(),
    // pub fn migrate(),
}

// #[async_trait]
// pub trait Interaction<
//     E: serde::Serialize + std::marker::Sync,
//     C: Signing + Context + std::marker::Sync,
// >
// {
//     async fn execute(
//         &self,
//         sender: &Sender<C>,
//         exec_msg: &E,
//         coins: Vec<Coin>,
//     ) -> Result<TXResultSync, TerraRustScriptError> {
//         let execute_msg_json = json!(exec_msg);
//         let contract = self.addresses()?;

//         let send: Message = MsgExecuteContract::create_from_value(
//             &sender.pub_addr()?,
//             &contract,
//             &execute_msg_json,
//             &coins,
//         )?;
//         // generate the transaction & calc fees
//         let messages: Vec<Message> = vec![send];
//         let (std_sign_msg, sigs) = sender
//             .terra
//             .generate_transaction_to_broadcast(&sender.secp, &sender.private_key, messages, None)
//             .await?;
//         // send it out
//         let resp = sender
//             .terra
//             .tx()
//             .broadcast_sync(&std_sign_msg, &sigs)
//             .await?;
//         match resp.code {
//             Some(code) => {
//                 log::error!("{}", serde_json::to_string(&resp)?);
//                 eprintln!("Transaction returned a {} {}", code, resp.txhash)
//             }
//             None => {
//                 println!("{}", resp.txhash)
//             }
//         };
//         Ok(resp)
//     }

// }
