//! # Fee helpers
//! Helper trait that lets you easily charge fees on assets

use cosmwasm_std::{StdResult, CosmosMsg};
use cw_asset::Asset;
use os::objects::fee::{TransferFee, Fee};

/// Indicates that the implementing type can be charged fees.
pub trait Chargeable {
    fn charge_fee(&mut self, fee: Fee) -> StdResult<()>;
    fn charge_transfer_fee(&mut self, fee: TransferFee) -> StdResult<CosmosMsg>;
}

impl Chargeable for Asset {
    fn charge_fee(&mut self, fee: Fee) -> StdResult<()> {
        let fee_amount = fee.compute(self.amount);
        self.amount -= fee_amount;
        Ok(())
    }

    fn charge_transfer_fee(&mut self, fee: TransferFee) -> StdResult<CosmosMsg> {
        let fee_amount = fee.compute(self.amount);
        self.amount -= fee_amount;
        Asset::new(self.info.clone(), fee_amount).transfer_msg(fee.recipient())
    }
}