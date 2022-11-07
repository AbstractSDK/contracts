use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError};
use serde::Serialize;

use crate::{base::contract_base::QueryHandlerFn, Handler};

pub trait QueryEndpoint: Handler {
    type QueryMsg<Msg>;

    fn query(
        &self,
        deps: Deps,
        env: Env,
        msg: Self::QueryMsg<Self::CustomQueryMsg>,
    ) -> Result<Binary, StdError>;
}
