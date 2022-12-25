//! # Module
//! The Module interface provides helper functions to execute functions on other modules installed on the OS.

use abstract_os::{
    api, app,
    manager::state::{ModuleId, OS_MODULES},
};
use cosmwasm_std::{
    wasm_execute, Addr, CosmosMsg, Deps, Empty, QueryRequest, StdError, StdResult, WasmQuery,
};
use cw2::{ContractVersion, CONTRACT};
use serde::Serialize;

use super::{Dependencies, Identification};

/// Interact with other modules on the OS.
pub trait ModuleInterface: Identification + Dependencies {
    fn modules<'a>(&'a self, deps: Deps<'a>) -> Modules<Self> {
        Modules { base: self, deps }
    }
}

impl<T> ModuleInterface for T where T: Identification + Dependencies {}

#[derive(Clone)]
pub struct Modules<'a, T: ModuleInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ModuleInterface> Modules<'a, T> {
    /// Retrieve the address of an application in this OS.
    /// This should **not** be used to execute messages on an `Api`.
    /// Use `Modules::api_request(..)` instead.
    pub fn module_address(&self, module_id: ModuleId) -> StdResult<Addr> {
        let manager_addr = self.base.manager_address(self.deps)?;
        let maybe_module_addr = OS_MODULES.query(&self.deps.querier, manager_addr, module_id)?;
        let Some(module_addr) = maybe_module_addr else {
            return Err(StdError::generic_err(format!("Module {} not enabled on OS.", module_id)));
        };
        Ok(module_addr)
    }

    /// Retrieve the version of an application in this OS.
    /// Note: this method makes use of the Cw2 query and may not coincide with the version of the
    /// module listed in VersionControl.
    pub fn module_version(&self, module_id: ModuleId) -> StdResult<ContractVersion> {
        let module_address = self.module_address(module_id)?;
        let req = QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: module_address.into(),
            key: CONTRACT.as_slice().into(),
        });
        self.deps.querier.query::<ContractVersion>(&req)
    }

    /// Construct an api request message.
    pub fn api_request<M: Serialize>(
        &self,
        api_id: ModuleId,
        message: impl Into<api::ExecuteMsg<M, Empty>>,
    ) -> StdResult<CosmosMsg> {
        self.assert_module_dependency(api_id)?;
        let api_msg: api::ExecuteMsg<M, Empty> = message.into();
        let api_address = self.module_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// Construct an API configure message
    pub fn configure_api(
        &self,
        api_id: ModuleId,
        message: api::BaseExecuteMsg,
    ) -> StdResult<CosmosMsg> {
        let api_msg: api::ExecuteMsg<Empty, Empty> = message.into();
        let api_address = self.module_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// Construct an api request message.
    pub fn app_request<M: Serialize>(
        &self,
        app_id: ModuleId,
        message: impl Into<app::ExecuteMsg<M, Empty>>,
    ) -> StdResult<CosmosMsg> {
        self.assert_module_dependency(app_id)?;
        let app_msg: app::ExecuteMsg<M, Empty> = message.into();
        let app_address = self.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Construct an API configure message
    pub fn configure_app(
        &self,
        app_id: ModuleId,
        message: app::BaseExecuteMsg,
    ) -> StdResult<CosmosMsg> {
        let app_msg: app::ExecuteMsg<Empty, Empty> = message.into();
        let app_address = self.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    fn assert_module_dependency(&self, module_id: ModuleId) -> StdResult<()> {
        let is_dependency = Dependencies::dependencies(self.base)
            .iter()
            .map(|d| d.id)
            .any(|x| x == module_id);

        match is_dependency {
            true => Ok(()),
            false => Err(StdError::generic_err(format!(
                "Module {} is not a dependency of this contract.",
                module_id
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cw_storage_plus::Map;
    use os::objects::dependency::StaticDependency;
    use std::collections::HashMap;
    use std::marker::PhantomData;

    use crate::apis::test_common::*;

    const TEST_MODULE_ID: ModuleId = "test_module";

    const TEST_MODULE_DEP: StaticDependency = StaticDependency::new(TEST_MODULE_ID, &[">1.0.0"]);

    impl Dependencies for MockModule {
        fn dependencies(&self) -> &[StaticDependency] {
            &[TEST_MODULE_DEP]
        }
    }

    /// mock querier that has the os modules loaded
    fn mock_querier_with_existing_module() -> MockQuerier {
        let mut q = MockQuerier::default();

        q.update_wasm(|wasm| match wasm {
            WasmQuery::Raw { contract_addr, key } => {
                let os_mod_key = "os_modules";
                let string_key = String::from_utf8(key.to_vec()).unwrap();
                let str_key = string_key.as_str();

                let mut modules = HashMap::<Binary, Binary>::default();
                modules.insert(b"os_modulestest_module".into(), b"the value".into());

                let test_binary = to_binary("os_modulestest_module".as_bytes()).unwrap();

                let res = match contract_addr.as_str() {
                    TEST_PROXY => match str_key {
                        "admin" => Ok(to_binary(&TEST_MANAGER).unwrap()),
                        _ => Err("unexpected key"),
                    },
                    TEST_MANAGER => {
                        if let Some(value) = modules.get(key) {
                            Ok(to_binary(&value.clone()).unwrap())
                        } else {
                            let b: Binary = b"\ros_modulestest_module".into();
                            panic!("{}, {}, {}, {}", contract_addr, key, b, test_binary);
                        }
                    }
                    _ => Err("unexpected contract"),
                }
                .into();

                //
                // let res: ContractResult<Binary> = match string_key.as_str() {
                //     "os_taco" => Ok(to_binary(&Some(Addr::unchecked("existing_address"))).unwrap()),
                //     _ => Err(format!(
                //         "Unexpected query: {} {}",
                //         contract_addr, string_key
                //     )),
                // }
                // .into();

                SystemResult::Ok(res)
            }
            _ => panic!("Unexpected smart query"),
        });

        q
    }

    pub fn mock_dependencies_with_existing_module(
    ) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: mock_querier_with_existing_module(),
            custom_query_type: PhantomData,
        }
    }

    mod assert_module_dependency {
        use super::*;

        #[test]
        fn should_return_ok_if_dependency() {
            let deps = mock_dependencies();
            let app = mock_module();

            let mods = app.modules(deps.as_ref());

            let res = mods.assert_module_dependency(TEST_MODULE_ID);
            assert_that!(res).is_ok();
        }

        #[test]
        fn should_return_err_if_not_dependency() {
            let deps = mock_dependencies();
            let app = mock_module();

            let mods = app.modules(deps.as_ref());

            let fake_module = "lol_no_chance";
            let res = mods.assert_module_dependency(fake_module);

            assert_that!(res).is_err().matches(|e| {
                e.to_string()
                    .contains(&format!("{} is not a dependency", fake_module))
            });
        }
    }

    /// Helper to check that the method is not callable when the module is not a dependency
    fn fail_when_not_dependency_test(
        modules_fn: impl FnOnce(&MockModule, Deps) -> StdResult<CosmosMsg>,
    ) {
        let deps = mock_dependencies_with_existing_module();
        let app = mock_module();

        let mods = app.modules(deps.as_ref());

        let fake_module = "lol_no_chance";
        let res = modules_fn(&app, deps.as_ref());

        print!("res: {:?}", res);

        assert_that!(res)
            .is_err()
            .matches(|e| e.to_string().contains(fake_module));
    }

    mod api_request {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(|app, deps| {
                let mods = app.modules(deps);
                mods.api_request(TEST_MODULE_ID, MockModuleExecuteMsg {})
            });
        }
    }

    mod app_request {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(|app, deps| {
                let mods = app.modules(deps);
                mods.app_request(TEST_MODULE_ID, MockModuleExecuteMsg {})
            });
        }
    }

    mod configure_api {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(|app, deps| {
                let mods = app.modules(deps);
                mods.configure_api(TEST_MODULE_ID, api::BaseExecuteMsg::Remove {})
            });
        }
    }

    mod configure_app {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(|app, deps| {
                let mods = app.modules(deps);
                mods.configure_app(
                    TEST_MODULE_ID,
                    app::BaseExecuteMsg::UpdateConfig {
                        ans_host_address: None,
                    },
                )
            });
        }
    }
}
