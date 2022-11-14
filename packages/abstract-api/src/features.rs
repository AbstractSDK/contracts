use abstract_os::api::ApiRequestMsg;
use abstract_sdk::{
    base::Handler,
    features::{AbstractNameSystem, ContractDeps, Identification, Versioning}, Execution,
};
use cosmwasm_std::{Addr, CosmosMsg, Deps, StdError, StdResult, SubMsg};
use serde::Serialize;
use crate::{ApiContract, ApiError};

// implement the SDK features
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ContractDeps
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn deps(&self) -> &Deps {
        self.contract_deps().unwrap()
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > AbstractNameSystem
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn ans_host(&self) -> StdResult<abstract_sdk::ans_host::AnsHost> {
        Ok(self.base_state.load(self.deps().storage)?.ans_host)
    }
}

/// Retrieve identifying information about the calling OS
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > Identification for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn proxy_address(&self) -> StdResult<Addr> {
        if let Some(target) = &self.target_os {
            Ok(target.proxy.clone())
        } else {
            Err(StdError::generic_err(
                "No target OS specified to execute on.",
            ))
        }
    }

    fn manager_address(&self) -> StdResult<Addr> {
        if let Some(target) = &self.target_os {
            Ok(target.manager.clone())
        } else {
            Err(StdError::generic_err(
                "No OS manager specified.",
            ))
        }
    }

    fn os_core(&self) -> StdResult<abstract_os::version_control::Core> {
        if let Some(target) = &self.target_os {
            Ok(target.clone())
        } else {
            Err(StdError::generic_err(
                "No OS core specified.",
            ))
        }
    }
    
}

/// Get the version control contract
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > Versioning
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn version_registry(&self) -> StdResult<Addr> {
        Ok(self.state(self.deps().storage)?.version_control)
    }
}
