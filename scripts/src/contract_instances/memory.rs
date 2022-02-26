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
    pub fn new(group_config: GroupConfig) -> Memory {
        let instance = ContractInstance {
            interface: Interface::default(),
            group_config,
            name: "memory".to_string(),
        };
        instance.check_scaffold().unwrap();
        instance
    }

    pub fn test(&self) -> bool {
        true
    }
}
