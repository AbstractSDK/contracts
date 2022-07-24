use abstract_api::state::ApiInterfaceResponse;
use abstract_api::{ApiContract, ApiResult};
use abstract_os::api::{ApiInstantiateMsg, ApiInterfaceMsg};
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use abstract_os::tendermint_staking::{ExecuteMsg, QueryMsg};

use crate::commands;
use crate::error::TendermintStakeError;

pub type TendermintStakeApi<'a> = ApiContract<'a, ExecuteMsg>;
pub type TendermintStakeResult = Result<Response, TendermintStakeError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInstantiateMsg,
) -> ApiResult {
    TendermintStakeApi::default().instantiate(deps, env, info, msg, "tendermint_staking", "3.2.8")?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInterfaceMsg<ExecuteMsg>,
) -> TendermintStakeResult {
    let mut api = TendermintStakeApi::default();
    let resp = api.handle_request(&mut deps, env,&info, msg)?;
    match resp {
        ApiInterfaceResponse::ExecResponse(resp) => Ok(resp),
        ApiInterfaceResponse::ProcessRequest(msg) => {
        match msg {
    ExecuteMsg::Delegate { validator, amount } => delegate(deps, env, info, validator, amount),
    ExecuteMsg::Undelegate { validator, amount } => todo!(),
    ExecuteMsg::Redelegate { source_validator, destination_validator, amount } => todo!(),
    ExecuteMsg::SetWithdrawAddress { new_withdraw_address } => todo!(),
    ExecuteMsg::WithdrawDelegatorReward { validator } => todo!(),
}
    }
}

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(dapp_msg) => TendermintStakeApi::default().query(deps, env, dapp_msg),
    }
}

/// Required to convert BaseDAppResult into TendermintStakeResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: ApiResult) -> TendermintStakeResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}
