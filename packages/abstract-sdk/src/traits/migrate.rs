use cosmwasm_std::{DepsMut, Env, Response, StdError};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

use crate::{base::contract_base::MigrateHandlerFn, Handler};

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
