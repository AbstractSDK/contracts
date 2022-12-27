use crate::{error::ApiError, state::ApiContract, ApiResult};
use abstract_os::api::ApiExecuteMsg;
use abstract_sdk::{
    base::{
        endpoints::{ExecuteEndpoint, IbcCallbackEndpoint, ReceiveEndpoint},
        Handler,
    },
    os::api::{BaseExecuteMsg, ExecuteMsg},
    Execution, ModuleInterface, Verification,
};
use cosmwasm_std::{
    to_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, WasmMsg,
};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg: Serialize + JsonSchema + ApiExecuteMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg: Serialize + JsonSchema,
    > ExecuteEndpoint
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;

    fn execute(
        mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Error> {
        let sender = &info.sender;
        match msg {
            ExecuteMsg::App(request) => {
                let core = match request.proxy_address {
                    Some(addr) => {
                        let proxy_addr = deps.api.addr_validate(&addr)?;
                        let traders = self.traders.load(deps.storage, proxy_addr)?;
                        if traders.contains(sender) {
                            self.os_register(deps.as_ref())
                                .assert_proxy(&deps.api.addr_validate(&addr)?)?
                        } else {
                            self.os_register(deps.as_ref())
                                .assert_manager(sender)
                                .map_err(|_| ApiError::UnauthorizedTraderApiRequest {})?
                        }
                    }
                    None => self
                        .os_register(deps.as_ref())
                        .assert_manager(sender)
                        .map_err(|_| ApiError::UnauthorizedTraderApiRequest {})?,
                };
                self.target_os = Some(core);
                self.execute_handler()?(deps, env, info, self, request.request)
            }
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info.clone(), exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.handle_ibc_callback(deps, env, info, msg),
            ExecuteMsg::Receive(msg) => self.handle_receive(deps, env, info, msg),
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported api execute message variant").into()),
        }
    }
}

/// The api-contract base implementation.
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn base_execute(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> ApiResult {
        match message {
            BaseExecuteMsg::UpdateTraders { to_add, to_remove } => {
                self.update_traders(deps, info, to_add, to_remove)
            }
            BaseExecuteMsg::Remove {} => self.remove_self_from_deps(deps.as_ref(), env, info),
        }
    }

    /// If dependencies are set, remove self from them.
    pub(crate) fn remove_self_from_deps(
        &mut self,
        deps: Deps,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ApiError> {
        let core = self
            .os_register(deps)
            .assert_manager(&info.sender)
            .map_err(|_| ApiError::UnauthorizedApiRequest {})?;
        self.target_os = Some(core);
        let dependencies = self.dependencies();
        let mut msgs: Vec<CosmosMsg> = vec![];
        let applications = self.modules(deps);
        for dep in dependencies {
            let api_addr = applications.module_address(dep.id);
            // just skip if dep is already removed. This means all the traders are already removed.
            if api_addr.is_err() {
                continue;
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api_addr?.into_string(),
                msg: to_binary(&BaseExecuteMsg::UpdateTraders {
                    to_add: None,
                    to_remove: Some(vec![env.contract.address.to_string()]),
                })?,
                funds: vec![],
            }));
        }
        self.executor(deps)
            .execute_with_response(msgs, "remove api from dependencies")
            .map_err(Into::into)
    }

    /// Remove traders from the api.
    fn update_traders(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<Response, ApiError> {
        // Either manager or proxy can add/remove traders.
        // This allows other apis to automatically add themselves, allowing for api-cross-calling.
        let core = self
            .os_register(deps.as_ref())
            .assert_manager(&info.sender)?;

        // Manager can only change traders for associated proxy
        let proxy = core.proxy;

        let mut traders = self
            .traders
            .may_load(deps.storage, proxy.clone())?
            .unwrap_or_default();

        // Handle the addition of traders
        if let Some(to_add) = to_add {
            for trader in to_add {
                let trader_addr = deps.api.addr_validate(trader.as_str())?;
                if !traders.insert(trader_addr) {
                    return Err(ApiError::TraderAlreadyPresent { trader });
                }
            }
        }

        // Handling the removal of traders
        if let Some(to_remove) = to_remove {
            for trader in to_remove {
                let trader_addr = deps.api.addr_validate(trader.as_str())?;
                if !traders.remove(&trader_addr) {
                    return Err(ApiError::TraderNotPresent { trader });
                }
            }
        }

        self.traders.save(deps.storage, proxy.clone(), &traders)?;
        Ok(Response::new().add_attribute("action", format!("update_{}_traders", proxy)))
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use cosmwasm_std::{testing::{MockQuerier, mock_env,mock_info, mock_dependencies}, WasmQuery, Binary, Addr, SystemResult, ContractResult, Empty};
    use thiserror::Error;

    use super::*;
    const TEST_MODULE_ADDRESS: &str = "test_module_address";
    const TEST_MANAGER: &str = "manager";
    const TEST_SENDER: &str = "sender";
    const TEST_PROXY: &str = "proxy";

    type TestApi = ApiContract::<TestError, Empty, Empty, Empty, Empty>;
    type ApiTestResult = Result<(), TestError>;

    #[derive(Error, Debug, PartialEq)]
    enum TestError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error(transparent)]
        Api(#[from] ApiError),
    }

    /// mock querier that has the os modules loaded
    fn mock_querier_with_existing_module() -> MockQuerier {
        let mut querier = MockQuerier::default();

        querier.update_wasm(|wasm| {
            match wasm {
                WasmQuery::Raw { contract_addr, key } => {
                    let string_key = String::from_utf8(key.to_vec()).unwrap();
                    let str_key = string_key.as_str();

                    let mut modules = HashMap::<Binary, Addr>::default();

                    // binary key is "os_modules<module_id>" (though with a \n or \r before)
                    let binary = Binary::from_base64("AApvc19tb2R1bGVzdGVzdF9tb2R1bGU=").unwrap();
                    modules.insert(binary, Addr::unchecked(TEST_MODULE_ADDRESS));

                    let res = match contract_addr.as_str() {
                        TEST_PROXY => match str_key {
                            "admin" => Ok(to_binary(&TEST_MANAGER).unwrap()),
                            _ => Err("unexpected key"),
                        },
                        TEST_MANAGER => {
                            if let Some(value) = modules.get(key) {
                                Ok(to_binary(&value.to_owned().clone()).unwrap())
                            } else {
                                // Debug print out what the key was
                                // let into_binary: Binary = b"\ros_modulestest_module".into();
                                // let to_binary_res =
                                //     to_binary("os_modulestest_module".as_bytes()).unwrap();
                                // panic!(
                                //     "contract: {}, binary_key: {}, into_binary: {}, to_binary_res: {}",
                                //     contract_addr, key, into_binary, to_binary_res
                                // );
                                Ok(Binary::default())
                            }
                        }
                        _ => Err("unexpected contract"),
                    };

                    match res {
                        Ok(res) => SystemResult::Ok(ContractResult::Ok(res)),
                        Err(e) => SystemResult::Ok(ContractResult::Err(e.to_string())),
                    }
                }
                _ => panic!("Unexpected smart query"),
            }
        });

        querier
    }

    #[test]
    fn add_trader() -> ApiTestResult {
        let mut api = TestApi::new("mock", "v1.9.9", None);
        let env = mock_env();
        let info = mock_info(TEST_SENDER, &vec![]);
        let mut deps = mock_dependencies();
        let msg = BaseExecuteMsg::UpdateTraders { to_add: None, to_remove: None };
        api.base_execute(deps.as_mut(), env, info, msg)?;

        Ok(())
    }
}