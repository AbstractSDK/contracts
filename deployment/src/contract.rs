use std::{path::Path, fs::File};

use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use terra_rust_api::{core_types::Coin, messages::MsgExecuteContract, GasOptions, Message, PrivateKey, Terra, client::wasm::Wasm};
use pandora_os::memory::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
// https://doc.rust-lang.org/std/process/struct.Command.html
// RUSTFLAGS='-C link-arg=-s' cargo wasm

pub struct Interface<I, E, Q, M> {
    pub init_msg: I,
    pub execute_msg: E,
    pub query_msg: Q,
    pub migrate_msg: M,
}
pub struct ContractInstance<I, E, Q, M> {
    pub interface: Interface<I, E, Q, M>,
    name: String,
    group: String,
    addr_file: String,
}

impl<I, E, Q, M> ContractInstance<I, E, Q, M> {
    pub fn new(
        name: String,
        group: String,
        addr_file: String,
        interface: Interface<I, E, Q, M>,
    ) -> ContractInstance<I, E, Q, M> {
        ContractInstance {
            name,
            group,
            addr_file,
            interface,
        }
    }
    pub fn execute() -> anyhow::Result<MsgExecuteContract> {
        let from_account = from_public_key.account()?;
        let send: Message = MsgExecuteContract::create_from_json(sender, contract, execute_msg_json, coins)
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

    pub fn addresses(&self) -> String { 
        let mut file = File::open("text.json").expect(format!("file should be present at {}", self.addr_file));
        let json: serde_json::Value = from_reader(file).unwrap();
        log::debug!("{}", serde_json::to_string(&resp)?);
        println!("{}",  );

    }

    pub fn code_id(&self) -> u64 {

    }
    // pub fn execute(),
    // pub fn query(),
    // pub fn migrate(),
}
