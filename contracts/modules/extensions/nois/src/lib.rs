use cosmos_nois::NoisCallback;
use cosmwasm_schema::cw_serde;

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
