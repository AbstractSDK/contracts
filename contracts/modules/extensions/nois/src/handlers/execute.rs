//! # Staking
use crate::contract::{NoisExtension, NoisResult};
use crate::error::NoisError;
use abstract_os::nois::state::{RANDOMNESS_OUTCOME, STATE};
use abstract_os::nois::NoisRequestMsg;
use abstract_sdk::Execution;
use cosmos_nois::ProxyExecuteMsg;
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response};

type NoisProxyExecuteMsg = ProxyExecuteMsg;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    extension: NoisExtension,
    msg: NoisRequestMsg,
) -> NoisResult {
    // Use executor to execute arbirtrary CosmosMsgs
    let executor = extension.executor(deps.as_ref());
    let msg = match msg {
        NoisRequestMsg::Randomness { job_id } => {
            executor.execute(vec![get_next_randomness(deps.as_ref(), info, job_id)?])
        }
    }?;
    Ok(Response::new().add_message(msg))
}

/// Function that triggers the process of requesting randomness
/// The request from randomness happens by calling the nois-proxy contract
pub fn get_next_randomness(
    deps: Deps,
    info: MessageInfo,
    job_id: String,
) -> Result<CosmosMsg, NoisError> {
    let nois_proxy = STATE.load(deps.storage)?.nois_proxy_addr;
    // Prevent a user from paying for an already existing randomness.
    // The actual immutability of the history comes in the execute_receive function
    if RANDOMNESS_OUTCOME
        .may_load(deps.storage, &job_id)?
        .is_some()
    {
        return Err(NoisError::JobIdAlreadyPresent);
    }

    Ok(wasm_execute(
        nois_proxy,
        // GetNextRandomness requests the randomness from the proxy
        // The job id is needed to know what randomness we are referring to upon reception in the callback
        // In this example, the job_id represents one round of dice rolling.
        &NoisProxyExecuteMsg::GetNextRandomness { job_id },
        // For now the randomness is for free. You don't need to send any funds to request randomness
        info.funds,
    )?
    .into())
}
