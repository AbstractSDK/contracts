use abstract_os::abstract_ica::{DispatchResponse, RegisterResponse, StdAck};
use abstract_sdk::{IbcCallbackEndpoint, IbcCallbackHandlerFn, ReplyEndpoint};
use cosmwasm_std::{entry_point, DepsMut, Env, Reply, Response};
use cw_utils::parse_reply_instantiate_data;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    state::{ACCOUNTS, PENDING, RESULTS},
    Host, HostError,
};

pub const RECEIVE_DISPATCH_ID: u64 = 1234;
pub const INIT_CALLBACK_ID: u64 = 7890;

impl<T: Serialize + DeserializeOwned> ReplyEndpoint for Host<'_, T> {
    type ContractError = HostError;

    fn reply_handler(
        &self,
        id: u64,
    ) -> Option<abstract_sdk::ReplyHandlerFn<Self, Self::ContractError>> {
        for reply_handler in self.reply_handlers {
            if reply_handler.0 == id {
                return Some(reply_handler.1);
            }
        }
        None
    }
}

pub fn reply_dispatch_callback<T: Serialize + DeserializeOwned>(
    deps: DepsMut,
    _env: Env,
    _host: Host<'_, T>,
    reply: Reply,
) -> Result<Response, HostError> {
    // add the new result to the current tracker
    let mut results = RESULTS.load(deps.storage)?;
    results.push(reply.result.unwrap().data.unwrap_or_default());
    RESULTS.save(deps.storage, &results)?;

    // update result data if this is the last
    let data = StdAck::success(&DispatchResponse { results });
    Ok(Response::new().set_data(data))
}

pub fn reply_init_callback<T: Serialize + DeserializeOwned>(
    deps: DepsMut,
    _env: Env,
    _host: Host<'_, T>,
    reply: Reply,
) -> Result<Response, HostError> {
    // we use storage to pass info from the caller to the reply
    let (channel, os_id) = PENDING.load(deps.storage)?;
    PENDING.remove(deps.storage);

    // parse contract info from data
    let raw_addr = parse_reply_instantiate_data(reply)?.contract_address;
    let contract_addr = deps.api.addr_validate(&raw_addr)?;

    if ACCOUNTS
        .may_load(deps.storage, (&channel, os_id))?
        .is_some()
    {
        return Err(HostError::ChannelAlreadyRegistered);
    }
    ACCOUNTS.save(deps.storage, (&channel, os_id), &contract_addr)?;
    let data = StdAck::success(&RegisterResponse {
        account: contract_addr.into_string(),
    });
    Ok(Response::new().set_data(data))
}
