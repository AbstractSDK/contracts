use cosmwasm_std::{Binary, Uint64};
use cw2::ContractVersion;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub os_id: u32,
    pub root_user: String,
    pub vc_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Updates the Modules
    UpdateModuleAddresses {
        to_add: Vec<(String, String)>,
        to_remove: Vec<String>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
    AddInternalDapp {
        module: String,
        version: Option<String>,
        init_msg: Binary,
    },
    UpdateConfig {
        vc_addr: Option<String>,
        root: Option<String>,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Queries assets based on name
    QueryVersions {
        names: Vec<String>,
    },
    QueryModules {
        names: Vec<String>,
    },
    QueryEnabledModules {},
    /// Query OS_ID
    QueryOsConfig {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VersionsQueryResponse {
    pub versions: Vec<ContractVersion>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ModuleQueryResponse {
    pub modules: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EnabledModulesResponse {
    pub modules: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigQueryResponse {
    pub root: String,
    pub vc_addr: String,
    pub os_id: Uint64,
}