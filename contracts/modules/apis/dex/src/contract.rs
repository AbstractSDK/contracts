use abstract_api::{ApiContract, ApiResult};
use abstract_os::{
    api::{BaseInstantiateMsg, ExecuteMsg, QueryMsg},
    dex::{ApiQueryMsg, RequestMsg},
    EXCHANGE,
};

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};

use crate::{
    commands::{
        resolve_exchange,
    },
    error::DexError,
    queries::simulate_swap,
};
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexApi<'a> = ApiContract<'a, RequestMsg>;
pub type DexResult = Result<Response, DexError>;
const DEX_API: DexApi<'static> = DexApi::new(&[]);

// Supported exchanges on XXX
// ...

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: BaseInstantiateMsg,
) -> ApiResult {
    DexApi::instantiate(deps, env, info, msg, EXCHANGE, CONTRACT_VERSION, vec![])?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<RequestMsg>,
) -> DexResult {
    DEX_API.handle_request(
        deps,
        env,
        info,
        msg,
        handle_api_request,
        None, // Some(handle_ibc_callback),
    )
}

pub fn handle_api_request(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _api: DexApi,
    msg: RequestMsg,
) -> DexResult {
    let RequestMsg {
        dex: dex_name,
        action: _,
    } = msg;
    let _exchange = resolve_exchange(dex_name)?;
    // if !exchange.over_ibc() {
    //     todo!()
    //     match action {
    //         DexAction::ProvideLiquidity { assets, max_spread } => {
    //             if assets.len() < 2 {
    //                 return Err(DexError::TooFewAssets {});
    //             }
    //             provide_liquidity(deps.as_ref(), env, info, api, assets, dex_name, max_spread)
    //         }
    //         DexAction::ProvideLiquiditySymmetric {
    //             offer_asset,
    //             paired_assets,
    //         } => {
    //             if paired_assets.is_empty() {
    //                 return Err(DexError::TooFewAssets {});
    //             }
    //             provide_liquidity_symmetric(
    //                 deps.as_ref(),
    //                 env,
    //                 info,
    //                 api,
    //                 offer_asset,
    //                 paired_assets,
    //                 dex_name,
    //             )
    //         }
    //         DexAction::WithdrawLiquidity { lp_token, amount } => {
    //             withdraw_liquidity(deps.as_ref(), env, info, api, (lp_token, amount), dex_name)
    //         }

    //         DexAction::Swap {
    //             offer_asset,
    //             ask_asset,
    //             max_spread,
    //             belief_price,
    //         } => swap(
    //             deps.as_ref(),
    //             env,
    //             info,
    //             api,
    //             offer_asset,
    //             ask_asset,
    //             dex_name,
    //             max_spread,
    //             belief_price,
    //         ),
    //     }
    // }
    todo!()
}

// pub fn handle_ibc_callback(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     api: DexApi,
//     id: String,
//     msg: StdAck,
// ) -> DexResult {
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<ApiQueryMsg>) -> Result<Binary, DexError> {
    DEX_API.handle_query(deps, env, msg, Some(query_handler))
}

fn query_handler(deps: Deps, env: Env, msg: ApiQueryMsg) -> Result<Binary, DexError> {
    match msg {
        ApiQueryMsg::SimulateSwap {
            offer_asset,
            ask_asset,
            dex,
        } => simulate_swap(deps, env, offer_asset, ask_asset, dex.unwrap()),
    }
}
