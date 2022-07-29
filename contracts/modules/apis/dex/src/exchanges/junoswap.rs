use abstract_sdk::LoadMemory;
use cosmwasm_std::{WasmMsg, wasm_execute, Deps, QueryRequest, StdError, WasmQuery, Decimal, Addr};
use cw20::Denom;
use cw_asset::{Asset, AssetInfo};
use wasmswap::msg::*;
use crate::{DEX, contract::DexApi, error::DexError};
pub const JUNOSWAP: &str = "junoswap";

pub struct JunoSwap{}

impl DEX for JunoSwap {
    fn swap(&self, deps: Deps, api: DexApi, contract_address: Addr, offer_asset: Asset, belief_price: Option<Decimal>, max_spread: Option<Decimal>) -> Result<WasmMsg,DexError> {
        let memory = api.mem(deps.storage)?;
        memory.query_contract(deps, format!("{}/{}_{}", JUNOSWAP    ,  ))?;

        let pair_config: InfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart { contract_addr: , msg: QueryMsg::Info {  } } ))?;
        match &offer_asset {
            if pair_config.token1_denom. == &offer_asset.info.to_string().into() {

            }
        } 
        let msg = ExecuteMsg::Swap { input_token: (), input_amount: (), min_output: (), expiration: () }
        wasm_execute(contract_addr, msg, funds)
    }
}

pub fn compare_denom_to_asset(denom: Denom, asset: AssetInfo) -> Result<bool,DexError> {
    match denom {
        Denom::Native(denom_name) => {
            match asset {
                cw_asset::AssetInfoBase::Native(asset_name) => return Ok(denom_name == asset_name),
                cw_asset::AssetInfoBase::Cw20(asset_addr) => Ok(false),
                cw_asset::AssetInfoBase::Cw1155(_, _) => return Err(DexError::Cw1155Unsupported),
            }
        },
        Denom::Cw20(denom_addr) => {
            match asset {
                cw_asset::AssetInfoBase::Native(asset_name) => return Ok(false),
                cw_asset::AssetInfoBase::Cw20(asset_addr) => Ok(denom_addr == asset_addr),
                cw_asset::AssetInfoBase::Cw1155(_, _) => return Err(DexError::Cw1155Unsupported),
            }
        },
    }
}
