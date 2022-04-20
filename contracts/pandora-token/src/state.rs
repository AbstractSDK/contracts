use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub transfers_restricted: bool,
    pub version_control_address: Addr,
    pub whitelisted_addr: Vec<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
pub const ADMIN: Admin = Admin::new("admin");