use crate::{state::AppContract, AppError, AppResult};
use crate::{ExecuteEndpoint, Handler, IbcCallbackEndpoint};
use abstract_sdk::{
    base::ReceiveEndpoint,
    os::app::{AppExecuteMsg, BaseExecuteMsg, ExecuteMsg},
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + 'static,
        CustomExecMsg: Serialize + JsonSchema + AppExecuteMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg: Serialize + JsonSchema,
    > ExecuteEndpoint
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;

    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Error> {
        match msg {
            ExecuteMsg::App(request) => self.execute_handler()?(deps, env, info, self, request),
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info, exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.handle_ibc_callback(deps, env, info, msg),
            ExecuteMsg::Receive(msg) => self.handle_receive(deps, env, info, msg),
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported App execute message variant").into()),
        }
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
    AppContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn base_execute(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> AppResult {
        match message {
            BaseExecuteMsg::UpdateConfig { ans_host_address } => {
                self.update_config(deps, info, ans_host_address)
            }
        }
    }

    fn update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        ans_host_address: Option<String>,
    ) -> AppResult {
        // self._update_config(deps, info, ans_host_address)?;
        // Only the admin should be able to call this
        self.admin.assert_admin(deps.as_ref(), &info.sender)?;

        let mut state = self.base_state.load(deps.storage)?;

        if let Some(ans_host_address) = ans_host_address {
            state.ans_host.address = deps.api.addr_validate(ans_host_address.as_str())?;
        }

        self.base_state.save(deps.storage, &state)?;

        Ok(Response::default().add_attribute("action", "update_config"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_common::*;
    use abstract_testing::TEST_ADMIN;

    type AppExecuteMsg = ExecuteMsg<MockExecMsg, MockReceiveMsg>;

    fn execute_as(deps: DepsMut, sender: &str, msg: AppExecuteMsg) -> Result<Response, MockError> {
        let info = mock_info(sender, &[]);
        MOCK_APP.execute(deps, mock_env(), info, msg)
    }

    fn execute_as_admin(deps: DepsMut, msg: AppExecuteMsg) -> Result<Response, MockError> {
        execute_as(deps, TEST_ADMIN, msg)
    }

    mod update_config {
        use super::*;
        use cosmwasm_std::testing::{mock_dependencies, mock_info};

        // #[test]
        // fn should_update_config() {
        //     let mut deps = mock_dependencies();
        //     let mut state = mock_base_state();
        //     state.ans_host.address = deps.api.addr_validate("ans_host").unwrap();
        //     state.save(deps.as_mut().storage).unwrap();
        //
        //     let msg = BaseExecuteMsg::UpdateConfig {
        //         ans_host_address: Some("new_ans_host".to_string()),
        //     };
        //
        //     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        //     assert_eq!(0, res.messages.len());
        //     assert_eq!(0, res.attributes.len());
        //     assert_eq!(0, res.data.len());
        //     assert_eq!(0, res.events.len());
        //     assert_eq!(0, res.log.len());
        //
        //     let state = BaseState::load(deps.as_ref().storage).unwrap();
        //     assert_eq!(
        //         deps.api.addr_validate("new_ans_host").unwrap(),
        //         state.ans_host.address
        //     );
        // }
    }
}
