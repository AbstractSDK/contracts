use cosmwasm_std::WasmMsg;
use cw_asset::Asset;

use crate::{DEX, contract::DexApi};
pub const JUNOSWAP: &str = "junoswap";

pub struct JunoSwap{}

impl DEX for JunoSwap {
    fn swap(&self, deps: Deps, api: DexApi, offer_asset: Asset, belief_price: Option<Decimal>, max_spread: Option<Decimal>) -> WasmMsg {
        print!("test");
    }
}
