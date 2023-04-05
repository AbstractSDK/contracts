use crate::{error::AppError, state::AppContract, ReceiveEndpoint};

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
    > ReceiveEndpoint
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
