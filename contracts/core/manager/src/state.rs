use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

pub const ADMIN: Admin = Admin::new("admin");
pub const ROOT: Admin = Admin::new("root");
pub const VC_ADDRESS: Item<String> = Item::new("\u{0}{7}vc_addr");
pub const NEW_MODULE: Item<String> = Item::new("\u{0}{10}new_module");
pub const OS_ID: Item<u32> = Item::new("\u{0}{5}os_id");
pub const OS_MODULES: Map<&str, String> = Map::new("os_modules");
