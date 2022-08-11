use abstract_os::objects::{ContractEntry, UncheckedContractEntry};
use cosmwasm_std::{WasmMsg, wasm_execute, Deps, QueryRequest, StdError, WasmQuery, Decimal, Addr, to_binary, Uint128, Fraction, CosmosMsg, StdResult, Coin, Response};
use cw20_junoswap::{Denom, Cw20Coin};
use cw_asset::{Asset, AssetInfo, AssetList};
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

        let (offer_token, price) = if denom_and_asset_match(&pair_config.token1_denom, &offer_asset.info)? {
            (TokenSelect::Token1,Decimal::from_ratio(pair_config.token2_reserve, pair_config.token1_reserve))
        } else if denom_and_asset_match(&pair_config.token1_denom, &ask_asset)? {
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
        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2))
        }
        let pair_config: InfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart { contract_addr: contract_address.to_string(), msg: to_binary(&QueryMsg::Info {  })? } ))?;
        let (token1,token2) = if denom_and_asset_match(&pair_config.token1_denom, &offer_assets[0].info)? {
            (&offer_assets[0],&offer_assets[1])
        } else if denom_and_asset_match(&pair_config.token1_denom, &offer_assets[1].info)? {
            (&offer_assets[1],&offer_assets[0])
        }else {
            return Err(DexError::DexMismatch(format!("{}/{}", offer_assets[0].info,offer_assets[1].info), self.name().into(), contract_address.to_string()))
        };

        let my_ratio = Decimal::from_ratio(token1.amount, token2.amount);
        let max_token2 = if let Some(max_spread) = max_spread {
            token1.amount * my_ratio.inv().unwrap() * (max_spread + Decimal::one())
        } else {
            Uint128::MAX
        };
        
        let msg = ExecuteMsg::AddLiquidity { token1_amount: token1.amount, min_liquidity: Uint128::zero(), max_token2, expiration: None };
        let approve_msgs = cw_approve_msgs(&offer_assets, api.request_destination)?;
        let coins = coins_in_assets(&offer_assets);
        let junoswap_msg = CosmosMsg::Wasm(WasmMsg::Execute { contract_addr: contract_address.into_string(), msg: to_binary(&msg)?, funds: coins });
        Ok(Response::new().add_messages(approve_msgs).add_message(junoswap_msg))
    }

    fn provide_liquidity_symmetric(&self, deps: Deps, api: DexApi, contract_address: Addr, offer_asset: Asset, other_assets: Vec<AssetInfo>)-> DexResult {
        todo!()
    }
}

fn denom_and_asset_match(denom: &Denom, asset: &AssetInfo) -> Result<bool,DexError> {
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

fn abs_price_diff(dec1:Decimal,dec2:Decimal) -> Decimal {
    if dec1> dec2 {
        dec2-dec1
    } else {
        dec1-dec2
    }
}

fn cw_approve_msgs(assets: &[Asset], spender: Addr) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];
    for asset in assets {
        match &asset.info {
            AssetInfo::Cw20(addr) => {
                let msg = cw20_junoswap::Cw20ExecuteMsg::IncreaseAllowance { spender: spender.to_string(), amount: asset.amount, expires: None };
                msgs.push(CosmosMsg::Wasm(WasmMsg::Execute { contract_addr: addr.to_string(), msg: to_binary(&msg)?, funds: vec![] }))
            },
            _ => ()
        }
    }
    Ok(msgs)
}

fn coins_in_assets(assets: &[Asset]) -> Vec<Coin> {
    let mut coins = vec![];
    for asset in assets {
        match &asset.info {
            AssetInfo::Native(denom) => {
                coins.push(Coin::new(asset.amount.u128(), denom.clone()));
            },
            _ => ()
        }
    }
    coins
}