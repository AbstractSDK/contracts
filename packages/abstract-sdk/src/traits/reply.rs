use cosmwasm_std::{DepsMut, Env, Reply, Response, StdError};

use crate::{base::contract_base::ReplyHandlerFn, Handler};

pub trait ReplyEndpoint: Handler {
    fn handle_reply(self, deps: DepsMut, env: Env, msg: Reply) -> Result<Response, Self::Error> {
        let id = msg.id;
        let handler = self.reply_handler(id)?;
        handler(deps, env, self, msg)
    }
}
