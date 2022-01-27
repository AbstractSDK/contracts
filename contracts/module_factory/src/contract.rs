use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};

use crate::error::ModuleFactoryError;
use cw2::set_contract_version;
use pandora::registery::FACTORY;

use crate::state::*;
use crate::{commands, msg::*};

pub type ModuleFactoryResult = Result<Response, ModuleFactoryError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ModuleFactoryResult {
    let config = Config {
        version_control_contract: deps.api.addr_validate(&msg.version_control_contract)?,
        memory_contract: deps.api.addr_validate(&msg.memory_contract)?,
    };

    set_contract_version(deps.storage, FACTORY, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ModuleFactoryResult {
    match msg {
        ExecuteMsg::UpdateConfig {
            admin,
            memory_contract,
            version_control_contract,
        } => commands::execute_update_config(
            deps,
            env,
            info,
            admin,
            memory_contract,
            version_control_contract,
        ),
        ExecuteMsg::CreateModule { module, init_msg } => {
            commands::execute_create_module(deps, env, info, module, init_msg)
        }
    }
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> ModuleFactoryResult {
    // match msg {
    //     Reply {
    //         id: commands::CREATE_OS_MANAGER_MSG_ID,
    //         result,
    //     } => commands::after_manager_create_treasury(deps, result),
    //     Reply {
    //         id: commands::CREATE_OS_TREASURY_MSG_ID,
    //         result,
    //     } => commands::after_treasury_add_to_manager(deps, result),
    //     _ => Err(ModuleFactoryError::UnexpectedReply {}),
    // }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let admin = ADMIN.get(deps)?.unwrap();
    let resp = ConfigResponse {
        owner: admin.into(),
        version_control_contract: state.version_control_contract.into(),
        memory_contract: state.memory_contract.into(),
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
