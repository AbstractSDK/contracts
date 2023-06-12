use cosmwasm_std::SubMsgResponse;

use crate::contract::{ProxyResponse, ProxyResult};

/// Add the message's data to the response
pub fn forward_response_data(result: SubMsgResponse) -> ProxyResult {
    // log if none
    let attr = if result.data.is_none() {
        ("response_data", "false")
    } else {
        ("response_data", "true")
    };

    let mut resp = ProxyResponse::new("forward_response_data_reply", vec![attr]);

    // set the data
    resp.data = result.data;

    Ok(resp)
}
