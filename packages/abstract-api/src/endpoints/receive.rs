use crate::state::{ApiContract, ContractError};
use abstract_sdk::base::endpoints::ReceiveEndpoint;

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg, ReceiveMsg>
    ReceiveEndpoint
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg, ReceiveMsg>
{
}
