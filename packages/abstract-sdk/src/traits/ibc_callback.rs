use abstract_os::simple_ica::{IbcResponseMsg, StdAck};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub type IbcCallbackHandlerFn<Module, Error> =
    fn(DepsMut, Env, MessageInfo, Module, String, StdAck) -> Result<Response, Error>;

pub trait IbcCallbackEndpoint: Sized {
    type ContractError: From<cosmwasm_std::StdError>;
    /// Takes request, sets destination and executes request handler
    /// This fn is the only way to get an ApiContract instance which ensures the destination address is set correctly.
    fn callback_handler(&self, id: &str) -> Option<IbcCallbackHandlerFn<Self, Self::ContractError>>;
    fn handle_ibc_callback(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: IbcResponseMsg,
    ) -> Result<Response, Self::ContractError> {
        let IbcResponseMsg { id, msg: ack } = msg;
        let maybe_handler = self.callback_handler(&id);
        maybe_handler.map_or_else(
            || Ok(Response::new()),
            |f| f(deps, env, info, self, id, ack),
        )
    }
}
