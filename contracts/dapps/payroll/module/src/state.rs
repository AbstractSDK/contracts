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
    pub total_weight: Uint128,
    pub next_pay_day: Uint64,
}

pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");

pub const CUSTOMERS: PagedMap<Deposit> = PagedMap::new("\u{0}{11}paging_info", "customers");
// List of nft ids that already got paid
pub const PAID_CONTRIBUTORS: Item<Vec<String>> = Item::new("\u{0}{17}paid_contributors");
