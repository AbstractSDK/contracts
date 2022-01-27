use cosmwasm_std::{Addr, Binary};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub version_control_contract: Addr,
    pub memory_contract: Addr,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
pub const MODULE_INIT_BINARIES: Map<(&str, &str), Binary> = Map::new("module_init_binaries");
