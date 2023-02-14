use crate::{ApiContract, ApiError};
use abstract_sdk::base::endpoints::IbcCallbackEndpoint;
use abstract_sdk::{EndpointError, SdkError};

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<SdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > IbcCallbackEndpoint
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
}
