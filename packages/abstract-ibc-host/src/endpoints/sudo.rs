use abstract_sdk::base::SudoEndpoint;

use crate::{state::ContractError, Host};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg, 
CResp,
    > SudoEndpoint
    for Host<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg, 
CResp,
    >
{
}
