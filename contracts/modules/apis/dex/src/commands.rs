use abstract_sdk::{MemoryOperation, Resolve};
use cosmwasm_std::{Deps, Env, MessageInfo, Decimal, Uint128};
use cw_asset::{AssetInfo, Asset};

use crate::{contract::{DexResult, DexApi}, error::DexError, DEX};
use abstract_os::{objects::{AssetEntry, UncheckedContractEntry}, dex::OfferAsset};

// Supported exchanges on Juno
#[cfg(feature = "juno")]
pub use crate::exchanges::junoswap::{JUNOSWAP,JunoSwap};


fn resolve_exchange(value: String) -> Result<&'static dyn DEX, DexError> {
    match value.as_str() {
                #[cfg(feature = "juno")]
                JUNOSWAP => {
                    Ok(&JunoSwap {})
                },
                _ => return Err(DexError::UnknownDex(value))
                }
}


pub fn swap(deps: Deps, env: Env, info: MessageInfo, api: DexApi,offer_asset: OfferAsset, ask_asset: AssetEntry, dex: String, max_spread: Option<Decimal>, belief_price: Option<Decimal>) -> DexResult {
    let exchange= resolve_exchange(dex)?;
    let (mut offer_asset, offer_amount) = offer_asset;
    offer_asset.format();
    ask_asset.format();
    let offer_asset_info = api.resolve(deps,&offer_asset)?;
    let ask_asset_info = api.resolve(deps,&ask_asset)?;
    
    let pair_address = exchange.pair_address(deps, &api, &mut [offer_asset,ask_asset])?;
    let offer_asset: Asset = Asset::new(offer_asset_info, offer_amount);

    exchange.swap(deps, api, pair_address, offer_asset, ask_asset_info, belief_price, max_spread)
}

pub fn provide_liquidity(deps: Deps, env: Env, info: MessageInfo, api: DexApi,offer_assets: Vec<OfferAsset>, dex: String, max_spread: Option<Decimal>) -> DexResult {
    let exchange= resolve_exchange(dex)?;
    let mut assets= vec![];
    for offer in offer_assets {
        let info = api.resolve(deps, &offer.0)?;
        let asset = Asset::new(info, offer.1);
        assets.push(asset);
        
    }
    let pair_address = exchange.pair_address(deps, &api, offer_assets.into_iter().map(|(a,_)|a).collect::<Vec<AssetEntry>>().as_mut())?;
    exchange.provide_liquidity(deps, api, pair_address, assets, max_spread)
}