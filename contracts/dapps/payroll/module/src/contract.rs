#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use cw_storage_plus::Map;
use protobuf::Message;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use pandora::fee::Fee;
use pandora::treasury::dapp_base::commands as dapp_base_commands;

use pandora::treasury::dapp_base::common::BaseDAppResult;
use pandora::treasury::dapp_base::msg::BaseInstantiateMsg;
use pandora::treasury::dapp_base::queries as dapp_base_queries;
use pandora::treasury::dapp_base::state::{BaseState, ADMIN, BASESTATE};

use crate::response::MsgInstantiateContractResponse;

use crate::error::PayrollError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse};
use crate::state::{Config, State, CONFIG, CUSTOMERS, STATE, MONTH};
use crate::{commands, queries};
pub type PayrollResult = Result<Response, PayrollError>;

const INSTANTIATE_REPLY_ID: u8 = 1u8;

const DEFAULT_LP_TOKEN_NAME: &str = "Vault LP token";
const DEFAULT_LP_TOKEN_SYMBOL: &str = "uvLP";


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> PayrollResult {
    let base_state: BaseState = dapp_base_commands::handle_base_init(deps.as_ref(), msg.base)?;

    let config: Config = Config {
        target: msg.target,
        contributor_nft_addr: deps.api.addr_validate(&msg.contributor_nft_addr)?,
        token_cap: msg.token_cap,
        payment_asset: msg.payment_asset,
        ratio: msg.ratio,
    };

    let state: State = State {
        income: Uint128::zero(),
        expense: Uint128::zero(),
        total_weight: Uint128::zero(),
        next_pay_day: Uint64::from(env.block.time.seconds() + MONTH),
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;
    BASESTATE.save(deps.storage, &base_state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> PayrollResult {
    match msg {
        ExecuteMsg::Base(message) => {
            from_base_dapp_result(dapp_base_commands::handle_base_message(deps, info, message))
        }
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::Pay { asset, os_id } => commands::try_pay(deps, info, asset, None, os_id),
        // ExecuteMsg::UpdatePool {
        //     deposit_asset,
        //     assets_to_add,
        //     assets_to_remove,
        // } => commands::update_pool(deps, info, deposit_asset, assets_to_add, assets_to_remove),
        // ExecuteMsg::SetFee { fee } => commands::set_fee(deps, info, fee),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => dapp_base_queries::handle_base_query(deps, message),
        // handle dapp-specific queries here
        QueryMsg::State {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&StateResponse {
                income: state.income,
                total_weight: state.total_weight,
                next_pay_day: state.next_pay_day,
            })
        }
    }
}

/// Required to convert BaseDAppResult into TerraswapResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: BaseDAppResult) -> PayrollResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}
