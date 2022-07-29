use std::convert::TryFrom;

use cosmwasm_std::{StdError, Deps, Addr, Decimal};
use cw_asset::Asset;

use crate::{error::DexError, contract::DexApi};

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

pub trait DEX {
    fn swap(&self, deps: Deps, api: DexApi, contract_address: Addr, offer_asset: Asset, belief_price: Option<Decimal>, max_spread: Option<Decimal>);
    // fn raw_swap();
    // fn provide_liquidity();
    // fn raw_provide_liquidity();
    // fn withdraw_liquidity();
    // fn raw_withdraw_liquidity();
    // fn route_swap();
    // fn raw_route_swap();
}