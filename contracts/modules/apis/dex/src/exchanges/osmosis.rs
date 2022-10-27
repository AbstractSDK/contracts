use crate::{
    dex_trait::{Fee, FeeOnInput, Return, Spread},
    error::DexError,
    DEX,
};

use cosmwasm_std::{Addr, Decimal, Deps};
use cw_asset::{Asset, AssetInfo};

pub const OSMOSIS: &str = "osmosis";
// Source https://github.com/wasmswap/wasmswap-contracts
pub struct Osmosis {}

/// Osmosis app-chain dex implementation
impl DEX for Osmosis {
    fn over_ibc(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        OSMOSIS
    }

    fn swap(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _offer_asset: Asset,
        _ask_asset: AssetInfo,
        _belief_price: Option<Decimal>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        todo!()
    }

    fn custom_swap(
        &self,
        deps: Deps,
        offer_assets: Vec<Asset>,
        ask_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        // The offer_assets have already been sent to the host contract
        // The ask_assets are the assets we want to receive
        // Generate the swap message(s) between the offer and ask assets
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn provide_liquidity(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _offer_assets: Vec<Asset>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn provide_liquidity_symmetric(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _offer_asset: Asset,
        _paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _lp_token: Asset,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn simulate_swap(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _offer_asset: Asset,
        _ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        Err(DexError::NotImplemented(self.name().to_string()))
    }
}
