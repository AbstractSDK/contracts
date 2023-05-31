use crate::{contract::VCResult, error::VCError};

use cosmwasm_std::{Coin, MessageInfo};

pub fn validate_native_funds(msg_info: MessageInfo, fee: Coin) -> VCResult<()> {
    if fee.amount.is_zero() {
        return Ok(());
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
    Ok(())
}
