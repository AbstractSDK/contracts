use abstract_api::{ApiContract, ApiResult};
use abstract_os::{api::{ApiInstantiateMsg, ApiInterfaceMsg, ApiQueryMsg}, EXCHANGE};
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use abstract_os::dex::{QueryMsg, RequestMsg};
use crate::{error::DexError, DEX, commands::{swap, provide_liquidity}};

pub type DexApi<'a> = ApiContract<'a, RequestMsg>;
pub type DexResult = Result<Response, DexError>;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Supported exchanges on XXX
// ...


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInstantiateMsg,
) -> ApiResult {
    DexApi::instantiate(deps, env, info, msg, EXCHANGE, CONTRACT_VERSION, vec![])?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInterfaceMsg<RequestMsg>,
) -> DexResult {
    DexApi::handle_request(deps, env, info, msg, handle_api_request)
}

pub fn handle_api_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    api: DexApi,
    msg: RequestMsg,
) -> DexResult {
    match msg {
        RequestMsg::ProvideLiquidity { assets, dex, max_spread } => {
            let dex_name = dex.unwrap();

            provide_liquidity(deps.as_ref(), env, info, api, assets, dex_name, max_spread)
        },
        RequestMsg::ProvideLiquiditySymmetric { assets, paired_assets, dex } => {

        },
        RequestMsg::WithdrawLiquidity { lp_token, amount } => todo!(),
        RequestMsg::Swap { offer_asset, ask_asset, dex, max_spread, belief_price } => {
            // add default dex in future (osmosis??)
            let dex_name = dex.unwrap();
            swap(deps.as_ref(), env, info, api, offer_asset, ask_asset, dex_name, max_spread, belief_price)
        },
    }
    // .map_err(From::from)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: ApiQueryMsg) -> StdResult<Binary> {
   DexApi::default().query(deps, env, msg)
}
