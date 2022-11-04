use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::Handler;

pub trait InstantiateEndpoint: Handler {
    type InstantiateMsg<Msg>;

    /// Instantiate the base contract
    fn instantiate(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::InstantiateMsg<Self::CustomInitMsg>,
    ) -> Result<Response, Self::Error>;
}
