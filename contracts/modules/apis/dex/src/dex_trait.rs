use abstract_os::objects::{DexAssetPairing, PoolId, PoolReference};
use abstract_sdk::feature_objects::AnsHost;
use abstract_sdk::os::objects::AssetEntry;
use cosmwasm_std::{CosmosMsg, Decimal, Deps, Uint128};
use cw_asset::{Asset, AssetInfo};

use crate::error::DexError;

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

pub trait Identify {
    fn over_ibc(&self) -> bool;
    fn name(&self) -> &'static str;
}

/// DEX ensures supported dexes support the expected functionality.
/// Trait that implements the actual dex interaction.
pub trait DEX: Identify {
    fn pair_address(
        &self,
        deps: Deps,
        ans_host: &AnsHost,
        mut assets: Vec<AssetEntry>,
    ) -> Result<PoolId, DexError> {
        let dex_pair =
            DexAssetPairing::new(assets.pop().unwrap(), assets.pop().unwrap(), self.name());
        let mut pool_ref = ans_host.query_asset_pairing(&deps.querier, &dex_pair)?;
        let found: PoolReference = pool_ref.pop().ok_or(DexError::AssetPairingNotFound {
            asset_pairing: dex_pair,
        })?;
        Ok(found.pool_id)
    }
    #[allow(clippy::too_many_arguments)]
    fn swap(
        &self,
        deps: Deps,
        pool_id: PoolId,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError>;
    fn custom_swap(
        &self,
        _deps: Deps,
        _offer_assets: Vec<Asset>,
        _ask_assets: Vec<Asset>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        // Must be implemented in the base to be available
        Err(DexError::NotImplemented(self.name().to_string()))
    }
    fn provide_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolId,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError>;
    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        pool_id: PoolId,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError>;
    // fn raw_swap();
    // fn raw_provide_liquidity();
    fn withdraw_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolId,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError>;
    // fn raw_withdraw_liquidity();
    // fn route_swap();
    // fn raw_route_swap();
    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolId,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError>;
}
