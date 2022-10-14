use abstract_os::ibc_host::{ExecuteMsg, PacketMsg};

use cosmwasm_std::{
    from_slice, Addr, DepsMut, Env, IbcPacketReceiveMsg, IbcReceiveResponse, MessageInfo, Response,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::HostError,
    host_commands::{receive_balances, receive_dispatch, receive_query, receive_register},
    state::{Host, ACCOUNTS, CLOSED_CHANNELS},
};

/// The host contract base implementation.
impl<'a, T: Serialize + DeserializeOwned> Host<'a, T> {
    /// Takes ibc request, matches and executes
    /// This fn is the only way to get an Host instance.
    pub fn handle_packet<RequestError: From<cosmwasm_std::StdError> + From<HostError>>(
        self,
        deps: DepsMut,
        env: Env,
        packet: IbcPacketReceiveMsg,
        packet_handler: impl FnOnce(
            DepsMut,
            Env,
            String,
            Host<T>,
            T,
        ) -> Result<IbcReceiveResponse, RequestError>,
    ) -> Result<IbcReceiveResponse, RequestError> {
        let packet = packet.packet;
        // which local channel did this packet come on
        let channel = packet.dest.channel_id;
        let msg: PacketMsg<T> = from_slice(&packet.data)?;
        match msg {
            PacketMsg::Register { os_id } => receive_register(deps, env,channel, os_id),
            PacketMsg::Dispatch { os_id, msgs, .. } => receive_dispatch(deps, channel, os_id,msgs),
            PacketMsg::Query { msgs, .. } => receive_query(deps.as_ref(), msgs),
            PacketMsg::Balances { os_id } => receive_balances(deps, channel,os_id),
            PacketMsg::SendAllBack { os_id } => todo!(),
            PacketMsg::App(msg) => return packet_handler(deps, env, channel, self, msg),
        }
        .map_err(Into::into)
    }
    pub fn execute(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message: ExecuteMsg,
    ) -> Result<Response, HostError> {
        match message {
            ExecuteMsg::ClearAccount {
                closed_channel,
                os_id,
            } => {
                let closed_channels = CLOSED_CHANNELS.load(deps.storage)?;
                if !closed_channels.contains(&closed_channel) {
                    return Err(HostError::ChannelNotClosed {});
                }
                // call send_all_back here
                todo!();
                // clean up state
                ACCOUNTS.remove(deps.storage, (&closed_channel, os_id));
            }
        }
    }
}
