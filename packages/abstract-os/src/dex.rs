use cosmwasm_std::{Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::objects::AssetEntry;

type DexName = String;
pub type OfferAsset = (AssetEntry,Uint128);

/// Dex Execute msg
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestMsg {
    ProvideLiquidity {
        // support complex pool types
        assets: Vec<OfferAsset>,
        dex: Option<DexName> ,
        max_spread: Option<Decimal>,
    },
    ProvideLiquiditySymmetric {
        offer_asset: OfferAsset,
        // support complex pool types
        /// Assets that are paired with the offered asset
        paired_assets: Vec<AssetEntry>,
        dex: DexName,
    },
    WithdrawLiquidity {
        lp_token: AssetEntry,
        amount: Uint128,
    },
    Swap{
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        dex: Option<DexName>,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    
}

