use abstract_os::{
    add_on::{BaseExecuteMsg, ExecuteMsg},
    simple_ica::{IbcResponseMsg, StdAck},
};

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
use serde::{de::DeserializeOwned, Serialize};

use crate::{state::AddOnContract, AddOnError, AddOnResult};

pub type IbcHandlerFn<T, E, C> =
    fn(DepsMut, Env, MessageInfo, AddOnContract<T, E, C>, String, StdAck) -> Result<Response, E>;

impl<
        'a,
        T: Serialize + DeserializeOwned,
        C: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<AddOnError>,
    > AddOnContract<'a, T, E, C>
{
    /// Takes request, sets destination and executes request handler
    /// This fn is the only way to get an ApiContract instance which ensures the destination address is set correctly.
    pub fn handle_request(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<T>,
        request_handler: impl FnOnce(DepsMut, Env, MessageInfo, Self, T) -> Result<Response, E>,
    ) -> Result<Response, E> {
        match msg {
            ExecuteMsg::Request(request) => request_handler(deps, env, info, self, request),
            ExecuteMsg::Configure(exec_msg) => self
                .execute(deps, env, info.clone(), exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(IbcResponseMsg { id, msg }) => {
                for ibc_callback_handler in self.ibc_callbacks {
                    if ibc_callback_handler.0 == id {
                        return ibc_callback_handler.1(deps, env, info, self, id, msg);
                    }
                }
                Ok(Response::new()
                    .add_attribute("action", "ibc_response")
                    .add_attribute("response_id", id))
            }
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported AddOn execute message variant").into()),
        }
    }
    pub fn execute(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> AddOnResult {
        match message {
            BaseExecuteMsg::UpdateConfig { memory_address } => {
                self.update_config(deps, info, memory_address)
            }
        }
    }

    fn update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        memory_address: Option<String>,
    ) -> AddOnResult {
        // self._update_config(deps, info, memory_address)?;
        // Only the admin should be able to call this
        self.admin.assert_admin(deps.as_ref(), &info.sender)?;

        let mut state = self.base_state.load(deps.storage)?;

        if let Some(memory_address) = memory_address {
            state.memory.address = deps.api.addr_validate(memory_address.as_str())?;
        }

        self.base_state.save(deps.storage, &state)?;

        Ok(Response::default().add_attribute("action", "updated_memory_address"))
    }
}
