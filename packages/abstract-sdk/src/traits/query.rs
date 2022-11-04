use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdError, Binary};

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
