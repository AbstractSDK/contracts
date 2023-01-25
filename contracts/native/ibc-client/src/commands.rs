use crate::contract::{IbcClientResponse, IbcClientResult, MAX_RETRIES};
use crate::error::ClientError;
use crate::ibc::PACKET_LIFETIME;
use abstract_os::ibc_client::state::{
    AccountData, ACCOUNTS, ADMIN, ANS_HOST, CHANNELS, CONFIG, LATEST_QUERIES,
};
use abstract_os::ibc_client::CallbackInfo;
use abstract_os::ibc_host::{HostAction, InternalAction, PacketMsg};
use abstract_os::objects::ans_host::AnsHost;
use abstract_os::objects::ChannelEntry;
use abstract_os::ICS20;
use abstract_sdk::base::features::Identification;
use abstract_sdk::feature_objects::VersionControlContract;
use abstract_sdk::{Execution, Resolve, Verification};
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, DepsMut, Env, IbcMsg, MessageInfo, Response, StdError, StdResult,
    Storage,
};

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_ans_host: Option<String>,
    new_version_control: Option<String>,
) -> IbcClientResult {
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut cfg = CONFIG.load(deps.storage)?;

    if let Some(ans_host) = new_ans_host {
        ANS_HOST.save(
            deps.storage,
            &AnsHost {
                address: deps.api.addr_validate(&ans_host)?,
            },
        )?;
    }
    if let Some(version_control) = new_version_control {
        cfg.version_control_address = deps.api.addr_validate(&version_control)?;
        // New version control address implies new accounts.
        clear_accounts(deps.storage);
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(IbcClientResponse::action("update_config"))
}

// allows admins to clear host if needed
pub fn execute_remove_host(
    deps: DepsMut,
    info: MessageInfo,
    host_chain: String,
) -> IbcClientResult {
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    CHANNELS.remove(deps.storage, &host_chain);

    Ok(IbcClientResponse::action("remove_host"))
}

pub fn execute_send_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
    action: HostAction,
    callback_info: Option<CallbackInfo>,
    mut retries: u8,
) -> IbcClientResult {
    // auth check
    let cfg = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract {
        address: cfg.version_control_address,
    };
    // Verify that the sender is a proxy contract
    let core = version_control
        .os_register(deps.as_ref())
        .assert_proxy(&info.sender)?;
    // Can only call non-internal actions
    if let HostAction::Internal(_) = action {
        return Err(ClientError::ForbiddenInternalCall {});
    }
    // Set max retries
    retries = retries.min(MAX_RETRIES);

    // get os_id
    let os_id = core.os_id(deps.as_ref())?;
    // ensure the channel exists and loads it.
    let channel = CHANNELS.load(deps.storage, &host_chain)?;
    let packet = PacketMsg {
        retries,
        client_chain: cfg.chain,
        os_id,
        callback_info,
        action,
    };
    let msg = IbcMsg::SendPacket {
        channel_id: channel,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(IbcClientResponse::action("handle_send_msgs").add_message(msg))
}

pub fn execute_register_os(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
) -> IbcClientResult {
    // auth check
    let cfg = CONFIG.load(deps.storage)?;
    // Verify that the sender is a proxy contract
    let version_control = VersionControlContract {
        address: cfg.version_control_address,
    };
    let core = version_control
        .os_register(deps.as_ref())
        .assert_proxy(&info.sender)?;
    // ensure the channel exists (not found if not registered)
    let channel_id = CHANNELS.load(deps.storage, &host_chain)?;
    let os_id = core.os_id(deps.as_ref())?;

    // construct a packet to send
    let packet = PacketMsg {
        retries: 0u8,
        client_chain: cfg.chain,
        os_id,
        callback_info: None,
        action: HostAction::Internal(InternalAction::Register {
            os_proxy_address: core.proxy.into_string(),
        }),
    };

    // save a default value to account
    let account = AccountData::default();
    ACCOUNTS.save(deps.storage, (&channel_id, os_id), &account)?;

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(IbcClientResponse::action("handle_register").add_message(msg))
}

pub fn execute_send_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
    funds: Vec<Coin>,
) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    let mem = ANS_HOST.load(deps.storage)?;
    // Verify that the sender is a proxy contract
    let version_control = VersionControlContract {
        address: cfg.version_control_address,
    };
    let core = version_control
        .os_register(deps.as_ref())
        .assert_proxy(&info.sender)?;
    // get os_id of OS
    let os_id = core.os_id(deps.as_ref())?;
    // get channel used to communicate to host chain
    let channel = CHANNELS.load(deps.storage, &host_chain)?;
    // load remote account
    let data = ACCOUNTS.load(deps.storage, (&channel, os_id))?;
    let remote_addr = match data.remote_addr {
        Some(addr) => addr,
        None => {
            return Err(StdError::generic_err(
                "We don't have the remote address for this channel or OS",
            ))
        }
    };

    let ics20_channel_entry = ChannelEntry {
        connected_chain: host_chain,
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &mem)?;

    let mut transfers: Vec<CosmosMsg> = vec![];
    for amount in funds {
        // construct a packet to send
        transfers.push(
            IbcMsg::Transfer {
                channel_id: ics20_channel_id.clone(),
                to_address: remote_addr.clone(),
                amount,
                timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
            }
            .into(),
        );
    }
    // let these messages be executed by proxy
    let proxy_msg = core.executor(deps.as_ref()).execute(transfers)?;

    Ok(IbcClientResponse::action("handle_send_funds").add_message(proxy_msg))
}

fn clear_accounts(store: &mut dyn Storage) {
    ACCOUNTS.clear(store);
    LATEST_QUERIES.clear(store);
}
