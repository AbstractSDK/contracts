use abstract_os::{
    host::{ExecuteMsg, PacketMsg},
    version_control::Core,
};
use abstract_sdk::{
    proxy::query_os_manager_address, query_module_address, verify_os_manager, verify_os_proxy,
    OsExecute,
};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, WasmMsg, IbcPacketReceiveMsg, from_slice, IbcReceiveResponse,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{error::HostError, state::HostContract, ApiResult, host_commands::{receive_who_am_i, receive_balances, receive_query, receive_dispatch}};

/// The host contract base implementation.
impl<'a, T: Serialize + DeserializeOwned> HostContract<'a, T> {
    /// Takes ibc request, matches and executes 
    /// This fn is the only way to get an HostContract instance.
    pub fn handle_packet<RequestError: From<cosmwasm_std::StdError> + From<HostError>>(
        mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        packet: IbcPacketReceiveMsg,
        app_handler: impl FnOnce(
            DepsMut,
            Env,
            MessageInfo,
            HostContract<T>,
            T,
        ) -> Result<IbcReceiveResponse, RequestError>,
    ) -> Result<IbcReceiveResponse, RequestError> {
        let packet = packet.packet;
        // which local channel did this packet come on
        let caller = packet.dest.channel_id;
        let msg: PacketMsg<T> = from_slice(&packet.data)?;
        match msg {
            PacketMsg::Dispatch { msgs, .. } => receive_dispatch(deps, caller, msgs),
            PacketMsg::Query { msgs, .. } => receive_query(deps.as_ref(), msgs),
            PacketMsg::WhoAmI {} => receive_who_am_i(deps, caller),
            PacketMsg::Balances {} => receive_balances(deps, caller),
            PacketMsg::SendAllBack { sender } => todo!(),
            PacketMsg::App(msg) => return app_handler(deps,env,info,self,msg)
        }.map_err(Into::into)
    }
    // pub fn execute(
    //     &mut self,
    //     deps: DepsMut,
    //     env: Env,
    //     info: MessageInfo,
    //     message: BaseExecuteMsg,
    // ) -> ApiResult {
    //     match message {
    //         BaseExecuteMsg::UpdateTraders { to_add, to_remove } => {
    //             self.update_traders(deps, info, to_add, to_remove)
    //         }
    //         BaseExecuteMsg::Remove {} => self.remove_self_from_deps(deps.as_ref(), env, info),
    //     }
    // }
}
