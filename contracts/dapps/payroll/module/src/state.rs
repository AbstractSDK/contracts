use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128, Uint64};
use cw_storage_plus::Item;
use pandora::{deposit_manager::Deposit, paged_map::PagedMap};
use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub target: Uint128,
    pub token_cap: Uint128,
    pub ratio: Decimal,
    pub payment_asset: AssetInfo,
    pub contributor_nft_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub income: Uint128,
    pub expense: Uint128,
    pub total_weight: Uint128,
    pub next_pay_day: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Compensation {
    pub base: u32,
    pub weight: u32,
    pub first_pay_day: Uint64,
    pub expiration: Uint64,
}

pub const MONTH: u64 = 60 * 60 * 24 * 30;
pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");

pub const CLIENTS: PagedMap<Deposit> = PagedMap::new("\u{0}{16}client_page_info", "clients");
// List contributors
pub const CONTRIBUTORS: PagedMap<Compensation> =PagedMap::new("\u{0}{22}contributors_page_info", "contributors");
