use crate::base::handler::Handler;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use schemars::JsonSchema;
use serde::Serialize;

pub trait SudoEndpoint: Handler {
    type SudoMsg: Serialize + JsonSchema;

    /// Handler for the Sudo endpoint.
    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::SudoMsg,
    ) -> Result<Response, Self::Error>;
}
