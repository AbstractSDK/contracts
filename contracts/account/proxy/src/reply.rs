use cosmwasm_std::Reply;

use crate::contract::{ProxyResponse, ProxyResult};

/// Add the message's data to the response
pub fn forward_response_data(result: Reply) -> ProxyResult {
    // get the result from the reply
    let reps = cw_utils::parse_reply_execute_data(result)?;

    // log and add data if needed
    let resp = if reps.data.is_none() {
        ProxyResponse::new(
            "forward_response_data_reply",
            vec![("response_data", "false")],
        )
    } else {
        ProxyResponse::new(
            "forward_response_data_reply",
            vec![("response_data", "true")],
        )
        .set_data(reps.data.unwrap())
    };

    Ok(resp)
}
