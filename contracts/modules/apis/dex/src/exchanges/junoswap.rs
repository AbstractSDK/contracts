use abstract_os::objects::{ContractEntry, UncheckedContractEntry};
use cosmwasm_std::{WasmMsg, wasm_execute, Deps, QueryRequest, StdError, WasmQuery, Decimal, Addr, to_binary, Uint128};
use cw20_junoswap::Denom;
use cw_asset::{Asset, AssetInfo};
use wasmswap::msg::*;
use abstract_sdk::{MemoryOperation, OsExecute};
use crate::{DEX, contract::{DexApi, DexResult}, error::DexError};
pub const JUNOSWAP: &str = "junoswap";

pub struct JunoSwap{}

impl DEX for JunoSwap {
    fn name(&self) -> &'static str {
        JUNOSWAP
    }
    fn swap(&self, deps: Deps, api: DexApi, contract_address: Addr, offer_asset: Asset, ask_asset: AssetInfo, belief_price: Option<Decimal>, max_spread: Option<Decimal>) -> DexResult {

        let pair_config: InfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart { contract_addr: contract_address.to_string(), msg: to_binary(&QueryMsg::Info {  })? } ))?;

        let (offer_token, price) = if denom_and_asset_match(pair_config.token1_denom, offer_asset.info)? {
            (TokenSelect::Token1,Decimal::from_ratio(pair_config.token2_reserve, pair_config.token1_reserve))
        } else if denom_and_asset_match(pair_config.token1_denom, ask_asset)? {
            (TokenSelect::Token2, Decimal::from_ratio(pair_config.token1_reserve, pair_config.token2_reserve))
        }else {
            return Err(DexError::DexMismatch(format!("{}/{}", &offer_asset.info, &ask_asset), self.name().into(), contract_address.to_string()))
        };

        let min_out: Uint128 = match max_spread {
            None => 0u128.into(),
            Some(spread) => {
                let price_to_use = belief_price.unwrap_or(price);
                let ideal_return = offer_asset.amount * price_to_use;
                ideal_return * (Decimal::one() - spread)
            },
        };
        
        let msg = ExecuteMsg::Swap { input_token: offer_token, input_amount: offer_asset.amount, min_output: min_out, expiration: None };

        let asset_msg = offer_asset.send_msg(contract_address, to_binary(&msg)?)?;
        api.os_execute(deps, vec![asset_msg]).map_err(From::from)
    }

    fn provide_liquidity(&self, deps: Deps, api: DexApi, contract_address: Addr, offer_assets: Vec<Asset>, max_spread: Option<Decimal>) -> DexResult {
        let msg = ExecuteMsg::AddLiquidity { token1_amount: (), min_liquidity: (), max_token2: (), expiration: () }
    }

    fn provide_liquidity_symmetric(&self, deps: Deps, api: DexApi, contract_address: Addr, offer_asset: Asset, other_assets: Vec<AssetInfo>)-> DexResult {
        todo!()
    }
}

fn denom_and_asset_match(denom: Denom, asset: AssetInfo) -> Result<bool,DexError> {
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