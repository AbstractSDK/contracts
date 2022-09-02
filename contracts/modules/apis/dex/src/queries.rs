use abstract_os::{dex::OfferAsset, objects::AssetEntry};
use abstract_sdk::MemoryOperation;
use cosmwasm_std::{to_binary, Binary, Deps, Env, MessageInfo, StdResult};
use cw_asset::Asset;

use crate::{commands::resolve_exchange, contract::DexApi, error::DexError};

pub fn simulate_swap(
    deps: Deps,
    _env: Env,
    offer_asset: OfferAsset,
    mut ask_asset: AssetEntry,
    dex: String,
) -> Result<Binary, DexError> {
    let exchange = resolve_exchange(dex)?;
    let api = DexApi::default();
    // format input
    let (mut offer_asset, offer_amount) = offer_asset;
    offer_asset.format();
    ask_asset.format();
    // get addresses
    let offer_asset_info = api.resolve(deps, &offer_asset)?;
    let ask_asset_info = api.resolve(deps, &ask_asset)?;
    let pair_address =
        exchange.pair_address(deps, &api, &mut [offer_asset.clone(), ask_asset.clone()])?;
    // create offer asset
    let swap_offer_asset: Asset = Asset::new(offer_asset_info, offer_amount);
    let mut resp = exchange.simulate_swap(deps, pair_address, swap_offer_asset, ask_asset_info)?;
    resp.pool = Some(exchange.pair_contract(&mut [offer_asset, ask_asset]));
    to_binary(&resp).map_err(From::from)
}
