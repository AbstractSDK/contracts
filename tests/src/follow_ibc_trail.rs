use base64::{engine::general_purpose, Engine as _};

use anyhow::{bail, Result};
use cosmwasm_std::StdError;
use cw_orch::queriers::DaemonQuerier;
use cw_orch::queriers::Node;
use cw_orch::{Daemon, DaemonError};

// This was coded thanks to this wonderful guide : https://github.com/CosmWasm/cosmwasm/blob/main/IBC.md

// type is from cosmos_sdk_proto::ibc::core::channel::v1::acknowledgement::Response
#[cosmwasm_schema::cw_serde]
pub enum AckResponse {
    Result(String), // This is a base64 string
    Error(String),
}

#[async_recursion::async_recursion(?Send)]
pub async fn follow_trail(chain1: &Daemon, chain2: &Daemon, tx_hash: String) -> Result<()> {

    // In this function, we need to :
    // 1. Get all ibc outgoing messages from the transaction
    // attribute type : send_packet
    // Things to track
    // connection
    // dest-port
    // dest_channel
    // packet_sequence
    // timeout_timestamp (for stopping the search) - Not needed here
    let tx = Node::new(chain1.channel())
        .find_tx_by_hash(tx_hash.clone())
        .await
        .unwrap();
    let send_packet_events = tx.get_events("send_packet");
    if send_packet_events.is_empty() {
        return Ok(());
    }
    log::info!("Investigating sent packet events on tx {}", tx_hash.clone());

    // We account for a single send_packet event at a time for now (TODO)
    let connections: Vec<String> = send_packet_events
        .iter()
        .map(|e| e.get_first_attribute_value("packet_connection").unwrap())
        .collect();
    let dest_ports: Vec<String> = send_packet_events
        .iter()
        .map(|e| e.get_first_attribute_value("packet_dst_port").unwrap())
        .collect();
    let dest_channels: Vec<String> = send_packet_events
        .iter()
        .map(|e| e.get_first_attribute_value("packet_dst_channel").unwrap())
        .collect();
    let sequences: Vec<String> = send_packet_events
        .iter()
        .map(|e| e.get_first_attribute_value("packet_sequence").unwrap())
        .collect();
    let packet_datas: Vec<String> = send_packet_events
        .iter()
        .map(|e| e.get_first_attribute_value("packet_data").unwrap())
        .collect();

    // 2. For each message find the transaction hash of the txs the message during which the message is broadcasted to the distant chain
    // This only works for 2 chains for now, we don't handle more chains
    let events_strings = connections.iter().enumerate().map(|(i, _ )| {
        log::info!(
            "IBC packet n° {}, sent on {} on tx {}, with data: {}",
            sequences[i],
            chain1.state.chain_id,
            tx_hash,
            packet_datas[i]
        );

        vec![
        format!("recv_packet.packet_connection='{}'", connections[i]),
        format!("recv_packet.packet_dst_port='{}'", dest_ports[i]),
        format!("recv_packet.packet_dst_channel='{}'", dest_channels[i]),
        format!("recv_packet.packet_sequence='{}'", sequences[i]),
    ]});

    let received_txs: Vec<_> = events_strings.map(|event_query| async move {
        let txs = Node::new(chain2.channel())
            .find_tx_by_events(event_query, None, None)
            .await
            .unwrap();

        // We need to make sure there is only 1 transaction with such events (always should be the case)
        if txs.len() != 1 {
            bail!(StdError::generic_err(
                "Found multiple transactions matching the events, not possible"
            ));
        }
        let received_tx = &txs[0];
        // We check if the tx errors (this shouldn't happen in IBC connections)
        if received_tx.code != 0 {
            bail!(DaemonError::TxFailed {
                code: received_tx.code,
                reason: format!(
                    "Raw log on {} : {}",
                    chain2.state.chain_id,
                    received_tx.raw_log.clone()
                ),
            });
        }
        Ok(received_tx.clone())
    }).collect();

    // We await all transactions
    let mut received_txs_awaited = vec![];
    for tx in received_txs{
        received_txs_awaited.push(tx.await?);
    }


    let ack_txs: Vec<_> = received_txs_awaited.iter().enumerate().map(|(i, received_tx)|{
        let this_connection = connections[i].clone();
        let this_dest_channel = dest_channels[i].clone();
        let this_dest_port = dest_ports[i].clone();
        let this_sequence = sequences[i].clone();


        async move{
        // 3. Then we look for the acknowledgment packet that should always be traced back during this transaction for all packets
        let recv_packet_sequence = received_tx.get_events("write_acknowledgement")[0] // There is only one acknowledgement per transaction possible
            .get_first_attribute_value("packet_sequence")
            .unwrap();
        let recv_packet_data = received_tx.get_events("write_acknowledgement")[0]
            .get_first_attribute_value("packet_data")
            .unwrap();
        let acknowledgment = received_tx.get_events("write_acknowledgement")[0]
            .get_first_attribute_value("packet_ack")
            .unwrap();

        // We try to unpack the acknowledgement if possible, when it's following the standard format (is not enforced so it's not always possible)
        let parsed_ack: Result<AckResponse, serde_json::Error> = serde_json::from_str(&acknowledgment);

        let decoded_ack: String = if let Ok(ack_result) = parsed_ack {
            match ack_result {
                AckResponse::Result(b) => {
                    std::str::from_utf8(&general_purpose::STANDARD.decode(b)?)?.to_string()
                }
                AckResponse::Error(e) => e,
            }
        } else {
            acknowledgment.clone()
        };

        
        log::info!(
            "IBC packet n°{} : {}, received on {} on tx {}, with acknowledgment sent back: {}",
            recv_packet_sequence,
            recv_packet_data,
            chain2.state.chain_id,
            received_tx.txhash,
            decoded_ack
        );



        // 4. Finally, we check to see if the acknowledgment packet has been transferd alright on the origin chain
        let ack_events_string = vec![
            format!("acknowledge_packet.packet_connection='{}'", this_connection),
            format!("acknowledge_packet.packet_dst_port='{}'", this_dest_port),
            format!("acknowledge_packet.packet_dst_channel='{}'", this_dest_channel),
            format!("acknowledge_packet.packet_sequence='{}'", this_sequence),
        ];
        let txs = Node::new(chain1.channel())
            .find_tx_by_events(ack_events_string, None, None)
            .await
            .unwrap();

        if txs.len() != 1 {
            bail!(StdError::generic_err(
                "Found multiple transactions matching the events, not possible"
            ));
        }
        let ack_tx = &txs[0];
        // First we check if the tx errors (this shouldn't happen in IBC connections)
        if ack_tx.code != 0 {
            bail!(DaemonError::TxFailed {
                code: ack_tx.code,
                reason: format!(
                    "Raw log on {} : {}",
                    chain1.state.chain_id,
                    ack_tx.raw_log.clone()
                ),
            })
        }
        log::info!(
            "IBC packet n°{} acknowledgment  received on {} on tx {}",
            this_sequence,
            chain1.state.chain_id,
            ack_tx.txhash
        );

        Ok(ack_tx.clone())
    }})
    .collect();

    // We await all transactions
    let mut ack_txs_awaited = vec![];
    for tx in ack_txs{
        ack_txs_awaited.push(tx.await?);
    }

    
    // All the tx hashes should now should also be analyzed for outgoing IBC transactions

    let received_hashes: Vec<_> = received_txs_awaited.iter().map(|tx| tx.txhash.clone()).collect();
    let ack_hashes: Vec<_> = ack_txs_awaited.iter().map(|tx| tx.txhash.clone()).collect();

    let mut follow_trail_futures: Vec<_> = received_hashes.iter().map(|hash| follow_trail(chain2, chain1, hash.to_string())).collect();
    follow_trail_futures.append(&mut ack_hashes.iter().map(|hash| follow_trail(chain1, chain2, hash.to_string())).collect());


    for fut in follow_trail_futures{
        fut.await?;
    }

    Ok(())
}
