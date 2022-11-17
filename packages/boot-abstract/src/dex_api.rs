use abstract_sdk::os::api::*;
use abstract_sdk::os::base;
use abstract_sdk::os::dex::*;
use abstract_sdk::os::objects::AnsAsset;
use abstract_sdk::os::objects::AssetEntry;
use abstract_sdk::os::EXCHANGE;
use abstract_sdk::os::MANAGER;
use boot_core::BootError;
use boot_core::{Contract, IndexResponse, TxHandler, TxResponse};
use cosmwasm_std::Empty;

use crate::manager::Manager;
use crate::AbstractOS;

pub type DexApi<Chain> = AbstractOS<
    Chain,
    ExecuteMsg<DexRequestMsg>,
    base::InstantiateMsg<BaseInstantiateMsg>,
    abstract_sdk::os::api::QueryMsg<abstract_sdk::os::dex::DexQueryMsg>,
    Empty,
>;

impl<Chain: TxHandler + Clone> DexApi<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("dex"),
            // .with_mock(Box::new(
            //     ContractWrapper::new_with_empty(
            //         ::contract::execute,
            //         ::contract::instantiate,
            //         ::contract::query,
            //     ),
            // ))
        )
    }

    pub fn swap(
        &self,
        offer_asset: (&str, u128),
        ask_asset: &str,
        dex: String,
    ) -> Result<(), BootError> {
        let manager = Manager::new(MANAGER, &self.chain());
        let asset = AssetEntry::new(offer_asset.0);
        let ask_asset = AssetEntry::new(ask_asset);
        manager.execute_on_module(
            EXCHANGE,
            ExecuteMsg::<DexRequestMsg>::App(ApiRequestMsg {
                proxy_address: None,
                request: DexRequestMsg {
                    dex,
                    action: DexAction::Swap {
                        offer_asset: AnsAsset::new(asset, offer_asset.1),
                        ask_asset,
                        max_spread: None,
                        belief_price: None,
                    },
                },
            }),
        )?;
        Ok(())
    }
}
