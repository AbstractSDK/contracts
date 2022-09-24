//! # Version Control
//!
//! `abstract_os::version_control` stores chain-specific code-ids, addresses and an os_id map.
//!
//! ## Description
//! Code-ids and api-contract addresses are stored on this address. This data can not be changed and allows for complex factory logic.
//! Both code-ids and addresses are stored on a per-module version basis which allows users to easily upgrade their modules.
//!
//! An internal os-id store provides external verification for manager and proxy addresses.  

pub mod state {
    use cw_controllers::Admin;
    use cw_storage_plus::Map;

    use crate::objects::{module::{ModuleInfo}, module_reference::ModuleReference};

    use super::Core;

    pub const ADMIN: Admin = Admin::new("admin");
    pub const FACTORY: Admin = Admin::new("factory");

    // We can iterate over the map giving just the prefix to get all the versions
    pub const MODULE_LIBRARY: Map<ModuleInfo,ModuleReference> = Map::new("module_lib");
    /// Maps OS ID to the address of its core contracts
    pub const OS_ADDRESSES: Map<u32, Core> = Map::new("os_core");
}

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::objects::{module::{ModuleInfo, Module}, module_reference::ModuleReference};

/// Contains the minimal Abstract-OS contract addresses.
#[cosmwasm_schema::cw_serde]
pub struct Core {
    pub manager: Addr,
    pub proxy: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Remove some version of a module
    RemoveModule { module: ModuleInfo },
    /// Add new modules
    AddModules {
        modules: Vec<(ModuleInfo, ModuleReference)>,
    },
    /// Add a new OS to the deployed OSs.  
    /// Only Factory can call this
    AddOs { os_id: u32, core: Core },
    /// Sets a new Admin
    SetAdmin { new_admin: String },
    /// Sets a new Factory
    SetFactory { new_factory: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
pub enum QueryMsg {
    /// Query Core of an OS
    /// Returns [`OsCoreResponse`]
    #[returns(OsCoreResponse)]
    OsCore { os_id: u32 },
    /// Queries api addresses
    /// Returns [`ModuleResponse`]
    #[returns(ModuleResponse)]
    ModuleReference { module: ModuleInfo },
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns [`ModulesResponse`]
    #[returns(ModulesResponse)]
    ModuleReferences {
        page_token: Option<ModuleInfo>,
        page_size: Option<u8>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct OsCoreResponse {
    pub os_core: Core,
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleResponse {
    pub module: Module,
}

#[cosmwasm_schema::cw_serde]
pub struct ModulesResponse {
    pub modules: Vec<(ModuleInfo,ModuleReference)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub factory: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
