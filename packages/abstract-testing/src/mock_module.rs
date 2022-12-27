use std::collections::HashMap;

use abstract_os::objects::ans_host::AnsHost;
use abstract_os::{api, app};
use abstract_sdk::base::features::{AbstractNameService, Identification};
use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    to_binary, Addr, Binary, ContractResult, Deps, Empty, QuerierWrapper, StdError, StdResult,
    SystemResult, WasmQuery,
};

use crate::{
    TEST_ANS_HOST, TEST_MANAGER, TEST_MODULE_ADDRESS, TEST_MODULE_ID, TEST_OS_ID, TEST_PROXY,
};

pub struct MockModule {}

impl MockModule {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn querier() -> MockQuerier {
        let mut querier = MockQuerier::default();
        querier.update_wasm(|wasm| {
            match wasm {
                WasmQuery::Raw { contract_addr, key } => {
                    let str_key = std::str::from_utf8(&key.0).unwrap();

                    let res = match contract_addr.as_str() {
                        TEST_PROXY => match str_key {
                            "admin" => Ok(to_binary(&TEST_MANAGER).unwrap()),
                            _ => Err("unexpected key".to_string()),
                        },
                        TEST_MANAGER => {
                            // add module
                            let map_key = map_key("os_modules", TEST_MODULE_ID);
                            let mut modules = HashMap::<Binary, Addr>::default();
                            modules.insert(
                                Binary(map_key.as_bytes().to_vec()),
                                Addr::unchecked(TEST_MODULE_ADDRESS),
                            );

                            if let Some(value) = modules.get(key) {
                                Ok(to_binary(&value.to_owned()).unwrap())
                            } else if str_key == "\u{0}{5}os_id" {
                                Ok(to_binary(&TEST_OS_ID).unwrap())
                            } else {
                                // return none
                                Ok(Binary::default())
                            }
                        }
                        // TODO: VERSION CONTROL
                        // TEST_VERSION_CONTROL => match key {
                        //     bin => Ok(to_binary(&1).unwrap()),
                        //     _ => Err("unexpected key".into()),
                        // },
                        _ => Err("unexpected contract".into()),
                    };

                    match res {
                        Ok(res) => SystemResult::Ok(ContractResult::Ok(res)),
                        Err(e) => SystemResult::Ok(ContractResult::Err(e)),
                    }
                }
                _ => panic!("Unexpected smart query"),
            }
        });
        querier
    }

    pub fn wrap_querier(querier: &MockQuerier) -> QuerierWrapper<'_, Empty> {
        QuerierWrapper::<Empty>::new(querier)
    }
}

#[cfg(test)]
mod tests {
    use crate::TEST_MODULE_ID;

    use super::*;
    use abstract_os::manager::state::OS_MODULES;
    use abstract_os::proxy::state::OS_ID;
    use cosmwasm_std::testing::mock_dependencies;
    use speculoos::prelude::*;

    #[test]
    fn should_return_test_os_id_with_test_manager() {
        let mut deps = mock_dependencies();
        deps.querier = MockModule::querier();
        let actual = OS_ID.query(
            &MockModule::wrap_querier(&deps.querier),
            Addr::unchecked(TEST_MANAGER),
        );

        assert_that!(actual).is_ok().is_equal_to(TEST_OS_ID);
    }

    mod querying_os_modules {
        use super::*;

        #[test]
        fn should_return_test_module_address_for_test_module() {
            let mut deps = mock_dependencies();
            deps.querier = MockModule::querier();

            let actual = OS_MODULES.query(
                &MockModule::wrap_querier(&deps.querier),
                Addr::unchecked(TEST_MANAGER),
                TEST_MODULE_ID,
            );

            assert_that!(actual)
                .is_ok()
                .is_some()
                .is_equal_to(Addr::unchecked(TEST_MODULE_ADDRESS));
        }

        #[test]
        fn should_return_none_for_unknown_module() {
            let mut deps = mock_dependencies();
            deps.querier = MockModule::querier();

            let actual = OS_MODULES.query(
                &MockModule::wrap_querier(&deps.querier),
                Addr::unchecked(TEST_MANAGER),
                "unknown_module",
            );

            assert_that!(actual).is_ok().is_none();
        }
    }

    mod querying_proxy {
        // use super::*;
        // TODO: doesn't invalid type response
        // use abstract_os::objects::common_namespace::ADMIN_NAMESPACE;
        // use abstract_os::proxy::state::ADMIN;
        // use cosmwasm_std::{to_vec, Querier};
        // use cw_controllers::AdminResponse;
        //
        // #[test]
        // fn admin_should_return_test_manager() {
        //     let mut deps = mock_dependencies();
        //     deps.querier = MockModule::querier();
        //
        //     let admin_query = QueryRequest::Wasm(WasmQuery::Raw {
        //         contract_addr: Addr::unchecked(TEST_PROXY).into(),
        //         key: ADMIN_NAMESPACE.as_bytes().into(),
        //     });
        //
        //     let actual =
        //         MockModule::wrap_querier(&deps.querier).query::<AdminResponse>(&admin_query);
        //
        //     assert_that!(actual)
        //         .is_ok()
        //         .map(|a| &a.admin)
        //         .is_some()
        //         .is_equal_to(&TEST_MANAGER.to_string());
        // }
    }
}

impl Identification for MockModule {
    fn proxy_address(&self, _deps: Deps) -> Result<Addr, StdError> {
        Ok(Addr::unchecked(TEST_PROXY))
    }
}

impl AbstractNameService for MockModule {
    fn ans_host(&self, _deps: Deps) -> StdResult<AnsHost> {
        Ok(AnsHost {
            address: Addr::unchecked(TEST_ANS_HOST),
        })
    }
}

#[cosmwasm_schema::cw_serde]
pub struct MockModuleExecuteMsg {}

#[cosmwasm_schema::cw_serde]
pub struct MockModuleQueryMsg {}

impl api::ApiExecuteMsg for MockModuleExecuteMsg {}

impl api::ApiQueryMsg for MockModuleQueryMsg {}

impl app::AppExecuteMsg for MockModuleExecuteMsg {}

impl app::AppQueryMsg for MockModuleQueryMsg {}

fn map_key<'a>(namespace: &'a str, key: &'a str) -> String {
    let line_feed_char = b"\x0a";
    let mut res = vec![0u8];
    res.extend_from_slice(line_feed_char);
    res.extend_from_slice(namespace.as_bytes());
    res.extend_from_slice(key.as_bytes());
    std::str::from_utf8(&res).unwrap().to_string()
}
