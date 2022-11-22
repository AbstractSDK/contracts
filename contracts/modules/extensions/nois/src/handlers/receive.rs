use crate::contract::{NoisExtension, NoisResult};
use crate::error::NoisError;
use crate::NoisReceiveMsg;
use abstract_os::nois::state::{RANDOMNESS_OUTCOME, STATE};
use cosmwasm_std::{ensure_eq, DepsMut, Env, HexBinary, MessageInfo, Response};

/// The execute_receive function is triggered upon reception of the randomness from the proxy contract
/// The callback contains the randomness from drand (HexBinary) and the job_id
pub fn nois_callback_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    extension: NoisExtension,
    nois_receive: NoisReceiveMsg,
) -> NoisResult {
    //load proxy address from store
    // TODO: retrieve the nois_proxy from memory
    let nois_proxy = STATE.load(deps.storage)?.nois_proxy_addr;

    //callback should only be allowed to be called by the proxy contract
    //otherwise anyone can cut the randomness workflow and cheat the randomness by sending the randomness directly to this contract
    ensure_eq!(info.sender, nois_proxy, NoisError::UnauthorizedReceive);

    let callback = nois_receive.callback;
    let randomness: HexBinary = callback.randomness;

    // check that the randomness can be converted to an array of 32 u8
    randomness
        .to_array::<32>()
        .map_err(|_| NoisError::InvalidRandomness)?;

    // Preserve the immutability of the previous rounds.
    // So that the user cannot retry and change history.
    let response = match RANDOMNESS_OUTCOME.may_load(deps.storage, &callback.job_id)? {
        None => Response::default(),
        Some(_randomness) => return Err(NoisError::JobIdAlreadyPresent),
    };
    RANDOMNESS_OUTCOME.save(deps.storage, &callback.job_id, &randomness)?;

    Ok(response)
}
