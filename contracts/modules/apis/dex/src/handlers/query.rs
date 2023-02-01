use crate::contract::DexApi;
use crate::exchanges::exchange_resolver::resolve_exchange;
use abstract_os::dex::state::SWAP_FEE;
use abstract_os::dex::{DexQueryMsg, OfferAsset, SimulateSwapResponse};
use abstract_os::objects::{AssetEntry, DexAssetPairing};
use abstract_sdk::base::features::AbstractNameService;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};

pub fn query_handler(deps: Deps, env: Env, api: &DexApi, msg: DexQueryMsg) -> StdResult<Binary> {
    match msg {
        DexQueryMsg::SimulateSwap {
            offer_asset,
            ask_asset,
            dex,
        } => {
            simulate_swap(deps, env, api, offer_asset, ask_asset, dex.unwrap()).map_err(Into::into)
        }
    }
}

pub fn simulate_swap(
    deps: Deps,
    _env: Env,
    api: &DexApi,
    mut offer_asset: OfferAsset,
    mut ask_asset: AssetEntry,
    dex: String,
) -> StdResult<Binary> {
    let exchange = resolve_exchange(&dex).map_err(|e| StdError::generic_err(e.to_string()))?;
    let ans = api.name_service(deps);
    let fee = SWAP_FEE.load(deps.storage)?;

    // format input
    offer_asset.name.format();
    ask_asset.format();
    // get addresses
    let swap_offer_asset = ans.query(&offer_asset)?;
    let ask_asset_info = ans.query(&ask_asset)?;
    let pair_address = exchange
        .pair_address(
            deps,
            ans.host(),
            (offer_asset.name.clone(), ask_asset.clone()),
        )
        .map_err(|e| {
            StdError::generic_err(format!(
                "Failed to get pair address for {offer_asset:?} and {ask_asset:?}: {e}"
            ))
        })?;
    let pool_info =
        DexAssetPairing::new(offer_asset.name.clone(), ask_asset.clone(), exchange.name());

    // compute api fee
    let api_fee = fee.compute(offer_asset.amount);
    offer_asset.amount -= api_fee;

    let (return_amount, spread_amount, commission_amount, fee_on_input) = exchange
        .simulate_swap(deps, pair_address, swap_offer_asset, ask_asset_info)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let commission_asset = if fee_on_input {
        ask_asset
    } else {
        offer_asset.name
    };
    let resp = SimulateSwapResponse {
        pool: pool_info,
        return_amount,
        spread_amount,
        commission: (commission_asset, commission_amount),
        api_fee,
    };
    to_binary(&resp).map_err(From::from)
}
