use crate::{error::ApiError, state::ApiContract};

use abstract_sdk::{ReceiveEndpoint, ReceiveHandlerFn};

impl<'a, T, E: From<cosmwasm_std::StdError> + From<ApiError>, R> ReceiveEndpoint
    for ApiContract<'a, T, E, R>
{
    type ContractError = E;
    type ReceiveMsg = R;

    fn receive_handler(
        &self,
    ) -> Option<ReceiveHandlerFn<Self, Self::ReceiveMsg, Self::ContractError>> {
        self.receive_handler
    }
}
