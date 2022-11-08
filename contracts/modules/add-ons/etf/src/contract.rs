use std::vec;

use abstract_add_on::AddOnContract;

use abstract_os::add_on::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use abstract_sdk::{
    ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint, ReplyEndpoint,
};
use cosmwasm_std::{
    Addr, Binary, Deps, DepsMut, Empty, entry_point, Env, MessageInfo, Reply, ReplyOn, Response,
    StdError, StdResult, SubMsg, to_binary, WasmMsg,
};

use cw20::{Cw20ReceiveMsg, MinterResponse};

use protobuf::Message;

use abstract_os::etf::{EtfExecuteMsg, EtfInstantiateMsg, EtfQueryMsg, StateResponse};
use abstract_os::objects::fee::Fee;
use abstract_os::ETF;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;

use crate::commands::{self, receive_cw20};
use crate::error::VaultError;
use crate::replies;
use crate::replies::INSTANTIATE_REPLY_ID;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{FEE, State, STATE};

const DEFAULT_LP_TOKEN_NAME: &str = "ETF LP token";
const DEFAULT_LP_TOKEN_SYMBOL: &str = "etfLP";

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type EtfAddOn =
    AddOnContract<VaultError, EtfExecuteMsg, EtfInstantiateMsg, EtfQueryMsg, Empty, Cw20ReceiveMsg>;
pub type EtfResult = Result<Response, VaultError>;

const ETF_ADDON: EtfAddOn = EtfAddOn::new(ETF, CONTRACT_VERSION)
    .with_instantiate(instantiate_handler)
    .with_execute(execute_handler)
    .with_query(query_handler)
    .with_receive(receive_cw20)
    .with_replies(&[(INSTANTIATE_REPLY_ID, replies::instantiate_reply)]);

// Instantiate
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg<EtfInstantiateMsg>,
) -> EtfResult {
    ETF_ADDON.instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<EtfQueryMsg>) -> StdResult<Binary> {
    ETF_ADDON.query(deps, env, msg)
}

// Reply
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> EtfResult {
    ETF_ADDON.reply(deps, env, msg)
}

// Migrate
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> EtfResult {
    ETF_ADDON.migrate(deps, env, msg)
}

// Execute
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<EtfExecuteMsg, Cw20ReceiveMsg>,
) -> EtfResult {
    ETF_ADDON.execute(deps, env, info, msg)
}

// #### Handlers ####

fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    _etf: EtfAddOn,
    msg: EtfInstantiateMsg,
) -> EtfResult {
    let state: State = State {
        liquidity_token_addr: Addr::unchecked(""),
        provider_addr: deps.api.addr_validate(msg.provider_addr.as_str())?,
    };

    let lp_token_name: String = msg
        .token_name
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_NAME));

    let lp_token_symbol: String = msg
        .token_symbol
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_SYMBOL));

    STATE.save(deps.storage, &state)?;
    FEE.save(deps.storage, &Fee::new(msg.fee)?)?;

    Ok(Response::new().add_submessage(SubMsg {
        // Create LP token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: lp_token_name,
                symbol: lp_token_symbol,
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
                marketing: None,
            })?,
            funds: vec![],
            label: "White Whale Vault LP".to_string(),
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault: EtfAddOn,
    msg: EtfExecuteMsg,
) -> EtfResult {
    match msg {
        EtfExecuteMsg::ProvideLiquidity { asset } => {
            // Check asset
            let asset = asset.check(deps.api, None)?;
            commands::try_provide_liquidity(deps, info, vault, asset, None)
        }
        EtfExecuteMsg::SetFee { fee } => commands::set_fee(deps, info, vault, fee),
    }
}

fn query_handler(deps: Deps, _env: Env, _etf: &EtfAddOn, msg: EtfQueryMsg) -> StdResult<Binary> {
    match msg {
        EtfQueryMsg::State {} => {
            let fee = FEE.load(deps.storage)?;
            to_binary(&StateResponse {
                liquidity_token: STATE.load(deps.storage)?.liquidity_token_addr.to_string(),
                fee: fee.share(),
            })
        }
    }
}

