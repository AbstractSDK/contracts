use abstract_os::ibc_host::{HostAction, InternalAction, PacketMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_slice, to_binary, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, IbcTimeout, StdResult,
};

use abstract_os::simple_ica::{
    check_order, check_version, BalancesResponse, RegisterResponse, StdAck, WhoAmIResponse,
};

use crate::error::ClientError;
use abstract_os::ibc_client::state::{AccountData, ACCOUNTS, CHANNELS, CONFIG, LATEST_QUERIES};
use abstract_os::ibc_client::{CallbackInfo, LatestQueryResponse};

// TODO: make configurable?
/// packets live one hour
pub const PACKET_LIFETIME: u64 = 60 * 60;

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioing constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<Option<Ibc3ChannelOpenResponse>, ClientError> {
    let channel = msg.channel();
    check_order(&channel.order)?;
    check_version(&channel.version)?;
    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }

    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    let channel_id = &channel.endpoint.channel_id;
    // // // create an account holder the channel exists (not found if not registered)
    // let data = AccountData::default();
    // ACCOUNTS.save(deps.storage, channel_id, &data)?;
    let cfg = CONFIG.load(deps.storage)?;

    // construct a packet to send
    let packet = PacketMsg {
        action: HostAction::Internal(InternalAction::WhoAmI),
        client_chain: cfg.chain,
        os_id: 0,
        callback_info: None,
        retries: 0,
    };

    let _msg = IbcMsg::SendPacket {
        channel_id: channel_id.clone(),
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(IbcBasicResponse::new()
        // .add_message(msg)
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", channel_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// On closed channel, simply delete the account from our local store
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();

    // remove the channel
    let channel_id = &channel.endpoint.channel_id;

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_close")
        .add_attribute("channel_id", channel_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// never should be called as the other side never sends packets
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _packet: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    Ok(IbcReceiveResponse::new()
        .set_ack(b"{}")
        .add_attribute("action", "ibc_packet_ack"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ClientError> {
    // which local channel was this packet send from
    let channel_id = msg.original_packet.src.channel_id.clone();
    // we need to parse the ack based on our request
    let mut original_packet: PacketMsg = from_slice(&msg.original_packet.data)?;
    let res: StdAck = from_slice(&msg.acknowledgement.data)?;
    // retry if error
    if let StdAck::Error(_) = res {
        if original_packet.retries > 0 {
            original_packet.retries -= 1;
            // retry sending the packet
            return Ok(IbcBasicResponse::new().add_message(IbcMsg::SendPacket {
                channel_id: msg.original_packet.src.channel_id,
                data: to_binary(&original_packet)?,
                timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(PACKET_LIFETIME)),
            }));
        }
    }

    let PacketMsg {
        os_id,
        callback_info,
        action,
        ..
    } = original_packet;
    match action {
        HostAction::Dispatch { .. } => acknowledge_dispatch(deps, env, callback_info, msg),
        HostAction::Query { .. } => {
            acknowledge_query(deps, env, channel_id, os_id, callback_info, msg)
        }
        HostAction::Balances { .. } => acknowledge_balances(deps, env, channel_id, os_id, res),
        HostAction::App { msg: _ } => {
            let response = IbcBasicResponse::new().add_attribute("action", "acknowledge_app");
            maybe_add_callback(response, callback_info, msg).map_err(Into::into)
        }
        HostAction::SendAllBack { .. } => {
            let response =
                IbcBasicResponse::new().add_attribute("action", "acknowledge_send_all_back");
            maybe_add_callback(response, callback_info, msg).map_err(Into::into)
        }
        HostAction::Internal(InternalAction::WhoAmI) => acknowledge_who_am_i(deps, channel_id, res),
        HostAction::Internal(InternalAction::Register) => {
            acknowledge_register(deps, channel_id, os_id, res)
        }
    }
}

// receive PacketMsg::Dispatch response
#[allow(clippy::unnecessary_wraps)]
fn acknowledge_dispatch(
    _deps: DepsMut,
    _env: Env,
    callback_info: Option<CallbackInfo>,
    ack: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ClientError> {
    let res = IbcBasicResponse::new().add_attribute("action", "acknowledge_dispatch");
    maybe_add_callback(res, callback_info, ack).map_err(Into::into)
}
#[inline(always)]
fn maybe_add_callback(
    response: IbcBasicResponse,
    callback_info: Option<CallbackInfo>,
    ack: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    match callback_info {
        Some(info) => {
            let msg = info.to_callback_msg(&ack.acknowledgement.data)?;
            // Send IBC packet ack message to another contract
            let response = response.add_message(msg);
            Ok(response)
        }
        None => Ok(response),
    }
}

fn acknowledge_query(
    deps: DepsMut,
    env: Env,
    channel_id: String,
    os_id: u32,
    callback_info: Option<CallbackInfo>,
    ack: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ClientError> {
    let msg: StdAck = from_slice(&ack.acknowledgement.data)?;
    let res = IbcBasicResponse::new().add_attribute("action", "acknowledge_ibc_query");
    // store IBC response for later querying from the smart contract??
    LATEST_QUERIES.save(
        deps.storage,
        (&channel_id, os_id),
        &LatestQueryResponse {
            last_update_time: env.block.time,
            response: msg,
        },
    )?;
    match callback_info {
        Some(info) => {
            let msg = info.to_callback_msg(&ack.acknowledgement.data)?;
            // Send IBC packet ack message to another contract
            let res = res.add_message(msg);
            Ok(res)
        }
        None => Ok(res),
    }
}

// receive PacketMsg::WhoAmI response
// store address info in accounts info
fn acknowledge_who_am_i(
    deps: DepsMut,
    channel_id: String,
    ack: StdAck,
) -> Result<IbcBasicResponse, ClientError> {
    // ignore errors (but mention in log)
    let WhoAmIResponse { chain } = match ack {
        StdAck::Result(res) => from_slice(&res)?,
        StdAck::Error(e) => {
            return Ok(IbcBasicResponse::new()
                .add_attribute("action", "acknowledge_who_am_i")
                .add_attribute("error", e))
        }
    };
    // Now we know over what channel to communicate!
    CHANNELS.save(deps.storage, &chain, &channel_id)?;

    Ok(IbcBasicResponse::new().add_attribute("action", "acknowledge_who_am_i"))
}

// receive PacketMsg::Register response
// store address info in accounts info
fn acknowledge_register(
    deps: DepsMut,
    channel_id: String,
    os_id: u32,
    ack: StdAck,
) -> Result<IbcBasicResponse, ClientError> {
    // ignore errors (but mention in log)
    let RegisterResponse { account } = match ack {
        StdAck::Result(res) => from_slice(&res)?,
        StdAck::Error(e) => {
            return Ok(IbcBasicResponse::new()
                .add_attribute("action", "acknowledge_who_am_i")
                .add_attribute("error", e))
        }
    };

    ACCOUNTS.update(deps.storage, (&channel_id, os_id), |acct| {
        match acct {
            Some(mut acct) => {
                // set the account the first time
                if acct.remote_addr.is_none() {
                    acct.remote_addr = Some(account);
                }
                Ok(acct)
            }
            None => Err(ClientError::UnregisteredChannel(channel_id.clone())),
        }
    })?;

    Ok(IbcBasicResponse::new().add_attribute("action", "acknowledge_who_am_i"))
}

// receive PacketMsg::Balances response
fn acknowledge_balances(
    deps: DepsMut,
    env: Env,
    channel_id: String,
    os_id: u32,
    ack: StdAck,
) -> Result<IbcBasicResponse, ClientError> {
    // ignore errors (but mention in log)
    let BalancesResponse { account, balances } = match ack {
        StdAck::Result(res) => from_slice(&res)?,
        StdAck::Error(e) => {
            return Ok(IbcBasicResponse::new()
                .add_attribute("action", "acknowledge_balances")
                .add_attribute("error", e))
        }
    };

    ACCOUNTS.update(deps.storage, (&channel_id, os_id), |acct| match acct {
        Some(acct) => {
            if let Some(old) = acct.remote_addr {
                if old != account {
                    return Err(ClientError::RemoteAccountChanged { old, addr: account });
                }
            }
            Ok(AccountData {
                last_update_time: env.block.time,
                remote_addr: Some(account),
                remote_balance: balances,
            })
        }
        None => Err(ClientError::UnregisteredChannel(channel_id.clone())),
    })?;

    Ok(IbcBasicResponse::new().add_attribute("action", "acknowledge_balances"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// we just ignore these now. shall we store some info?
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::contract::{execute, instantiate, query};
//     use abstract_os::ibc_client::{AccountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

//     use abstract_os::simple_ica::{APP_ORDER, BAD_APP_ORDER, IBC_APP_VERSION};
//     use cosmwasm_std::testing::{
//         mock_dependencies, mock_env, mock_ibc_channel_connect_ack, mock_ibc_channel_open_init,
//         mock_ibc_channel_open_try, mock_ibc_packet_ack, mock_info, MockApi, MockQuerier,
//         MockStorage,
//     };
//     use cosmwasm_std::{coin, coins, BankMsg, CosmosMsg, IbcAcknowledgement, OwnedDeps};

//     const CREATOR: &str = "creator";

//     fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
//         let mut deps = mock_dependencies();
//         let msg = InstantiateMsg {};
//         let info = mock_info(CREATOR, &[]);
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());
//         deps
//     }

//     // connect will run through the entire handshake to set up a proper connect and
//     // save the account (tested in detail in `proper_handshake_flow`)
//     fn connect(mut deps: DepsMut, channel_id: &str) {
//         let handshake_open = mock_ibc_channel_open_init(channel_id, APP_ORDER, IBC_APP_VERSION);
//         // first we try to open with a valid handshake
//         ibc_channel_open(deps.branch(), mock_env(), handshake_open).unwrap();

//         // then we connect (with counter-party version set)
//         let handshake_connect =
//             mock_ibc_channel_connect_ack(channel_id, APP_ORDER, IBC_APP_VERSION);
//         let res = ibc_channel_connect(deps.branch(), mock_env(), handshake_connect).unwrap();

//         // this should send a WhoAmI request, which is received some blocks later
//         assert_eq!(1, res.messages.len());
//         match &res.messages[0].msg {
//             CosmosMsg::Ibc(IbcMsg::SendPacket {
//                 channel_id: packet_channel,
//                 ..
//             }) => assert_eq!(packet_channel.as_str(), channel_id),
//             o => panic!("Unexpected message: {:?}", o),
//         };
//     }

//     fn who_am_i_response(deps: DepsMut, channel_id: &str, account: impl Into<String>) {
//         let packet = PacketMsg::WhoAmI {};
//         let res = StdAck::success(WhoAmIResponse {
//             account: account.into(),
//         });
//         let ack = IbcAcknowledgement::new(res);
//         let msg = mock_ibc_packet_ack(channel_id, &packet, ack).unwrap();
//         let res = ibc_packet_ack(deps, mock_env(), msg).unwrap();
//         assert_eq!(0, res.messages.len());
//     }

//     #[test]
//     fn enforce_version_in_handshake() {
//         let mut deps = setup();

//         let wrong_order = mock_ibc_channel_open_try("channel-12", BAD_APP_ORDER, IBC_APP_VERSION);
//         ibc_channel_open(deps.as_mut(), mock_env(), wrong_order).unwrap_err();

//         let wrong_version = mock_ibc_channel_open_try("channel-12", APP_ORDER, "reflect");
//         ibc_channel_open(deps.as_mut(), mock_env(), wrong_version).unwrap_err();

//         let valid_handshake = mock_ibc_channel_open_try("channel-12", APP_ORDER, IBC_APP_VERSION);
//         ibc_channel_open(deps.as_mut(), mock_env(), valid_handshake).unwrap();
//     }

//     #[test]
//     fn proper_handshake_flow() {
//         // setup and connect handshake
//         let mut deps = setup();
//         let channel_id = "channel-1234";
//         connect(deps.as_mut(), channel_id);

//         // check for empty account
//         let q = QueryMsg::Account {
//             channel_id: channel_id.into(),
//         };
//         let r = query(deps.as_ref(), mock_env(), q).unwrap();
//         let acct: AccountResponse = from_slice(&r).unwrap();
//         assert!(acct.remote_addr.is_none());
//         assert!(acct.remote_balance.is_empty());
//         assert_eq!(0, acct.last_update_time.nanos());

//         // now get feedback from WhoAmI packet
//         let remote_addr = "account-789";
//         who_am_i_response(deps.as_mut(), channel_id, remote_addr);

//         // account should be set up
//         let q = QueryMsg::Account {
//             channel_id: channel_id.into(),
//         };
//         let r = query(deps.as_ref(), mock_env(), q).unwrap();
//         let acct: AccountResponse = from_slice(&r).unwrap();
//         assert_eq!(acct.remote_addr.unwrap(), remote_addr);
//         assert!(acct.remote_balance.is_empty());
//         assert_eq!(0, acct.last_update_time.nanos());
//     }

//     #[test]
//     fn dispatch_message_send_and_ack() {
//         let channel_id = "channel-1234";
//         let remote_addr = "account-789";

//         // init contract
//         let mut deps = setup();
//         // channel handshake
//         connect(deps.as_mut(), channel_id);
//         // get feedback from WhoAmI packet
//         who_am_i_response(deps.as_mut(), channel_id, remote_addr);

//         // try to dispatch a message
//         let msgs_to_dispatch = vec![BankMsg::Send {
//             to_address: "my-friend".into(),
//             amount: coins(123456789, "uatom"),
//         }
//         .into()];
//         let handle_msg = ExecuteMsg::SendMsgs {
//             channel_id: channel_id.into(),
//             msgs: msgs_to_dispatch,
//             callback_id: None,
//         };
//         let info = mock_info(CREATOR, &[]);
//         let mut res = execute(deps.as_mut(), mock_env(), info, handle_msg).unwrap();
//         assert_eq!(1, res.messages.len());
//         let msg = match res.messages.swap_remove(0).msg {
//             CosmosMsg::Ibc(IbcMsg::SendPacket {
//                 channel_id, data, ..
//             }) => {
//                 let ack = IbcAcknowledgement::new(StdAck::success(&()));
//                 let mut msg = mock_ibc_packet_ack(&channel_id, &1u32, ack).unwrap();
//                 msg.original_packet.data = data;
//                 msg
//             }
//             o => panic!("Unexpected message: {:?}", o),
//         };
//         let res = ibc_packet_ack(deps.as_mut(), mock_env(), msg).unwrap();
//         // no actions expected, but let's check the events to see it was dispatched properly
//         assert_eq!(0, res.messages.len());
//         assert_eq!(vec![("action", "acknowledge_dispatch")], res.attributes)
//     }

//     #[test]
//     fn send_remote_funds() {
//         let reflect_channel_id = "channel-1234";
//         let remote_addr = "account-789";
//         let transfer_channel_id = "transfer-2";

//         // init contract
//         let mut deps = setup();
//         // channel handshake
//         connect(deps.as_mut(), reflect_channel_id);
//         // get feedback from WhoAmI packet
//         who_am_i_response(deps.as_mut(), reflect_channel_id, remote_addr);

//         // let's try to send funds to a channel that doesn't exist
//         let msg = ExecuteMsg::SendFunds {
//             ica_channel_id: "random-channel".into(),
//             transfer_channel_id: transfer_channel_id.into(),
//         };
//         let info = mock_info(CREATOR, &coins(12344, "utrgd"));
//         execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

//         // let's try with no sent funds in the message
//         let msg = ExecuteMsg::SendFunds {
//             ica_channel_id: reflect_channel_id.into(),
//             transfer_channel_id: transfer_channel_id.into(),
//         };
//         let info = mock_info(CREATOR, &[]);
//         execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

//         // 3rd times the charm
//         let msg = ExecuteMsg::SendFunds {
//             ica_channel_id: reflect_channel_id.into(),
//             transfer_channel_id: transfer_channel_id.into(),
//         };
//         let info = mock_info(CREATOR, &coins(12344, "utrgd"));
//         let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(1, res.messages.len());
//         match &res.messages[0].msg {
//             CosmosMsg::Ibc(IbcMsg::Transfer {
//                 channel_id,
//                 to_address,
//                 amount,
//                 timeout,
//             }) => {
//                 assert_eq!(transfer_channel_id, channel_id.as_str());
//                 assert_eq!(remote_addr, to_address.as_str());
//                 assert_eq!(&coin(12344, "utrgd"), amount);
//                 assert!(timeout.block().is_none());
//                 assert!(timeout.timestamp().is_some());
//             }
//             o => panic!("unexpected message: {:?}", o),
//         }
//     }
// }
