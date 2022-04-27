use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DappInstantiateMsg {
    /// Used by Module Factory to instantiate dApp
    pub memory_address: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DappExecuteMsg {
    /// Updates the base config
    /// Sets new values for the provided options
    UpdateConfig { proxy_address: Option<String> },
    /// Adds/removes traders
    /// If a trader is both in to_add and to_remove, it will be removed.
    UpdateTraders {
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}
