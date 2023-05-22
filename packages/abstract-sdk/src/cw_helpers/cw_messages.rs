use cosmwasm_std::CosmosMsg;

// When a method returns an AbstractMessage, this means this message needs to be dispatched to the account proxy using the execution api.
// The message method (which is only public to the crate), shouldn't be used to unwrap the message
#[derive(Debug, PartialEq, Clone)]
pub struct AbstractMessage(CosmosMsg);

impl From<CosmosMsg> for AbstractMessage {
    fn from(m: CosmosMsg) -> Self {
        Self(m)
    }
}

impl AbstractMessage {
    pub(crate) fn message(&self) -> CosmosMsg {
        self.0.clone()
    }
}
