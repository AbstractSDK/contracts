use abstract_os::{
    abstract_ica::{DispatchResponse, RegisterResponse, StdAck},
    ibc_host::PacketMsg,
};
use abstract_sdk::ReplyEndpoint;
use cosmwasm_std::{DepsMut, Empty, Env, Reply, Response};
use cw_utils::parse_reply_instantiate_data;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    state::{ACCOUNTS, CLIENT_PROXY, PENDING, PROCESSING_PACKET, RESULTS},
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
    fn handle_reply(
        mut self,
        deps: DepsMut,
        env: Env,
        msg: Reply,
    ) -> Result<Response, Self::ContractError> {
        let id = msg.id;
        let maybe_handler = self.reply_handler(id);
        if let Some(reply_fn) = maybe_handler {
            reply_fn(deps, env, self, msg)
        } else {
            let (packet, channel) = PROCESSING_PACKET.load(deps.storage)?;
            PROCESSING_PACKET.remove(deps.storage);
            let PacketMsg {
                client_chain,
                os_id,
                ..
            } = packet;
            let client_proxy_addr = CLIENT_PROXY.load(deps.storage, (&channel, os_id))?;
            let local_proxy_addr = ACCOUNTS.load(deps.storage, (&channel, os_id))?;
            self.proxy_address = Some(local_proxy_addr);
            // send everything back to client
            let send_back_msg =
                self.send_all_back(deps.as_ref(), env, client_proxy_addr, client_chain)?;

            Ok(Response::new()
                .add_message(send_back_msg)
                .set_data(StdAck::success(&Empty {})))
        }
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
