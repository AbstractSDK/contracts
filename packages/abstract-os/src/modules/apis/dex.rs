//! # Decentralized Exchange Api
//!
//! `abstract_os::dex` is a generic dex-interfacing contract that handles address retrievals and dex-interactions.

use crate::objects::DexAssetPairing;
use crate::{
    api::{self},
    objects::{AnsAsset, AssetEntry},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Uint128};

pub type DexName = String;
pub type OfferAsset = AnsAsset;
pub type AskAsset = AnsAsset;

pub mod state {
    use cw_storage_plus::Item;

    use crate::objects::fee::UsageFee;

    pub const SWAP_FEE: Item<UsageFee> = Item::new("swap_fee");
}

pub const IBC_DEX_ID: u32 = 11335;

pub type ExecuteMsg = api::ExecuteMsg<DexExecuteMsg>;
pub type QueryMsg = api::QueryMsg<DexQueryMsg>;

impl api::ApiExecuteMsg for DexExecuteMsg {}
impl api::ApiQueryMsg for DexQueryMsg {}

#[cosmwasm_schema::cw_serde]
pub struct DexInstantiateMsg {
    pub swap_fee: Decimal,
    pub recipient_os: u32,
}

/// Dex Execute msg
#[cosmwasm_schema::cw_serde]
// Struct messages not yet supported by BOOT
pub struct DexExecuteMsg {
    pub dex: DexName,
    pub action: DexAction,
}

#[cosmwasm_schema::cw_serde]
/// Possible actions to perform on the DEX
pub enum DexAction {
    /// Provide arbitrary liquidity
    ProvideLiquidity {
        // support complex pool types
        /// Assets to add
        assets: Vec<OfferAsset>,
        max_spread: Option<Decimal>,
    },
    /// Provide liquidity equally between assets to a pool
    ProvideLiquiditySymmetric {
        offer_asset: OfferAsset,
        // support complex pool types
        /// Assets that are paired with the offered asset
        paired_assets: Vec<AssetEntry>,
    },
    /// Withdraw liquidity from a pool
    WithdrawLiquidity {
        lp_token: AssetEntry,
        amount: Uint128,
    },
    /// Standard swap between one asset to another
    Swap {
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    },
    /// Allow alternative swap routers and methods
    CustomSwap {
        offer_assets: Vec<OfferAsset>,
        ask_assets: Vec<AskAsset>,
        max_spread: Option<Decimal>,
        /// Optionally supply a router to use
        router: Option<SwapRouter>,
    },
}

#[cosmwasm_schema::cw_serde]
pub enum SwapRouter {
    /// Matrix router
    Matrix,
    /// Use a custom router (using String type for cross-chain compatibility)
    Custom(String),
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
#[cfg_attr(feature = "boot", impl_into(QueryMsg))]
pub enum DexQueryMsg {
    #[returns(SimulateSwapResponse)]
    SimulateSwap {
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        dex: Option<DexName>,
    },
}

// LP/protocol fees could be withheld from either input or output so commission asset must be included.
#[cosmwasm_schema::cw_serde]
pub struct SimulateSwapResponse {
    pub pool: DexAssetPairing,
    /// Amount you would receive when performing the swap.
    pub return_amount: Uint128,
    /// Spread in ask_asset for this swap
    pub spread_amount: Uint128,
    /// Commission charged for the swap
    pub commission: (AssetEntry, Uint128),
    /// API fee charged for the swap (paid in offer asset)
    pub api_fee: Uint128,
}
