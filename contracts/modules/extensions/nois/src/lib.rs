use cosmwasm_schema::cw_serde;
use cosmos_nois::NoisCallback;

pub mod contract;
pub mod error;
mod handlers;

#[cw_serde]
pub struct NoisReceiveMsg {
    callback: NoisCallback,
}

// TODO: FIX
// #[cfg(test)]
// #[cfg(not(target_arch = "wasm32"))]
// mod tests;
