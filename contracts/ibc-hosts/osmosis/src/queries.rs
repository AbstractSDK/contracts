use std::io::Empty;
use cosmwasm_std::{Deps};
use crate::error::OsmoError;
use osmosis_std::shim::Timestamp;

use osmosis_std::types::osmosis::twap::v1beta1::{
    ArithmeticTwapToNowRequest, ArithmeticTwapToNowResponse, TwapQuerier,
};

fn handle_twap_query(
    deps: Deps,
    id: u64,
    quote_asset_denom: String,
    base_asset_denom: String,
    start_time: i64,
) -> Result<ArithmeticTwapToNowResponse, OsmoError> {
    let twap_querier = TwapQuerier::new(&deps.querier);

    let response: ArithmeticTwapToNowResponse = twap_querier.arithmetic_twap_to_now(
        id,
        base_asset_denom,
        quote_asset_denom,
        Some(Timestamp {
            seconds: start_time,
            nanos: 0,
        }),
    )?;
    Ok(response)
}
