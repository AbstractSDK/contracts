use cosmwasm_std::{Binary, Deps, Env, StdResult, to_binary};

use abstract_os::etf::{EtfQueryMsg, StateResponse};
use abstract_os::etf::state::{FEE, STATE};

use crate::contract::EtfApp;

pub fn query_handler(deps: Deps, _env: Env, _etf: &EtfApp, msg: EtfQueryMsg) -> StdResult<Binary> {
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
