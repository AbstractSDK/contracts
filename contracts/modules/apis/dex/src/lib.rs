

mod exchanges;
pub (crate) mod dex_trait;
pub (crate) mod commands;
pub mod contract;
pub mod error;

pub use dex_trait::DEX;

// TODO: FIX
// #[cfg(test)]
// #[cfg(not(target_arch = "wasm32"))]
// mod tests;
