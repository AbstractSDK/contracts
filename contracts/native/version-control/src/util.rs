use crate::{contract::VCResult, error::VCError};

use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, MessageInfo};

pub fn validate_native_funds(
    msg_info: MessageInfo,
    fee: Coin,
    receiver: Option<Addr>,
) -> VCResult<Vec<CosmosMsg>> {
    if fee.amount.is_zero() {
        return Ok(vec![]);
    }

    if msg_info.funds.len() != 1
        || msg_info.funds[0].denom != fee.denom
        || fee.amount != msg_info.funds[0].amount
    {
        return Err(VCError::InvalidFeePayment {
            expected: fee,
            sent: msg_info.funds,
        });
    }
    let fee_messages = if let Some(receiver) = receiver {
        vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: receiver.to_string(),
            amount: msg_info.funds,
        })]
    } else {
        vec![]
    };

    Ok(fee_messages)
}
