use cosmwasm_std::{Addr, Deps, StdResult};

use crate::{AddOnContract, AddOnError};
use abstract_sdk::{
    base::features::{AbstractNameSystem, Identification},
    feature_objects::AnsHost,
};
impl<
        Error: From<cosmwasm_std::StdError> + From<AddOnError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > AbstractNameSystem
    for AddOnContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    fn ans_host(&self, deps: Deps) -> StdResult<AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<AddOnError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Identification
    for AddOnContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    fn proxy_address(&self, deps: Deps) -> StdResult<Addr> {
        Ok(self.base_state.load(deps.storage)?.proxy_address)
    }
}
