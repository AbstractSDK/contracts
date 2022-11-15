use crate::{error::AddOnError, state::AddOnContract};

use crate::ReceiveEndpoint;

impl<
        Error: From<cosmwasm_std::StdError> + From<AddOnError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > ReceiveEndpoint
    for AddOnContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
}
