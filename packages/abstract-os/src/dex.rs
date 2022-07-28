use cosmwasm_std::{Uint128, Decimal};
use cw_asset::AssetInfoUnchecked;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::objects::memory_entry::AssetEntry;

type AssetName = String;
type DexName = String;
pub type ProvidedAsset = (AssetName,Uint128);

/// Dex Execute msg
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestMsg {
    ProvideLiquidity {
        assets: (ProvidedAsset,ProvidedAsset),
        dex: DexName,
        slippage_tolerance: Option<Decimal>,
    },
    ProvideLiquiditySymmetric {
        assets: ProvidedAsset,
        paired_asset: AssetName,
        dex: DexName,
    },
    WithdrawLiquidity {
        lp_token: AssetName,
        amount: Uint128,
    },
    Swap{
        offer_asset: ProvidedAsset,
        ask_asset: AssetName,
        dex: Option<DexName>,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    
}

