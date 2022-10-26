use abstract_os::dex::{OfferAsset, SwapRouter};
use cosmwasm_std::Decimal;

use crate::contract::IbcHostResult;

use osmosis_std::types::osmosis::tokenfactory::v1beta1::{
    MsgCreateDenom, MsgMint, TokenfactoryQuerier,
};

pub fn handle_matrix_swap(
    offer_assets: Vec<OfferAsset>,
    ask_assets: Vec<OfferAsset>,
    max_spread: Option<Decimal>,
) -> IbcHostResult {
    // the sasets have alneady been transfered
    todo!()
}

pub fn handle_default_swap(
  offer_assets: Vec<OfferAsset>,
  ask_assets: Vec<OfferAsset>,
  max_spread: Option<Decimal>,
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


