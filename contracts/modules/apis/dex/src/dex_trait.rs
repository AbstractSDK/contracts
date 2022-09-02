use abstract_os::{
    dex::SimulateSwapResponse,
    objects::{AssetEntry, ContractEntry},
};
use abstract_sdk::MemoryOperation;
use cosmwasm_std::{Addr, Decimal, Deps, StdResult};
use cw_asset::{Asset, AssetInfo};

use crate::{
    contract::{DexApi, DexResult},
    error::DexError,
};

// pub struct Exchange<T: &dyn DEX + 'static>(pub T);

// impl TryFrom<String> for Exchange<&'static dyn DEX> {
//     type Error = DexError;

//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         match value.as_str() {
//         #[cfg(feature = "juno")]
//         JUNOSWAP => {
//             Ok(Exchange(&JunoSwap {}))
//         },
//         _ => return Err(DexError::UnknownDex(value))
//         }
//     }
// }

/// DEX trait resolves asset names and dex to pair and lp address and ensures supported dexes support swaps and liquidity provisioning.
pub trait DEX {
    fn pair_address(&self, deps: Deps, api: &DexApi, assets: &mut [AssetEntry]) -> StdResult<Addr> {
        let dex_pair = self.pair_contract(assets);
        api.resolve(deps, &dex_pair)
    }
    fn pair_contract(&self, assets: &mut [AssetEntry]) -> ContractEntry {
        ContractEntry::construct_dex_entry(self.name(), assets)
    }
    fn name(&self) -> &'static str;
    #[allow(clippy::too_many_arguments)]
    fn swap(
        &self,
        deps: Deps,
        api: DexApi,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> DexResult;
    fn provide_liquidity(
        &self,
        deps: Deps,
        api: DexApi,
        pair_address: Addr,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> DexResult;
    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        api: DexApi,
        pair_address: Addr,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> DexResult;
    // fn raw_swap();
    // fn raw_provide_liquidity();
    fn withdraw_liquidity(
        &self,
        deps: Deps,
        api: &DexApi,
        pair_address: Addr,
        lp_token: Asset,
    ) -> DexResult;
    // fn raw_withdraw_liquidity();
    // fn route_swap();
    // fn raw_route_swap();
    fn simulate_swap(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<SimulateSwapResponse, DexError>;
}
