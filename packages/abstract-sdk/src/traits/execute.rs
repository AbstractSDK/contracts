use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::base::handler::Handler;

pub trait ExecuteEndpoint: Handler{
    type ExecuteMsg<Msg> ;

    /// Entry point for contract execution
    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg<Self::CustomExecMsg>,
    ) -> Result<Response, Self::Error>;
}
