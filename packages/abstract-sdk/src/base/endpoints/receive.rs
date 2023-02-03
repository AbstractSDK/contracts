use crate::{base::Handler, SdkError};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub trait ReceiveEndpoint: Handler {
    fn handle_receive(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: <Self as Handler>::ReceiveMsg,
    ) -> Result<Response, <Self as Handler>::Error> {
        let maybe_handler = self.maybe_receive_handler();
        maybe_handler.map_or_else(
            || {
                Err(Self::Error::from(SdkError::MissingHandler {
                    endpoint: "receive".to_string(),
                }))
            },
            |f| f(deps, env, info, self, msg),
        )
    }
}
