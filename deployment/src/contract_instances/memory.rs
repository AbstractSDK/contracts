use crate::{contract::ContractInstance, sender::Sender};
use cosmwasm_std::Empty;
use pandora_os::memory::msg::*;
use secp256k1::{Context, Signing};
use serde::Serialize;
use terra_rust_api::{
    client::{tx_types::TXResultSync, wasm::Wasm},
    errors::TerraRustAPIError,
};

pub struct Memory {
    instance: ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty>,
}

impl Memory {
    async fn execute<C: Signing + Context>(
        &mut self,
        sender: Sender<C>,
        exec_msg: ExecuteMsg,
    ) -> Result<TXResultSync, TerraRustAPIError> {
        self.instance.interface.execute_msg = exec_msg;
        self.instance.execute(sender).await?;
    }

    pub async fn add_new_assets(&mut self, assets: Vec<(String, String)>) -> ExecuteMsg {
        let msg: ExecuteMsg = ExecuteMsg::UpdateAssetAddresses {
            to_add: assets,
            to_remove: vec![],
        };
        self.execute(msg.clone());
        msg
    }
}
