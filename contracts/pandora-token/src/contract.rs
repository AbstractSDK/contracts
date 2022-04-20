use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Api, Addr,
};

use cw2::set_contract_version;
use cw20_base::contract::{create_accounts, execute as cw20_execute, query as cw20_query};
use cw20_base::msg::{ExecuteMsg, QueryMsg};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};
use cw20_base::ContractError;

use pandora_os::native::version_control;
use pandora_os::util::pandora_token::{InstantiateMsg, MigrateMsg};

use crate::state::{ADMIN, Config, CONFIG};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "pandora:token";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
/// Returns the default object of type [`Response`] if the operation was successful,
/// or a [`ContractError`] if the contract was not created.
/// ## Params
/// * **deps** is the object of type [`DepsMut`].
///
/// * **_env** is the object of type [`Env`].
///
/// * **_info** is the object of type [`MessageInfo`].
/// * **msg** is a message of type [`InstantiateMsg`] which contains the basic settings for creating a contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Check valid token info
    msg.validate()?;

    // Create initial accounts
    let total_supply = create_accounts(&mut deps, msg.initial_balances.as_slice())?;

    // Check supply cap
    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap"));
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: addr_validate_to_lower(deps.api, &m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // Store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };

    TOKEN_INFO.save(deps.storage, &data)?;

    //Custom logic

    let config = Config{
        transfers_restricted: true,
        version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
        whitelisted_addr: vec![],
    };

    CONFIG.save(deps.storage, &config)?;

    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match &msg {
    ExecuteMsg::Transfer { recipient, .. } |
    ExecuteMsg::Send { contract: recipient, .. } |
    ExecuteMsg::TransferFrom { recipient, .. } |
    ExecuteMsg::SendFrom { contract: recipient, .. } => {
        assert_recipient_allowed(deps.as_ref(), recipient)?;
    },
    _ => ()
}

    cw20_execute(deps, env, info, msg)
}

fn assert_recipient_allowed(deps: Deps, recipient: &str) -> Result<(), ContractError> {
    // is recipient a whitelisted? 
    let config = CONFIG.load(deps.storage)?;
    if config.whitelisted_addr.contains(&deps.api.addr_validate(recipient)?) {
        return Ok(())
    }

    version_control::queries::try_raw_os_manager_query(deps, &config.version_control_address,  )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_query(deps, env, msg)
}

/// ## Description
/// Used for migration of contract. Returns the default object of type [`Response`].
/// ## Params
/// * **_deps** is the object of type [`DepsMut`].
///
/// * **_env** is the object of type [`Env`].
///
/// * **_msg** is the object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

fn addr_validate_to_lower(api: &dyn Api, addr: &str) -> StdResult<Addr> {
    if addr.to_lowercase() != addr {
        return Err(StdError::generic_err(format!(
            "Address {} should be lowercase",
            addr
        )));
    }
    api.addr_validate(addr)
}