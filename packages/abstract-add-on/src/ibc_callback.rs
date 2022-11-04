use abstract_sdk::{IbcCallbackEndpoint, IbcCallbackHandlerFn, Handler};

use crate::{AddOnContract, AddOnError};

impl<
        Error: From<cosmwasm_std::StdError> + From<AddOnError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
    
    IbcCallbackEndpoint for AddOnContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg,CustomMigrateMsg, ReceiveMsg>
{
}
