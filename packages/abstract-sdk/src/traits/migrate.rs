use cosmwasm_std::{DepsMut, Env, Response};

use crate::Handler;

pub type Name = &'static str;
pub type VersionString = &'static str;

pub trait MigrateEndpoint: Handler {
    type MigrateMsg<Msg>;
    fn migrate(
        self,
        deps: DepsMut,
        env: Env,
        msg: Self::MigrateMsg<Self::CustomMigrateMsg>,
    ) -> Result<Response, Self::Error>;
}
