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
use os::manager::state::ModuleId;
use serde::de::DeserializeOwned;

/// Interact with the dex api in your module.
pub trait DexInterface: Identification + Dependencies {
    /// Construct a new dex interface
    /// Params:
    /// - deps: the deps object
    /// - dex_name: the name of the dex to interact with
    fn dex<'a>(&'a self, deps: Deps<'a>, dex_name: DexName) -> Dex<Self> {
        Dex {
            base: self,
            deps,
            dex_name,
            dex_module_id: EXCHANGE,
        }
    }
}

impl<T> DexInterface for T where T: Identification + Dependencies {}

#[derive(Clone)]
pub struct Dex<'a, T: DexInterface> {
    base: &'a T,
    dex_name: DexName,
    dex_module_id: ModuleId<'a>,
    deps: Deps<'a>,
}

impl<'a, T: DexInterface> Dex<'a, T> {
    /// Set the module id for the
    pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
        Self {
            dex_module_id: module_id,
            ..self
        }
    }
    fn dex_name(&self) -> DexName {
        self.dex_name.clone()
    }
    fn dex_module_id(&self) -> ModuleId {
        self.dex_module_id
    }
    fn request(&self, action: DexAction) -> StdResult<CosmosMsg> {
        let modules = self.base.modules(self.deps);

        modules.api_request(
            self.dex_module_id(),
            DexExecuteMsg {
                dex: self.dex_name(),
                action,
            },
        )
    }

    pub fn swap(
        &self,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    ) -> StdResult<CosmosMsg> {
        self.request(DexAction::Swap {
            offer_asset,
            ask_asset,
            belief_price,
            max_spread,
        })
    }

    pub fn custom_swap(
        &self,
        offer_assets: Vec<OfferAsset>,
        ask_assets: Vec<AskAsset>,
        max_spread: Option<Decimal>,
        router: Option<SwapRouter>,
    ) -> StdResult<CosmosMsg> {
        self.request(DexAction::CustomSwap {
            offer_assets,
            ask_assets,
            max_spread,
            router,
        })
    }

    pub fn provide_liquidity(
        &self,
        assets: Vec<OfferAsset>,
        max_spread: Option<Decimal>,
    ) -> StdResult<CosmosMsg> {
        self.request(DexAction::ProvideLiquidity { assets, max_spread })
    }

    pub fn provide_liquidity_symmetric(
        &self,
        offer_asset: OfferAsset,
        paired_assets: Vec<AssetEntry>,
    ) -> StdResult<CosmosMsg> {
        self.request(DexAction::ProvideLiquiditySymmetric {
            offer_asset,
            paired_assets,
        })
    }

    pub fn withdraw_liquidity(
        &self,
        lp_token: AssetEntry,
        amount: Uint128,
    ) -> StdResult<CosmosMsg> {
        self.request(DexAction::WithdrawLiquidity { lp_token, amount })
    }
}

impl<'a, T: DexInterface> Dex<'a, T> {
    fn query<R: DeserializeOwned>(&self, query_msg: DexQueryMsg) -> StdResult<R> {
        let modules = self.base.modules(self.deps);
        modules.query_api(EXCHANGE, query_msg)
    }
    pub fn simulate_swap(
        &self,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
    ) -> StdResult<SimulateSwapResponse> {
        let response: SimulateSwapResponse = self.query(DexQueryMsg::SimulateSwap {
            dex: Some(self.dex_name()),
            offer_asset,
            ask_asset,
        })?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::apis::test_common::*;
    use abstract_os::objects::dependency::StaticDependency;

    #[test]
    fn swap_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::querier();
        let stub = MockModule::new();
        let dex = stub
            .dex(deps.as_ref(), "junoswap".into())
            .with_module_id(abstract_testing::TEST_MODULE_ID);

        let dex_name = "junoswap".to_string();
        let offer_asset = OfferAsset::new("juno", 1000u128);
        let ask_asset = AssetEntry::new("uusd");
        let max_spread = Some(Decimal::percent(1));
        let belief_price = Some(Decimal::percent(2));

        let expected = DexExecuteMsg {
            dex: dex_name.clone(),
            action: DexAction::Swap {
                offer_asset: offer_asset.clone(),
                ask_asset: ask_asset.clone(),
                max_spread: max_spread.clone(),
                belief_price: belief_price.clone(),
            },
        };

        let actual = dex.swap(offer_asset, ask_asset, max_spread, belief_price);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected =
            wasm_execute(abstract_testing::TEST_MODULE_ADDRESS, &expected, vec![]).unwrap();

        assert_that!(actual).is_equal_to(expected);
    }
}
