use crate::contract::{HostResponse, HostResult, IbcHostResult, CONTRACT_VERSION};
use abstract_core::{
    objects::module_version::{get_module_data, set_module_data},
    IBC_HOST,
};
use abstract_sdk::{
    base::{Handler, MigrateEndpoint},
    core::ibc_host::MigrateMsg,
};
use cosmwasm_std::{Response, StdError};
use cw2::set_contract_version;
use schemars::JsonSchema;
use semver::Version;
use serde::Serialize;

pub fn migrate(deps: cosmwasm_std::DepsMut, env: cosmwasm_std::Env, msg: MigrateMsg) -> HostResult {
    let version: Version =
        Version::parse(CONTRACT_VERSION).map_err(|e| StdError::generic_err(e.to_string()))?;
    let storage_version: Version = get_module_data(deps.storage)?.version.parse().unwrap();
    if storage_version < version {
        set_contract_version(deps.storage, IBC_HOST, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}
