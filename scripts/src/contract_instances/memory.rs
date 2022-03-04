use std::{cmp::min, fs::File};

use cosmwasm_std::Empty;
use secp256k1::{Context, Signing};
use serde_json::from_reader;

use terra_rust_script::{
    contract::{ContractInstance, Interface},
    sender::GroupConfig,
};
use terra_rust_script::{error::TerraRustScriptError, sender::Sender};
use terraswap::asset::AssetInfo;
use pandora_os::native::memory::msg::*;

pub struct Memory(pub ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty>);

impl Memory {
    pub fn new(group_config: GroupConfig) -> Memory {
        let instance = ContractInstance {
            interface: Interface::default(),
            group_config,
            name: "memory".to_string(),
        };
        instance.check_scaffold().unwrap();
        Memory(instance)
    }

    pub async fn update_all<C: Signing + Context>(
        &self,
        sender: &Sender<C>,
    ) -> Result<(), TerraRustScriptError> {
        let file = File::open(
            "/home/cyberhoward/Programming/WhiteWhale/contracts/scripts/resources/assets.json",
        )
        .expect(&format!(
            "file should be present at {}",
            "/home/cyberhoward/Programming/WhiteWhale/contracts/scripts/resources/assets.json"
        ));
        let json: serde_json::Value = from_reader(file)?;
        let maybe_assets = json.get(self.0.group_config.network_config.chain_id.clone());

        match maybe_assets {
            Some(assets_value) => {
                let assets = assets_value.as_object().unwrap();
                let to_add: Vec<(String, AssetInfo)> = assets
                    .iter()
                    .map(|(name, value)| {
                        let id = value.as_str().unwrap().to_owned();
                        if id.contains("terra1") {
                            (name.to_owned(), AssetInfo::Token { contract_addr: id })
                        } else {
                            (name.to_owned(), AssetInfo::NativeToken { denom: id })
                        }
                    })
                    .collect();
                let mut i = 0;
                while i != to_add.len() - 1 {
                    let chunk = to_add.get(i..min(i + 25, to_add.len() - 1)).unwrap();
                    i += chunk.len();
                    self.0
                        .execute(
                            sender,
                            ExecuteMsg::update_asset_addresses(chunk.to_vec(), vec![]),
                            vec![],
                        )
                        .await?;
                }

                return Ok(());
            }
            None => return Err(TerraRustScriptError::StdErr("network not found".into())),
        }
    }
}
