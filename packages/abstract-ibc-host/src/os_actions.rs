use abstract_os::abstract_ica::{BalancesResponse, DispatchResponse, SendAllBackResponse, StdAck};
use cosmwasm_std::{
    wasm_execute, CosmosMsg, DepsMut, Empty, Env, IbcMsg, IbcReceiveResponse, SubMsg,
};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::host_commands::PACKET_LIFETIME;
use crate::reply::RECEIVE_DISPATCH_ID;
use crate::state::RESULTS;
use crate::{Host, HostError};

impl<'a, T: Serialize + DeserializeOwned> Host<'a, T> {
    // processes PacketMsg::Balances variant
    pub fn receive_balances(&self, deps: DepsMut) -> Result<IbcReceiveResponse, HostError> {
        let account = self.proxy_address.as_ref().unwrap();
        let balances = deps.querier.query_all_balances(account)?;
        let response = BalancesResponse {
            account: account.into(),
            balances,
        };
        let acknowledgement = StdAck::success(&response);
        // and we are golden
        Ok(IbcReceiveResponse::new()
            .set_ack(acknowledgement)
            .add_attribute("action", "receive_balances"))
    }

    // processes PacketMsg::Dispatch variant
    pub fn receive_dispatch(
        &self,
        deps: DepsMut,
        msgs: Vec<CosmosMsg>,
    ) -> Result<IbcReceiveResponse, HostError> {
        let reflect_addr = self.proxy_address.as_ref().unwrap();

        // let them know we're fine
        let response = DispatchResponse { results: vec![] };
        let acknowledgement = StdAck::success(&response);
        // create the message to re-dispatch to the reflect contract
        let reflect_msg = cw1_whitelist::msg::ExecuteMsg::Execute { msgs };
        let wasm_msg = wasm_execute(reflect_addr, &reflect_msg, vec![])?;

        // we wrap it in a submessage to properly report results
        let msg = SubMsg::reply_on_success(wasm_msg, RECEIVE_DISPATCH_ID);

        // reset the data field
        RESULTS.save(deps.storage, &vec![])?;

        Ok(IbcReceiveResponse::new()
            .set_ack(acknowledgement)
            .add_submessage(msg)
            .add_attribute("action", "receive_dispatch"))
    }

    // processes PacketMsg::SendAllBack variant
    pub fn receive_send_all_back(
        &self,
        deps: DepsMut,
        env: Env,
        os_proxy_address: String,
        transfer_channel: String,
    ) -> Result<IbcReceiveResponse, HostError> {
        let reflect_addr = self.proxy_address.as_ref().unwrap();
        // let them know we're fine
        let response = SendAllBackResponse {};
        let acknowledgement = StdAck::success(&response);

        let coins = deps.querier.query_all_balances(reflect_addr)?;
        let mut msgs: Vec<CosmosMsg> = vec![];
        for coin in coins {
            msgs.push(
                IbcMsg::Transfer {
                    channel_id: transfer_channel.clone(),
                    to_address: os_proxy_address.to_string(),
                    amount: coin,
                    timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
                }
                .into(),
            )
        }
        // create the message to re-dispatch to the reflect contract
        let reflect_msg = cw1_whitelist::msg::ExecuteMsg::Execute { msgs };
        let wasm_msg: CosmosMsg<Empty> = wasm_execute(reflect_addr, &reflect_msg, vec![])?.into();

        // reset the data field
        RESULTS.save(deps.storage, &vec![])?;

        Ok(IbcReceiveResponse::new()
            .set_ack(acknowledgement)
            .add_message(wasm_msg)
            .add_attribute("action", "receive_dispatch"))
    }
}
