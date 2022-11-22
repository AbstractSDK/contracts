use crate::contract::{NoisExtension, NoisResult};
use abstract_os::nois::state::{State, STATE};
use abstract_os::nois::NoisInstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _extension: NoisExtension,
    msg: NoisInstantiateMsg,
) -> NoisResult {
    let state: State = State {
        nois_proxy_addr: deps.api.addr_validate(msg.nois_proxy_addr.as_str())?,
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::new())
}
