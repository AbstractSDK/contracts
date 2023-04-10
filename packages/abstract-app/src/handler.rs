use crate::{state::ContractError, AbstractContract, AppContract, Handler};

impl<Error: ContractError, InitMsg, ExecMsg, QueryMsg, MigrateMsg, SudoMsg, Receive> Handler
    for AppContract<Error, InitMsg, ExecMsg, QueryMsg, MigrateMsg, SudoMsg, Receive>
{
    type Error = Error;
    type CustomInitMsg = InitMsg;
    type CustomExecMsg = ExecMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = MigrateMsg;
    type SudoMsg = SudoMsg;
    type ReceiveMsg = Receive;

    fn contract(&self) -> &AbstractContract<Self, Error> {
        &self.contract
    }
}
