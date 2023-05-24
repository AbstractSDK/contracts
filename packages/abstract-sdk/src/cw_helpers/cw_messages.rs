use cosmwasm_std::Response;
use crate::AbstractSdkResult;
use crate::{Executor, Execution};
use cosmwasm_std::{CosmosMsg, ReplyOn, SubMsg};

// When a method returns an AccountAction, this means this message needs to be dispatched to the account proxy using the execution api.
// The message method (which is only public to the crate), shouldn't be used to unwrap the message
#[derive(Debug, PartialEq, Clone)]
pub struct AccountAction(Vec<CosmosMsg>);

impl From<CosmosMsg> for AccountAction {
    fn from(m: CosmosMsg) -> Self {
        Self(vec![m])
    }
} 

impl From<Vec<CosmosMsg>> for AccountAction {
    fn from(msgs: Vec<CosmosMsg>) -> Self {
        Self(msgs)
    }
}

impl AccountAction {

    pub fn empty() -> Self{
        Self(vec![])
    }

    pub(crate) fn messages(&self) -> Vec<CosmosMsg> {
        self.0.clone()
    }

    pub fn extend(&mut self, other: &Self){ 
        self.0.extend(other.messages())
    }

    pub fn execute<T: Execution>(self, app: Executor<T>) -> AbstractSdkResult<CosmosMsg>{
        app.execute(self)
    }

    pub fn execute_with_reply<T: Execution>(self, app: Executor<T>, reply_on: ReplyOn, id: u64) -> AbstractSdkResult<SubMsg>{
        app.execute_with_reply(self, reply_on, id)
    }

    pub fn execute_with_response<T: Execution>(self, app: Executor<T>, action: &str) -> AbstractSdkResult<Response>{
        app.execute_with_response(self, action)
    }
}

impl Extend<CosmosMsg> for AccountAction{

    fn extend<T>(&mut self, iter: T) where T: std::iter::IntoIterator<Item=CosmosMsg> { 
        self.0.extend(iter)
    }
}