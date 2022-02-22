use crate::{contract::ContractInstance, error::TerraRustScriptError, sender::Sender};
use cosmwasm_std::Empty;
use pandora_os::memory::msg::*;
use secp256k1::{Context, Signing};

use terra_rust_api::client::tx_types::TXResultSync;

pub type Memory = ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty>;

impl Memory {
    pub async fn add_new_assets<C: Signing + Context>(
        &mut self,
        assets: Vec<(String, String)>,
        sender: &Sender<C>,
    ) -> Result<TXResultSync, TerraRustScriptError> {
        let msg: ExecuteMsg = ExecuteMsg::UpdateAssetAddresses {
            to_add: assets,
            to_remove: vec![],
        };
        self.execute(sender, msg, vec![]).await
    }
}
