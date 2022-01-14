use cosmwasm_std::{
    to_binary, Binary, DepsMut, Env, MessageInfo, QueryRequest, ReplyOn, Response, SubMsg, WasmMsg,
    WasmQuery,
};
use dao_os::version_control::msg::CodeIdResponse;

use crate::contract::ManagerResult;
use crate::error::ManagerError;
use crate::state::*;
use dao_os::manager::msg::ExecuteMsg;
// use semver::Version;
use dao_os::version_control::msg::QueryMsg as VCQuery;

pub const DAPP_CREATE_ID: u64 = 1u64;

pub fn handle_message(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    message: ExecuteMsg,
) -> ManagerResult {
    match message {
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
        ExecuteMsg::UpdateConfig { vc_addr, root,  } => execute_update_config(deps, info, vc_addr, root),
        ExecuteMsg::UpdateModuleAddresses { to_add, to_remove } => {
            // Only Admin can call this method
            ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
            update_module_addresses(deps, to_add, to_remove)
        }
        ExecuteMsg::AddInternalDapp {
            module,
            version,
            init_msg,
        } => add_internal_dapp(deps, info, env, module, version, init_msg),
    }
}

/// Adds, updates or removes provided addresses.
/// Should only be called by contract that adds/removes modules.
/// Factory is admin on init
/// TODO: Add functionality to version_control (or some other contract) to add and upgrade contracts.
pub fn update_module_addresses(
    deps: DepsMut,
    to_add: Vec<(String, String)>,
    to_remove: Vec<String>,
) -> ManagerResult {
    for (name, new_address) in to_add.into_iter() {
        // validate addr
        deps.as_ref().api.addr_validate(&new_address)?;
        OS_MODULES.save(deps.storage, name.as_str(), &new_address)?;
    }

    for name in to_remove {
        OS_MODULES.remove(deps.storage, name.as_str());
    }

    Ok(Response::new().add_attribute("action", "update OS module addresses"))
}

pub fn add_internal_dapp(
    deps: DepsMut,
    msg_info: MessageInfo,
    env: Env,
    module: String,
    _version: Option<String>,
    init_msg: Binary,
) -> ManagerResult {
    // Only Root can call this method
    ROOT.assert_admin(deps.as_ref(), &msg_info.sender)?;

    // Check if dapp is already enabled.
    match OS_MODULES.may_load(deps.storage, &module)? {
        Some(_) => return Err(ManagerError::InternalDappAlreadyAdded {}),
        None => (),
    };

    // https://github.com/CosmWasm/cosmwasm/blob/879465910cb0958195e51707cb2b3412de302bbd/packages/vm/src/serde.rs

    let vc_addr = VC_ADDRESS.load(deps.storage)?;
    // Query version_control for code_id of Manager contract
    // Replace with query to get latest version if applicable
    let module_code_id_response: CodeIdResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vc_addr,
            msg: to_binary(&VCQuery::QueryCodeId {
                module: module.clone(),
                version: "v0.1.0".to_string(),
            })?,
        }))?;

    // Save module name for use in Reply
    NEW_MODULE.save(deps.storage, &module)?;

    let response = Response::new().add_submessage(SubMsg {
        id: DAPP_CREATE_ID,
        gas_limit: None,
        msg: WasmMsg::Instantiate {
            code_id: module_code_id_response.code_id.u64(),
            funds: vec![],
            admin: Some(env.contract.address.to_string()),
            label: format!("CosmWasm OS dApp: {}", module),
            msg: init_msg,
        }
        .into(),
        reply_on: ReplyOn::Success,
    });

    Ok(response)
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> ManagerResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}

// Only owner can execute it
pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    version_control_contract: Option<String>,
    root: Option<String>,
) -> ManagerResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    if let Some(version_control_contract) = version_control_contract {
        deps.api.addr_validate(&version_control_contract)?;
        VC_ADDRESS.save(deps.storage, &version_control_contract)?;
    }

    if let Some(root) = root {
        let addr = deps.api.addr_validate(&root)?;
        ROOT.set(deps, Some(addr))?;
    }

    Ok(Response::new().add_attribute("action", "update_config"))
}