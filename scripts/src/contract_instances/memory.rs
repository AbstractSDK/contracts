use crate::{
    contract::{ContractInstance, Interface},
    error::TerraRustScriptError,
    sender::{GroupConfig, Sender},
};
use cosmwasm_std::Empty;
use pandora_os::memory::msg::*;
use secp256k1::{Context, Signing};

use terra_rust_api::client::tx_types::TXResultSync;
use terra_rust_script_derive::contract;

pub struct Memory(pub ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty>);

impl Memory {
    pub fn new(group_config: GroupConfig) -> Memory {
        Memory (
            ContractInstance{
            interface: Interface::default(),
            group_config,
            name: "memory".to_string(),
            })
    }
    pub async fn add_new_assets<C: Signing + Context>(
        &self,
        sender: &Sender<C>,
        assets: Vec<(String, String)>,
    ) -> Result<TXResultSync, TerraRustScriptError> {
        let msg: ExecuteMsg = ExecuteMsg::UpdateAssetAddresses {
            to_add: assets,
            to_remove: vec![],
        };
        self.0.execute(sender, msg, vec![]).await
    }
}
