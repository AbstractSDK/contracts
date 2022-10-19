use abstract_os::{ibc_client::{ExecuteMsg as IbcClientMsg, CallbackInfo}, ibc_host::HostAction};
use cosmwasm_std::{Addr, StdError, CosmosMsg, Coin};

use crate::{proxy::os_ibc_action, OsAction};

/// Call a [`HostAction`] on the host of the provided `host_chain`.
pub fn host_ibc_action(proxy_address: &Addr, host_chain: String, action: HostAction, callback: Option<CallbackInfo>) -> Result<OsAction, StdError> {
    os_ibc_action(vec![IbcClientMsg::SendPacket { host_chain, action, callback_info: callback }], proxy_address)
}
/// Transfer the provided coins from the OS to it's proxy on the `receiving_chain`.
pub fn ics20_transfer(proxy_address: &Addr, receiving_chain: String, funds: Vec<Coin>) -> Result<OsAction, StdError> {
    os_ibc_action(vec![IbcClientMsg::SendFunds { host_chain: receiving_chain, funds }], proxy_address)
}