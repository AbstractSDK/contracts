use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use pandora::{deposit_manager::Deposit, paged_map::PagedMap};
use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub token_cap: Uint128,
    pub ratio: Decimal,
    pub payment_asset: AssetInfo,
    pub subscription_cost: Uint64,
    pub project_token: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub income: Uint64,
    pub target: Uint64,
    pub expense: Uint64,
    pub total_weight: Uint128,
    pub next_pay_day: Uint64,
    pub debtors: Vec<u32>,
    pub expense_ratio: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Compensation {
    pub base: u32,
    pub weight: u32,
    pub next_pay_day: Uint64,
    pub expiration: Uint64,
    pub mint_price_factor: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct IncomeAccumulator {
    pub income: u32,
    pub debtors: Vec<u32>,
}

pub const MONTH: u64 = 60 * 60 * 24 * 30;
pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");

// List clients
pub const CLIENTS: PagedMap<Deposit, IncomeAccumulator> =
    PagedMap::new("clients", "clients_status");
// List contributors
pub const CONTRIBUTORS: Map<&str, Compensation> = Map::new("contributors");
