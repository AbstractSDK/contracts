//! # Transfer
//! The Transfer object handles asset transfers to and from the OS.

use abstract_os::objects::AnsAsset;
use cosmwasm_std::{Addr, CosmosMsg, Deps, StdResult};

use super::execution::Execution;
use super::AbstractNameSystem;
use crate::ans_resolve::Resolve;

pub trait TransferInterface: AbstractNameSystem + Execution {
    fn transfer<'a>(&'a self, deps: Deps<'a>) -> Transfer<Self> {
        Transfer { base: self, deps }
    }
}

impl<T> TransferInterface for T where T: AbstractNameSystem + Execution {}

#[derive(Clone)]
pub struct Transfer<'a, T: TransferInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: TransferInterface> Transfer<'a, T> {
    /// Transfer funds from the OS
    pub fn transfer(&self, funds: Vec<AnsAsset>, to: &Addr) -> StdResult<CosmosMsg> {
        let resolved_funds = funds.resolve(&self.deps.querier, &self.base.ans_host(self.deps)?)?;
        let transfer_msgs = resolved_funds
            .iter()
            .map(|asset| asset.transfer_msg(to.clone()))
            .collect::<StdResult<Vec<CosmosMsg>>>();
        self.base.executor(self.deps).execute(transfer_msgs?)
    }

    /// Deposit into the OS
    pub fn deposit(&self, funds: Vec<AnsAsset>) -> StdResult<Vec<CosmosMsg>> {
        let to = self.base.proxy_address(self.deps)?;
        let resolved_funds = funds.resolve(&self.deps.querier, &self.base.ans_host(self.deps)?)?;
        resolved_funds
            .iter()
            .map(|asset| asset.transfer_msg(to.clone()))
            .collect::<StdResult<Vec<CosmosMsg>>>()
    }
}
