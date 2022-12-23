use crate::{
    contract::{DexApi, DexResult},
    error::DexError,
    DEX,
};

use crate::dex_trait::{Fee, FeeOnInput, Return, Spread};
use abstract_os::objects::PoolId;
use abstract_sdk::OsExecute;
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Coin, CosmosMsg, Decimal, Deps, QueryRequest, StdResult,
    Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use terraswap::pair::{PoolResponse, SimulationResponse};

pub const TERRASWAP: &str = "terraswap";
pub struct Terraswap {}

impl DEX for Terraswap {
    fn name(&self) -> &'static str {
        TERRASWAP
    }
    fn over_ibc(&self) -> bool {
        false
    }
    fn swap(
        &self,
        deps: Deps,
        pair_address: PoolId,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let proxy_msg = if let AssetInfoBase::Cw20(token_addr) = &ask_asset.info {
            let hook_msg = terraswap::pair::Cw20HookMsg::Swap {
                max_spread: belief_price,
                max_spread,
                to: None,
            };
            // Call swap on pair through cw20 Send
            let msg = Cw20ExecuteMsg::Send {
                contract: ask_asset.to_string(),
                amount: ask_asset.amount,
                msg: to_binary(&hook_msg)?,
            };
            // call send on cw20
            wasm_execute(token_addr, &msg, vec![])?
        } else {
            let swap_msg = terraswap::pair::ExecuteMsg::Swap {
                offer_asset: cw_asset_to_terraswap(&ask_asset)?,
                max_spread: belief_price,
                max_spread,
                to: None,
            };
            wasm_execute(ask_asset, &swap_msg, coins_in_assets(&[ask_asset]))?
        };

        ask_asset
            .os_execute(deps, vec![proxy_msg.into()])
            .map_err(From::from)
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        pair_address: PoolId,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        if max_spread.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        }

        let terraswap_assets = max_spread
            .iter()
            .map(cw_asset_to_terraswap)
            .collect::<Result<Vec<_>, _>>()?;
        // execute msg
        let msg = terraswap::pair::ExecuteMsg::ProvideLiquidity {
            assets: [terraswap_assets[0].clone(), terraswap_assets[1].clone()],
            slippage_tolerance: max_spread,
            receiver: None,
        };
        // approval msgs for cw20 tokens (if present)
        let mut msgs = cw_approve_msgs(&max_spread, &max_spread)?;
        let coins = coins_in_assets(&max_spread);
        // actual call to pair
        let liquidity_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: max_spread.into_string(),
            msg: to_binary(&msg)?,
            funds: coins,
        });
        msgs.push(liquidity_msg);

        max_spread.os_execute(deps, msgs).map_err(From::from)
    }

    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        pair_address: PoolId,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
        other_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        if other_assets.len() > 1 {
            return Err(DexError::TooManyAssets(2));
        }
        // Get pair info
        let pair_config: PoolResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: paired_assets.to_string(),
                msg: to_binary(&terraswap::pair::QueryMsg::Pool {})?,
            }))?;

        let ts_offer_asset = cw_asset_to_terraswap(&paired_assets)?;
        let other_asset = if pair_config.assets[0].info == ts_offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[1].amount, pair_config.assets[0].amount);
            let other_token_amount = price * paired_assets.amount;
            Asset {
                amount: other_token_amount,
                info: other_assets[0].clone(),
            }
        } else if pair_config.assets[1].info == ts_offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[0].amount, pair_config.assets[1].amount);
            let other_token_amount = price * paired_assets.amount;
            Asset {
                amount: other_token_amount,
                info: other_assets[0].clone(),
            }
        } else {
            return Err(DexError::ArgumentMismatch(
                paired_assets.to_string(),
                pair_config
                    .assets
                    .iter()
                    .map(|e| e.info.to_string())
                    .collect(),
            ));
        };

        let offer_assets = [paired_assets, other_asset];

        let coins = coins_in_assets(&offer_assets);
        // approval msgs for cw20 tokens (if present)
        let mut msgs = cw_approve_msgs(&offer_assets, &paired_assets)?;

        // construct execute msg
        let terraswap_assets = offer_assets
            .iter()
            .map(cw_asset_to_terraswap)
            .collect::<Result<Vec<_>, _>>()?;
        let msg = terraswap::pair::ExecuteMsg::ProvideLiquidity {
            assets: [terraswap_assets[0].clone(), terraswap_assets[1].clone()],
            slippage_tolerance: None,
            receiver: None,
        };
        // actual call to pair
        let liquidity_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: paired_assets.into_string(),
            msg: to_binary(&msg)?,
            funds: coins,
        });
        msgs.push(liquidity_msg);

        paired_assets.os_execute(deps, msgs).map_err(From::from)
    }

    fn withdraw_liquidity(
        &self,
        deps: Deps,
        pair_address: PoolId,
        lp_token: Asset,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let hook_msg = terraswap::pair::Cw20HookMsg::WithdrawLiquidity {};
        // Call swap on pair through cw20 Send
        let withdraw_msg = lp_token.send_msg(lp_token, to_binary(&hook_msg)?)?;
        lp_token
            .os_execute(deps, vec![withdraw_msg])
            .map_err(From::from)
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pair_address: PoolId,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        // Do simulation
        let SimulationResponse {
            return_amount,
            spread_amount,
            commission_amount,
        } = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pair_address.to_string(),
            msg: to_binary(&terraswap::pair::QueryMsg::Simulation {
                offer_asset: cw_asset_to_terraswap(&offer_asset)?,
            })?,
        }))?;
        // commission paid in result asset
        Ok((return_amount, spread_amount, commission_amount, false))
    }
}

fn cw_asset_to_terraswap(asset: &Asset) -> Result<terraswap::asset::Asset, DexError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(terraswap::asset::Asset {
            amount: asset.amount,
            info: terraswap::asset::AssetInfo::NativeToken {
                denom: denom.clone(),
            },
        }),
        AssetInfoBase::Cw20(contract_addr) => Ok(terraswap::asset::Asset {
            amount: asset.amount,
            info: terraswap::asset::AssetInfo::Token {
                contract_addr: contract_addr.to_string(),
            },
        }),
        _ => Err(DexError::Cw1155Unsupported {}),
    }
}

fn cw_approve_msgs(assets: &[Asset], spender: &Addr) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];
    for asset in assets {
        if let AssetInfo::Cw20(addr) = &asset.info {
            let msg = cw20_junoswap::Cw20ExecuteMsg::IncreaseAllowance {
                spender: spender.to_string(),
                amount: asset.amount,
                expires: None,
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            }))
        }
    }
    Ok(msgs)
}

fn coins_in_assets(assets: &[Asset]) -> Vec<Coin> {
    let mut coins = vec![];
    for asset in assets {
        if let AssetInfo::Native(denom) = &asset.info {
            coins.push(Coin::new(asset.amount.u128(), denom.clone()));
        }
    }
    coins
}
