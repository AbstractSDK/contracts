//! # Executor
//! The executor provides function for executing commands on the OS.
//!
use abstract_os::proxy::ExecuteMsg;
use cosmwasm_std::{
    to_binary, Attribute, CosmosMsg, Deps, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::features::Identification;

/// Execute an action on the OS or over IBC on a remote chain.
pub trait Execution: Identification {
    fn executor(&self) -> Executor<Self> {
        Executor { base: self }
    }
}


impl<T> Execution for T where T: Identification {}

pub struct Executor<'a, T: Execution> {
    pub base: &'a T,
}

impl<'a, T: Execution> Executor<'a, T> {
    pub fn execute(&self, msgs: Vec<CosmosMsg>) -> Result<CosmosMsg, StdError> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.base.proxy_address()?.to_string(),
            msg: to_binary(&ExecuteMsg::ModuleAction { msgs })?,
            funds: vec![],
        }))
    }
    pub fn execute_with_reply(
        &self,
        _deps: Deps,
        msgs: Vec<CosmosMsg>,
        reply_on: ReplyOn,
        id: u64,
    ) -> Result<SubMsg, StdError> {
        let msg = self.execute(msgs)?;
        let sub_msg = SubMsg {
            id,
            msg,
            gas_limit: None,
            reply_on,
        };
        Ok(sub_msg)
    }
    pub fn execute_response(
        &self,
        msgs: Vec<CosmosMsg>,
        action: &str,
    ) -> StdResult<Response>
    {
        let msg = self.execute(msgs)?;
        Ok(Response::new().add_message(msg).add_attribute("action", action))
    }
}
