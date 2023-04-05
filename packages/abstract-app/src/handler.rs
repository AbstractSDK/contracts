use crate::{AbstractContract, AppContract, AppError, Handler};

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AppError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_core::AbstractError>,
        InitMsg,
        ExecMsg,
        QueryMsg,
        MigrateMsg,
        Receive,
    > Handler for AppContract<Error, InitMsg, ExecMsg, QueryMsg, MigrateMsg, Receive>
{
    type Error = Error;
    type CustomInitMsg = InitMsg;
    type CustomExecMsg = ExecMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = MigrateMsg;
    type ReceiveMsg = Receive;

    fn contract(
        &self,
    ) -> &AbstractContract<Self, Error, InitMsg, ExecMsg, QueryMsg, MigrateMsg, Receive> {
        &self.contract
    }
}
