use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub trait AbstractExecute: Sized {
    type RequestMsg;
    type ExecuteMsg<T>;
    type ContractError: From<cosmwasm_std::StdError>;

    /// Takes request, sets destination and executes request handler
    /// This fn is the only way to get an ApiContract instance which ensures the destination address is set correctly.
    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg<Self::RequestMsg>,
        request_handler: impl FnOnce(
            DepsMut,
            Env,
            MessageInfo,
            Self,
            Self::RequestMsg,
        ) -> Result<Response, Self::ContractError>,
    ) -> Result<Response, Self::ContractError>;
}
