//! # Transfer
//! The Transfer object handles asset transfers to and from the OS.

use abstract_os::objects::AnsAsset;
use cosmwasm_std::{Addr, CosmosMsg, StdResult};

use crate::{
    ans_host_traits::Resolve,
    features::{AbstractNameSystem, Identification},
};

use super::execution::Execution;

pub trait TransferInterface: AbstractNameSystem + Execution {
    fn transfer(&self) -> Transfer<Self> {
        Transfer { base: &self }
    }
}

impl<T> TransferInterface for T where T: AbstractNameSystem + Execution {}

pub struct Transfer<'a, T: TransferInterface> {
    base: &'a T,
}

impl<'a, T: TransferInterface> Transfer<'a, T> {
    /// Transfer funds from the OS
    pub fn transfer(&self, funds: Vec<AnsAsset>, to: &Addr) -> StdResult<CosmosMsg> {
        let resolved_funds = funds.resolve(self.base.querier(), &self.base.ans_host()?)?;
        let transfer_msgs = resolved_funds
            .iter()
            .map(|asset| asset.transfer_msg(to.clone()))
            .collect::<StdResult<Vec<CosmosMsg>>>();
        self.base.executor().execute(transfer_msgs?)
    }

    /// Deposit into the OS
    pub fn deposit(&self, funds: Vec<AnsAsset>) -> StdResult<Vec<CosmosMsg>> {
        let to = self.base.proxy_address()?;
        let resolved_funds = funds.resolve(self.base.querier(), &self.base.ans_host()?)?;
        resolved_funds
            .iter()
            .map(|asset| asset.transfer_msg(to.clone()))
            .collect::<StdResult<Vec<CosmosMsg>>>()
    }
}
