use crate::{AppContract, AppError, ReplyEndpoint};

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AppError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_core::AbstractError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > ReplyEndpoint
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
}
