use crate::{dex_trait::Identify, error::DexError, DEX};
use abstract_os::objects::PoolAddress;
use abstract_sdk::helpers::cosmwasm_std::wasm_smart_query;
use astroport::{asset::PairInfo, pair::QueryMsg};
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Coin, CosmosMsg, Decimal, Deps, Fraction, StdResult, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetList};
pub const ASTROPORT: &str = "astroport";

// Source https://github.com/astroport-fi/astroport-core
pub struct Astroport {}

impl Identify for Astroport {
    fn name(&self) -> &'static str {
        ASTROPORT
    }
    fn over_ibc(&self) -> bool {
        false
    }
}

impl DEX for Astroport {
    fn swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;
        let pair_config: PairInfo = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &QueryMsg::Pair {},
        )?)?;

        let swap_msg: Vec<CosmosMsg> = match &offer_asset.info {
            AssetInfo::Native(_) => vec![wasm_execute(
                pair_address.to_string(),
                &astroport::pair::ExecuteMsg::Swap {
                    offer_asset: cw_asset_to_astroport(offer_asset.clone())?,
                    ask_asset_info: None,
                    belief_price,
                    max_spread,
                    to: None,
                },
                vec![offer_asset.clone().try_into()?],
            )?
            .into()],
            AssetInfo::Cw20(addr) => vec![wasm_execute(
                addr.to_string(),
                &Cw20ExecuteMsg::Send {
                    contract: pair_address.to_string(),
                    amount: offer_asset.amount,
                    msg: to_binary(&astroport::pair::Cw20HookMsg::Swap {
                        ask_asset_info: None,
                        belief_price,
                        max_spread: Some(Decimal::zero()),
                        to: None,
                    })?,
                },
                vec![],
            )?
            .into()],
            AssetInfo::Cw1155(..) => return Err(DexError::Cw1155Unsupported {}),
            _ => panic!("unsupported asset"),
        };
        Ok(swap_msg)
    }

    //     fn provide_liquidity(
    //         &self,
    //         deps: Deps,
    //         env: &Env,
    //         assets: AssetList,
    //         min_out: Uint128,
    //     ) -> Result<Response, DexError> {
    //         let lp_out = self.simulate_provide_liquidity(deps, env, assets.clone())?;
    //         if min_out > lp_out.amount {
    //             return Err(CwDexError::MinOutNotReceived {
    //                 min_out,
    //                 received: lp_out.amount,
    //             });
    //         }

    //         let msg = PairExecuteMsg::ProvideLiquidity {
    //             assets: assets.to_owned().try_into()?,
    //             slippage_tolerance: Some(Decimal::from_str(MAX_ALLOWED_SLIPPAGE)?),
    //             auto_stake: Some(false),
    //             receiver: None,
    //         };

    //         let (funds, cw20s) = separate_natives_and_cw20s(&assets);

    //         // Increase allowance on all Cw20s
    //         let allowance_msgs: Vec<CosmosMsg> = cw20s
    //             .into_iter()
    //             .map(|asset| {
    //                 Ok(CosmosMsg::Wasm(WasmMsg::Execute {
    //                     contract_addr: asset.address,
    //                     msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
    //                         spender: self.pair_addr.to_string(),
    //                         amount: asset.amount,
    //                         expires: Some(Expiration::AtHeight(env.block.height + 1)),
    //                     })?,
    //                     funds: vec![],
    //                 }))
    //             })
    //             .collect::<StdResult<Vec<_>>>()?;

    //         let provide_liquidity = CosmosMsg::Wasm(WasmMsg::Execute {
    //             contract_addr: self.pair_addr.to_string(),
    //             msg: to_binary(&msg)?,
    //             funds,
    //         });

    //         let event = Event::new("apollo/cw-dex/provide_liquidity")
    //             .add_attribute("pair_addr", &self.pair_addr)
    //             .add_attribute("assets", format!("{:?}", assets));

    //         Ok(Response::new()
    //             .add_messages(allowance_msgs)
    //             .add_message(provide_liquidity)
    //             .add_event(event))
    //     }

    //     fn provide_liquidity_symmetric(
    //         &self,
    //         _deps: Deps,
    //         _pool_id: PoolAddress,
    //         _offer_asset: Asset,
    //         _paired_assets: Vec<AssetInfo>,
    //     ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
    //         Err(DexError::NotImplemented(self.name().to_string()))
    //     }

    //     fn withdraw_liquidity(
    //         &self,
    //         _deps: Deps,
    //         _env: &Env,
    //         asset: Asset,
    //     ) -> Result<Response, DexError> {
    //         if let AssetInfoBase::Cw20(token_addr) = &asset.info {
    //             let withdraw_liquidity = CosmosMsg::Wasm(WasmMsg::Execute {
    //                 contract_addr: token_addr.to_string(),
    //                 msg: to_binary(&Cw20ExecuteMsg::Send {
    //                     contract: self.pair_addr.to_string(),
    //                     amount: asset.amount,
    //                     msg: to_binary(&PairCw20HookMsg::WithdrawLiquidity {})?,
    //                 })?,
    //                 funds: vec![],
    //             });

    //             let event = Event::new("apollo/cw-dex/withdraw_liquidity")
    //                 .add_attribute("pair_addr", &self.pair_addr)
    //                 .add_attribute("asset", format!("{:?}", asset))
    //                 .add_attribute("token_amount", asset.amount);

    //             Ok(Response::new()
    //                 .add_message(withdraw_liquidity)
    //                 .add_event(event))
    //         } else {
    //             Err(CwDexError::InvalidInAsset { a: asset })
    //         }
    //     }

    //     fn simulate_swap(
    //         &self,
    //         deps: Deps,
    //         offer_asset: Asset,
    //         _ask_asset_info: AssetInfo,
    //         _sender: Option<String>,
    //     ) -> StdResult<Uint128> {
    //         Ok(deps
    //             .querier
    //             .query::<SimulationResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
    //                 contract_addr: self.pair_addr.to_string(),
    //                 msg: to_binary(&PairQueryMsg::Simulation {
    //                     offer_asset: offer_asset.into(),
    //                 })?,
    //             }))?
    //             .return_amount)
    //     }
}

fn cw_asset_to_astroport(asset: Asset) -> Result<astroport::asset::Asset, DexError> {
    match asset.info {
        AssetInfoBase::Native(denom) => Ok(astroport::asset::Asset {
            amount: asset.amount,
            info: astroport::asset::AssetInfo::NativeToken {
                denom: denom.clone(),
            },
        }),
        AssetInfoBase::Cw20(contract_addr) => Ok(astroport::asset::Asset {
            amount: asset.amount,
            info: astroport::asset::AssetInfo::Token { contract_addr },
        }),
        _ => Err(DexError::Cw1155Unsupported {}),
    }
}
