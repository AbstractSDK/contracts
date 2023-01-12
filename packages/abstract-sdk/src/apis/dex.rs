// TODO: this should be moved to the public dex package
// It cannot be in abstract-os because it does not have a dependency on sdk (as it shouldn't)
use crate::base::features::{Dependencies, Identification};
use crate::ModuleInterface;
use abstract_os::dex::{
    AskAsset, DexAction, DexExecuteMsg, DexName, DexQueryMsg, OfferAsset, SimulateSwapResponse,
    SwapRouter,
};
use abstract_os::objects::AssetEntry;
use abstract_os::EXCHANGE;
use cosmwasm_std::{CosmosMsg, Decimal, Deps, StdResult, Uint128};
use serde::de::DeserializeOwned;

/// Interact with the dex api in your module.
pub trait DexInterface: Identification + Dependencies {
    fn dex<'a>(&'a self, deps: Deps<'a>) -> Dex<Self> {
        Dex { base: self, deps }
    }
}

impl<T> DexInterface for T where T: Identification + Dependencies {}

#[derive(Clone)]
pub struct Dex<'a, T: DexInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: DexInterface> Dex<'a, T> {
    fn request(&self, dex: DexName, action: DexAction) -> StdResult<CosmosMsg> {
        let modules = self.base.modules(self.deps);

        modules.api_request(EXCHANGE, DexExecuteMsg { dex, action })
    }

    pub fn swap(
        &self,
        dex: DexName,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    ) -> StdResult<CosmosMsg> {
        self.request(
            dex,
            DexAction::Swap {
                offer_asset,
                ask_asset,
                belief_price,
                max_spread,
            },
        )
    }

    pub fn custom_swap(
        &self,
        dex: DexName,
        offer_assets: Vec<OfferAsset>,
        ask_assets: Vec<AskAsset>,
        max_spread: Option<Decimal>,
        router: Option<SwapRouter>,
    ) -> StdResult<CosmosMsg> {
        self.request(
            dex,
            DexAction::CustomSwap {
                offer_assets,
                ask_assets,
                max_spread,
                router,
            },
        )
    }

    pub fn provide_liquidity(
        &self,
        dex: DexName,
        assets: Vec<OfferAsset>,
        max_spread: Option<Decimal>,
    ) -> StdResult<CosmosMsg> {
        self.request(dex, DexAction::ProvideLiquidity { assets, max_spread })
    }

    pub fn provide_liquidity_symmetric(
        &self,
        dex: DexName,
        offer_asset: OfferAsset,
        paired_assets: Vec<AssetEntry>,
    ) -> StdResult<CosmosMsg> {
        self.request(
            dex,
            DexAction::ProvideLiquiditySymmetric {
                offer_asset,
                paired_assets,
            },
        )
    }

    pub fn withdraw_liquidity(
        &self,
        dex: DexName,
        lp_token: AssetEntry,
        amount: Uint128,
    ) -> StdResult<CosmosMsg> {
        self.request(dex, DexAction::WithdrawLiquidity { lp_token, amount })
    }
}

impl<'a, T: DexInterface> Dex<'a, T> {
    fn query<R: DeserializeOwned>(&self, query_msg: DexQueryMsg) -> StdResult<R> {
        let modules = self.base.modules(self.deps);
        modules.query_api(EXCHANGE, query_msg)
    }
    pub fn simulate_swap(
        &self,
        dex: Option<DexName>,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
    ) -> StdResult<SimulateSwapResponse> {
        let response: SimulateSwapResponse = self.query(DexQueryMsg::SimulateSwap {
            dex,
            offer_asset,
            ask_asset,
        })?;
        Ok(response)
    }
}
