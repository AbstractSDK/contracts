use base64::{engine::general_purpose, Engine as _};

use cw_orch::queriers::DaemonQuerier;
use cw_orch::{Daemon, DaemonError};
use cw_orch::queriers::Node;
use cosmwasm_std::StdError;
use anyhow::{Result, bail};

// This was coded thanks to this wonderful guide : https://github.com/CosmWasm/cosmwasm/blob/main/IBC.md

// type is from cosmos_sdk_proto::ibc::core::channel::v1::acknowledgement::Response
#[cosmwasm_schema::cw_serde]
pub enum AckResponse{
     Result(String), // This is a base64 string
     Error(String)
}


#[async_recursion::async_recursion(?Send)]
pub async fn follow_trail(chain1: &Daemon, chain2: &Daemon, tx_hash: String) -> Result<()>{
	// In this function, we need to : 
	// 1. Get all ibc outgoing messages from the transaction
		// attribute type : send_packet
		// Things to track
		// connection
		// dest-port
		// dest_channel
		// packet_sequence
		// timeout_timestamp (for stopping the search) - Not needed here
	let tx = Node::new(chain1.channel()).find_tx_by_hash(tx_hash.clone()).await.unwrap();
	let send_packet_events = tx.get_events("send_packet");
	if send_packet_events.is_empty(){
			return Ok(()) //Box::pin(async {  });
	}
	log::info!("Investigating sent packet events on tx {}", tx_hash.clone());

	// We account for a single send_packet event at a time for now (TODO)
	let connection = send_packet_events[0].get_first_attribute_value("packet_connection").unwrap();
	let dest_port = send_packet_events[0].get_first_attribute_value("packet_dst_port").unwrap();
	let dest_channel = send_packet_events[0].get_first_attribute_value("packet_dst_channel").unwrap();
	let sequence = send_packet_events[0].get_first_attribute_value("packet_sequence").unwrap();
	let packet_data = send_packet_events[0].get_first_attribute_value("packet_data").unwrap();

	// 2. For each message find the transaction hash of the txs the message during which the message is broadcasted to the distant chain
	// This only works for 2 chains for now, we don't handle more chains
	let events_string = vec![format!(
		"recv_packet.packet_connection='{}'", connection
	), format!(
		"recv_packet.packet_dst_port='{}'", dest_port
	), format!(
		"recv_packet.packet_dst_channel='{}'", dest_channel
	), format!(
		"recv_packet.packet_sequence='{}'", sequence
	)];

	log::info!("IBC packet n° {}, sent on {} on tx {}, with data: {}", 
		sequence,
		chain1.state.chain_id, 
		tx_hash, 
		packet_data
	);

	let txs = Node::new(chain2.channel()).find_tx_by_events(events_string, None, None).await.unwrap();

	// We need to make sure there is only 1 transaction with such events (always should be the case)
	if txs.len() != 1 {
		bail!(StdError::generic_err("Found multiple transactions matching the events, not possible"));
	}
	let received_tx = &txs[0];
	// We check if the tx errors (this shouldn't happen in IBC connections)
	if received_tx.code != 0 {
		bail!(DaemonError::TxFailed {
            code: received_tx.code,
            reason: format!("Raw log on {} : {}", chain2.state.chain_id, received_tx.raw_log.clone()),
        });
    }

    // 3. Then we look for the acknowledgment packet that should always be traced back during this transaction
    let recv_packet_sequence = received_tx.get_events("write_acknowledgement")[0].get_first_attribute_value("packet_sequence").unwrap();
    let recv_packet_data = received_tx.get_events("write_acknowledgement")[0].get_first_attribute_value("packet_data").unwrap();
	let acknowledgment = received_tx.get_events("write_acknowledgement")[0].get_first_attribute_value("packet_ack").unwrap();

	// We try to unpack the acknowledgement if possible, when it's following the standard format (is not enforced so it's not always possible)
	let parsed_ack : Result<AckResponse, serde_json::Error> = serde_json::from_str(&acknowledgment);

	let decoded_ack: String = if let Ok(ack_result) = parsed_ack{
		match ack_result{
			AckResponse::Result(b) => std::str::from_utf8(&general_purpose::STANDARD.decode(b)?)?.to_string(),
			AckResponse::Error(e) => e
		}
	}else{
		acknowledgment.clone()
	};

	log::info!("IBC packet n°{} : {}, received on {} on tx {}, with acknowledgment sent back: {}", 
		recv_packet_sequence,
		recv_packet_data,
		chain2.state.chain_id, 
		received_tx.txhash, 
		decoded_ack
	);  // This tx hash should also be analyzed for outgoing IBC transactions

	// 4. Finally, we check to see if the acknowledgment packet has been transferd alright on the origin chain
	let ack_events_string = vec![format!(
		"acknowledge_packet.packet_connection='{}'", connection
	), format!(
		"acknowledge_packet.packet_dst_port='{}'", dest_port
	), format!(
		"acknowledge_packet.packet_dst_channel='{}'", dest_channel
	), format!(
		"acknowledge_packet.packet_sequence='{}'", sequence
	)];
	let txs = Node::new(chain1.channel()).find_tx_by_events(ack_events_string, None, None).await.unwrap();

	if txs.len() != 1 {
		bail!(StdError::generic_err("Found multiple transactions matching the events, not possible"));
	}
	let ack_tx = &txs[0];
	// First we check if the tx errors (this shouldn't happen in IBC connections)
	if ack_tx.code != 0 {
		bail!(DaemonError::TxFailed {
            code: ack_tx.code,
            reason: format!("Raw log on {} : {}", chain1.state.chain_id, ack_tx.raw_log.clone()),
        })
    }	
    log::info!("IBC packet n°{} acknowledgment  received on {} on tx {}", sequence, chain1.state.chain_id, ack_tx.txhash); 

    // Now that we have followed the packet lifetime, we need to to the exact same procedure for the 2 transactions encountered, to make sure to follow all trails
    tokio::try_join!(
    	follow_trail(chain2, chain1, received_tx.txhash.clone()),
    	follow_trail(chain1, chain2, ack_tx.txhash.clone())
    ).unwrap();

	Ok(())
}
