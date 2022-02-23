use crate::{
    contract::{ContractInstance, Interface},
    error::TerraRustScriptError,
    sender::{GroupConfig, Sender},
};
use cosmwasm_std::Empty;
use pandora_os::memory::msg::*;
use secp256k1::{Context, Signing};

use terra_rust_api::client::tx_types::TXResultSync;

pub type Memory = ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty>;

impl Memory {
    pub fn new(config: GroupConfig) -> Memory {
        Memory {
            interface: Interface::default(),
            config,
        }
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
        self.execute(sender, msg, vec![]).await
    }
}
