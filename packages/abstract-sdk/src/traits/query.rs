use cosmwasm_std::{Binary, Deps, Env, StdError};

use crate::Handler;

pub trait QueryEndpoint: Handler {
    type QueryMsg<Msg>;

    fn query(
        &self,
        deps: Deps,
        env: Env,
        msg: Self::QueryMsg<Self::CustomQueryMsg>,
    ) -> Result<Binary, StdError>;
}
