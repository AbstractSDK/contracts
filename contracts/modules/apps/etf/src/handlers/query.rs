use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

use abstract_os::etf::state::{FEE, STATE};
use abstract_os::etf::{EtfQueryMsg, StateResponse};

use crate::contract::EtfApp;

pub fn query_handler(deps: Deps, _env: Env, _etf: &EtfApp, msg: EtfQueryMsg) -> StdResult<Binary> {
    match msg {
        EtfQueryMsg::State {} => to_binary(&query_state(&deps)?)
    }
}

fn query_state(deps: &Deps) -> StdResult<StateResponse> {
    let fee = FEE.load(deps.storage)?;
    Ok(StateResponse {
        liquidity_token: STATE.load(deps.storage)?.liquidity_token_addr.to_string(),
        fee: fee.share(),
    })
}
