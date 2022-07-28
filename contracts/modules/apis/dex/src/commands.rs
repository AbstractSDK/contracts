use abstract_sdk::LoadMemory;
use cosmwasm_std::{Deps, Env, MessageInfo, Decimal};

use crate::{contract::{DexResult, DexApi}, error::DexError, DEX};
use abstract_os::{dex::ProvidedAsset, objects::memory_entry::AssetEntry};

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


pub fn swap(deps: Deps, env: Env, info: MessageInfo, api: DexApi,offer_asset: ProvidedAsset, ask_asset: String, dex: String, max_spread: Option<Decimal>, belief_price: Option<Decimal>) -> DexResult {
    let (offer_asset, offer_amount) = offer_asset;
    let memory = api.mem(deps.storage)?;
    let offer_asset_info = AssetEntry::from(offer_asset).resolve(deps,&memory)?;
    let ask_asset_info = AssetEntry::from(ask_asset).resolve(deps,&memory)?;

    let exchange= resolve_exchange(dex)?;
    
}