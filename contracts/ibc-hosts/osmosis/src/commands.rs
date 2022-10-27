use abstract_os::dex::OfferAsset;
use cosmwasm_std::Decimal;

use crate::contract::IbcHostResult;
#[allow(unused)]
pub fn handle_matrix_swap(
    _offer_assets: Vec<OfferAsset>,
    _ask_assets: Vec<OfferAsset>,
    _max_spread: Option<Decimal>,
) -> IbcHostResult {
    // the assets have alneady been transfered
    todo!()
}

#[allow(unused)]
pub fn handle_default_swap(
    _offer_assets: Vec<OfferAsset>,
    _ask_assets: Vec<OfferAsset>,
    _max_spread: Option<Decimal>,
) -> IbcHostResult {
    // if let Some(router) = router {
    //   match router {
    //     SwapRouter::Matrix => todo!(),
    //     SwapRouter::Custom(address) => todo!(),
    //   }
    // } else {
    //   // default swap
    //
    // }
    todo!()
}
