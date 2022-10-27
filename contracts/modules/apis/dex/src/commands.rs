// #![allow(unused)]
use abstract_sdk::{MemoryOperation, OsExecute};
use cosmwasm_std::{Decimal, Deps, Env, MessageInfo};
use cw_asset::{Asset, AssetInfo};

use crate::{
    contract::{DexApi, DexResult},
    error::DexError,
    exchanges::osmosis::{Osmosis, OSMOSIS},
    DEX,
};
use abstract_os::{
    dex::{OfferAsset, SwapRouter},
    objects::{AssetEntry, UncheckedContractEntry},
};

// Supported exchanges on Juno
#[cfg(feature = "juno")]
pub use crate::exchanges::junoswap::{JunoSwap, JUNOSWAP};

#[cfg(any(feature = "juno", feature = "terra"))]
pub use crate::exchanges::loop_dex::{Loop, LOOP};

#[cfg(feature = "terra")]
pub use crate::exchanges::terraswap::{Terraswap, TERRASWAP};

pub(crate) fn resolve_exchange(value: &str) -> Result<&'static dyn DEX, DexError> {
    match value {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(&JunoSwap {}),
        #[cfg(feature = "juno")]
        OSMOSIS => Ok(&Osmosis {}),
        #[cfg(any(feature = "juno", feature = "terra"))]
        LOOP => Ok(&Loop {}),
        #[cfg(feature = "terra")]
        TERRASWAP => Ok(&Terraswap {}),
        _ => Err(DexError::UnknownDex(value.to_owned())),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn swap(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    api: DexApi,
    offer_asset: OfferAsset,
    mut ask_asset: AssetEntry,
    exchange: &dyn DEX,
    max_spread: Option<Decimal>,
    belief_price: Option<Decimal>,
) -> DexResult {
    let (mut offer_asset, offer_amount) = offer_asset;
    offer_asset.format();
    ask_asset.format();
    let offer_asset_info = api.resolve(deps, &offer_asset)?;
    let ask_asset_info = api.resolve(deps, &ask_asset)?;

    let pair_address = exchange.pair_address(deps, &api, &mut vec![&offer_asset, &ask_asset])?;
    let offer_asset: Asset = Asset::new(offer_asset_info, offer_amount);

    let msgs = exchange.swap(
        deps,
        pair_address,
        offer_asset,
        ask_asset_info,
        belief_price,
        max_spread,
    )?;
    api.os_execute(deps, msgs).map_err(From::from)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_swap(
    _deps: Deps,
    _env: Env,
    _info: MessageInfo,
    _api: DexApi,
    _offer_assets: Vec<OfferAsset>,
    _ask_assets: Vec<OfferAsset>,
    _exchange: &dyn DEX,
    _max_spread: Option<Decimal>,
    _router: Option<SwapRouter>,
) -> DexResult {
    todo!()

    // let memory = api.load_memory(deps.storage)?;
    //
    // // Resolve the asset information
    // let mut offer_asset_infos: Vec<AssetInfo> =
    //     exchange.resolve_assets(deps, &api, offer_assets.into_iter().unzip().0)?;
    // let mut ask_asset_infos: Vec<AssetInfo> =
    //     exchange.resolve_assets(deps, &api, ask_assets.into_iter().unzip().0)?;
    //
    // let offer_assets: Vec<Asset> = offer_assets
    //     .into_iter()
    //     .zip(offer_asset_infos)
    //     .map(|(asset, info)| Asset::new(info, asset.1))
    //     .collect();
    // let ask_assets: Vec<Asset> = ask_assets
    //     .into_iter()
    //     .zip(ask_asset_infos)
    //     .map(|(asset, info)| Asset::new(info, asset.1))
    //     .collect();
    //
    // exchange.custom_swap(deps, offer_assets, ask_assets, max_spread)
}

pub fn provide_liquidity(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    api: DexApi,
    offer_assets: Vec<OfferAsset>,
    exchange: &dyn DEX,
    max_spread: Option<Decimal>,
) -> DexResult {
    let mut assets = vec![];
    for offer in &offer_assets {
        let info = api.resolve(deps, &offer.0)?;
        let asset = Asset::new(info, offer.1);
        assets.push(asset);
    }
    let pair_address = exchange.pair_address(
        deps,
        &api,
        offer_assets
            .iter()
            .map(|(a, _)| a)
            .collect::<Vec<&AssetEntry>>()
            .as_mut(),
    )?;
    let msgs = exchange.provide_liquidity(deps, pair_address, assets, max_spread)?;
    api.os_execute(deps, msgs).map_err(From::from)
}

pub fn provide_liquidity_symmetric(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    api: DexApi,
    offer_asset: OfferAsset,
    mut paired_assets: Vec<AssetEntry>,
    exchange: &dyn DEX,
) -> DexResult {
    let paired_asset_infos: Result<Vec<AssetInfo>, _> = paired_assets
        .iter()
        .map(|entry| api.resolve(deps, entry))
        .collect();
    paired_assets.push(offer_asset.0.clone());
    let pair_address = exchange.pair_address(deps, &api, &mut paired_assets.iter().collect())?;
    let offer_asset = Asset::new(api.resolve(deps, &offer_asset.0)?, offer_asset.1);
    let msgs = exchange.provide_liquidity_symmetric(
        deps,
        pair_address,
        offer_asset,
        paired_asset_infos?,
    )?;
    api.os_execute(deps, msgs).map_err(From::from)
}

pub fn withdraw_liquidity(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    api: DexApi,
    lp_token: OfferAsset,
    exchange: &dyn DEX,
) -> DexResult {
    let info = api.resolve(deps, &lp_token.0)?;
    let lp_asset = Asset::new(info, lp_token.1);
    let pair_entry = UncheckedContractEntry::new(exchange.name(), lp_token.0.as_str()).check();

    let pair_address = api.resolve(deps, &pair_entry)?;
    let msgs = exchange.withdraw_liquidity(deps, pair_address, lp_asset)?;
    api.os_execute(deps, msgs).map_err(From::from)
}
