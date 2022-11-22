use cosmwasm_std::{to_binary, Binary, Deps, Env, HexBinary, Order, StdResult, Uint128, StdError};

use abstract_os::nois::state::RANDOMNESS_OUTCOME;
use abstract_os::nois::state::{State, STATE};
use abstract_os::nois::{JobHistoryResponse, JobOutcomeResponse, NoisQueryMsg, StateResponse};

use crate::contract::NoisExtension;

/// Handle queries sent to this extension.
pub fn query_handler(
    deps: Deps,
    _env: Env,
    _extension: &NoisExtension,
    msg: NoisQueryMsg,
) -> StdResult<Binary> {
    match msg {
        // handle extension-specific queries here
        NoisQueryMsg::State {} => {
            let state: State = STATE.load(deps.storage)?;
            to_binary(&StateResponse {
                nois_proxy_addr: state.nois_proxy_addr.to_string(),
            })
        }
        NoisQueryMsg::JobHistory {} => to_binary(&query_job_history(deps)?),
        NoisQueryMsg::JobOutcome { job_id } => to_binary(&query_job_outcome(deps, job_id)?),
    }
}

/// Query the outcome for a specific dice roll round/job_id
fn query_job_outcome(deps: Deps, job_id: String) -> StdResult<JobOutcomeResponse> {
    let outcome = RANDOMNESS_OUTCOME.may_load(deps.storage, &job_id)?;
    match outcome {
        Some(outcome) => Ok(JobOutcomeResponse {
            outcome: outcome.to_string()
        }),
        None => Err(StdError::generic_err("Job outcome not found")),
    }
}

/// This function shows all the history of the dice outcomes from all rounds/job_ids
fn query_job_history(deps: Deps) -> StdResult<JobHistoryResponse> {
    let history: Vec<String> = RANDOMNESS_OUTCOME
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(id, value)| format!("{id}:{value}")))
        .collect::<StdResult<_>>()?;

    Ok(JobHistoryResponse {
        jobs: history
    })
}
