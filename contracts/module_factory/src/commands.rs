use cosmwasm_std::{
    to_binary, Binary, DepsMut, Env, MessageInfo, QueryRequest, ReplyOn, Response, SubMsg, WasmMsg,
    WasmQuery,
};

use cw2::ContractVersion;

use pandora::manager::queries::query_os_id;
use pandora::modules::{Module, ModuleInfo, ModuleInitMsg, ModuleKind};
use pandora::version_control::queries::try_raw_os_manager_query;

use crate::contract::ModuleFactoryResult;

use crate::error::ModuleFactoryError;

use crate::state::*;
use pandora::manager::msg::{ConfigQueryResponse, QueryMsg as ManagerQuery};

use pandora::version_control::msg::{CodeIdResponse, QueryMsg as VCQuery};

pub const CREATE_INTERNAL_DAPP_RESPONSE_ID: u64 = 1u64;
pub const CREATE_EXTERNAL_DAPP_RESPONSE_ID: u64 = 2u64;
pub const CREATE_SERVICE_RESPONSE_ID: u64 = 3u64;
pub const CREATE_PERK_RESPONSE_ID: u64 = 4u64;

/// Function that starts the creation of the OS
pub fn execute_create_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut module: Module,
    root_init_msg: Option<Binary>,
) -> ModuleFactoryResult {
    let config = CONFIG.load(deps.storage)?;
    // Check if caller is manager of registered OS
    let os_id = query_os_id(deps.as_ref(), &info.sender)?;

    let os_manager_addr =
        try_raw_os_manager_query(deps.as_ref(), &config.version_control_contract, os_id);
    match os_manager_addr {
        Ok(addr) => {
            if !info.sender.eq(&addr) {
                return Err(ModuleFactoryError::UnknownCaller());
            }
        }
        Err(_) => return Err(ModuleFactoryError::UnknownCaller()),
    };

    // Get root user of that OS
    let _manager_config_response: ConfigQueryResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.version_control_contract.to_string(),
            msg: to_binary(&ManagerQuery::QueryOsConfig {})?,
        }))?;

    // Query version_control for code_id Module
    let module_code_id_response: CodeIdResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.version_control_contract.to_string(),
            msg: to_binary(&VCQuery::QueryCodeId {
                module: module.info,
            })?,
        }))?;

    // Update module info
    module.info = ModuleInfo::from(module_code_id_response.info.clone());
    // Get factory binary
    let ContractVersion { contract, version } = &module_code_id_response.info;
    let fixed_binairy = MODULE_INIT_BINARIES.may_load(deps.storage, (contract, version))?;
    let init_msg = ModuleInitMsg {
        fixed_init: fixed_binairy,
        root_init: root_init_msg,
    }
    .format()?;

    // Match Module type
    match module {
        Module {
            kind: ModuleKind::External,
            ..
        } => create_external_dapp(
            deps,
            env,
            module_code_id_response.code_id.u64(),
            init_msg,
            module,
        ),
        Module {
            kind: ModuleKind::Internal,
            ..
        } => create_internal_dapp(
            deps,
            env,
            module_code_id_response.code_id.u64(),
            init_msg,
            module,
        ),
        Module {
            kind: ModuleKind::Service,
            ..
        } => create_service(
            deps,
            env,
            module_code_id_response.code_id.u64(),
            init_msg,
            module,
        ),
        Module {
            kind: ModuleKind::Perk,
            ..
        } => create_perk(
            deps,
            env,
            module_code_id_response.code_id.u64(),
            init_msg,
            module,
        ),
    }
}

pub fn create_internal_dapp(
    _deps: DepsMut,
    env: Env,
    code_id: u64,
    init_msg: Binary,
    module: Module,
) -> ModuleFactoryResult {
    let response = Response::new();

    Ok(response
        .add_attributes(vec![
            ("action", "create internal dapp"),
            ("initmsg:", &init_msg.to_string()),
        ])
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_INTERNAL_DAPP_RESPONSE_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id,
                funds: vec![],
                // This contract should be able to migrate the contract
                admin: Some(env.contract.address.to_string()),
                label: format!("Module: --{}--", module),
                msg: init_msg,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

// Todo: review if we want external dapps to remain per-os instantiated
pub fn create_external_dapp(
    _deps: DepsMut,
    env: Env,
    code_id: u64,
    init_msg: Binary,
    module: Module,
) -> ModuleFactoryResult {
    let response = Response::new();

    Ok(response
        .add_attributes(vec![
            ("action", "create external dapp"),
            ("initmsg:", &init_msg.to_string()),
        ])
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_EXTERNAL_DAPP_RESPONSE_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id,
                funds: vec![],
                // This contract should be able to migrate the contract
                admin: Some(env.contract.address.to_string()),
                label: format!("Module: --{}--", module),
                msg: init_msg,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

pub fn create_perk(
    _deps: DepsMut,
    _env: Env,
    code_id: u64,
    init_msg: Binary,
    module: Module,
) -> ModuleFactoryResult {
    let response = Response::new();

    Ok(response
        .add_attributes(vec![
            ("action", "create perk"),
            ("initmsg:", &init_msg.to_string()),
        ])
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_PERK_RESPONSE_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id,
                funds: vec![],
                // Not migratable
                admin: None,
                label: format!("Module: --{}--", module),
                msg: init_msg,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

pub fn create_service(
    _deps: DepsMut,
    env: Env,
    code_id: u64,
    init_msg: Binary,
    module: Module,
) -> ModuleFactoryResult {
    let response = Response::new();

    Ok(response
        .add_attributes(vec![
            ("action", "create service"),
            ("initmsg:", &init_msg.to_string()),
        ])
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_SERVICE_RESPONSE_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id,
                funds: vec![],
                // This contract should be able to migrate the contract
                admin: Some(env.contract.address.to_string()),
                label: format!("Module: --{}--", module),
                msg: init_msg,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

// /// Registers the DAO on the version_control contract and
// /// instantiates the Treasury contract of the newly created DAO
// pub fn after_manager_create_treasury(
//     deps: DepsMut,
//     result: ContractResult<SubMsgExecutionResponse>,
// ) -> ModuleFactoryResult {
//     let config = CONFIG.load(deps.storage)?;

//     // Get address of Manager contract
//     let res: MsgInstantiateContractResponse =
//         Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
//             StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
//         })?;
//     let manager_address = res.get_contract_address();

//     // Add OS to version_control
//     let response = Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: config.version_control_contract.to_string(),
//         funds: vec![],
//         msg: to_binary(&VCExecuteMsg::AddOs {
//             os_id: config.os_id_sequence,
//             os_manager_address: manager_address.to_string(),
//         })?,
//     }));

//     // Query version_control for code_id of Treasury
//     // TODO: replace with raw-query from package.
//     let treasury_code_id_response: CodeIdResponse =
//         deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
//             contract_addr: config.version_control_contract.to_string(),
//             msg: to_binary(&VCQuery::QueryCodeId {
//                 module: String::from(TREASURY),
//                 version: None,
//             })?,
//         }))?;

//     Ok(response
//         .add_attribute("Manager Address:", &manager_address.to_string())
//         // Instantiate Treasury contract
//         .add_submessage(SubMsg {
//             id: CREATE_OS_TREASURY_MSG_ID,
//             gas_limit: None,
//             msg: WasmMsg::Instantiate {
//                 code_id: treasury_code_id_response.code_id.u64(),
//                 funds: vec![],
//                 admin: Some(manager_address.to_string()),
//                 label: format!("Treasury of OS: {}", config.os_id_sequence),
//                 msg: to_binary(&TreasuryInstantiateMsg {})?,
//             }
//             .into(),
//             reply_on: ReplyOn::Success,
//         }))
// }

// /// Adds treasury contract address and name to Manager
// /// contract of OS
// pub fn after_treasury_add_to_manager(
//     deps: DepsMut,
//     result: ContractResult<SubMsgExecutionResponse>,
// ) -> ModuleFactoryResult {
//     let mut config = CONFIG.load(deps.storage)?;

//     let res: MsgInstantiateContractResponse =
//         Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
//             StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
//         })?;

//     // TODO: Should we store the manager address in the local state between the previous step and this?
//     // Get address of manager
//     let manager_address: String = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
//         contract_addr: config.version_control_contract.to_string(),
//         msg: to_binary(&VCQuery::QueryOsAddress {
//             os_id: config.os_id_sequence,
//         })?,
//     }))?;

//     // Update id sequence
//     config.os_id_sequence += 1;
//     CONFIG.save(deps.storage, &config)?;

//     Ok(Response::new()
//         .add_attribute("Treasury Address: ", res.get_contract_address())
//         .add_message(register_module_on_manager(
//             manager_address,
//             TREASURY.to_string(),
//             res.get_contract_address().to_string(),
//         )?))
// }

// Only owner can execute it
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    memory_contract: Option<String>,
    version_control_contract: Option<String>,
) -> ModuleFactoryResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config: Config = CONFIG.load(deps.storage)?;

    if let Some(memory_contract) = memory_contract {
        // validate address format
        config.memory_contract = deps.api.addr_validate(&memory_contract)?;
    }

    if let Some(version_control_contract) = version_control_contract {
        // validate address format
        config.version_control_contract = deps.api.addr_validate(&version_control_contract)?;
    }

    CONFIG.save(deps.storage, &config)?;

    if let Some(admin) = admin {
        let addr = deps.api.addr_validate(&admin)?;
        ADMIN.set(deps, Some(addr))?;
    }

    Ok(Response::new().add_attribute("action", "update_config"))
}
