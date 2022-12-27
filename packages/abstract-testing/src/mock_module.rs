use std::collections::HashMap;

use abstract_os::objects::ans_host::AnsHost;
use abstract_os::{api, app};
use abstract_sdk::base::features::{AbstractNameService, Identification};
use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    to_binary, Addr, Binary, ContractResult, Deps, Empty, QuerierWrapper, StdError, StdResult,
    SystemResult, WasmQuery,
};

use crate::{TEST_ANS_HOST, TEST_MANAGER, TEST_MODULE_ADDRESS, TEST_MODULE_ID, TEST_PROXY};

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
                                Ok(to_binary(&1).unwrap())
                            } else {
                                Err(format!("unexpected key {}", str_key))
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
    use cosmwasm_std::{Deps, Empty, QuerierWrapper, QueryRequest};

    #[test]
    fn test_querier() {
        let mut deps = mock_dependencies();
        deps.querier = MockModule::querier();
        OS_ID
            .query(
                &MockModule::wrap_querier(&deps.querier),
                Addr::unchecked(TEST_MANAGER),
            )
            .unwrap();

        OS_MODULES
            .query(
                &MockModule::wrap_querier(&deps.querier),
                Addr::unchecked(TEST_MANAGER),
                TEST_MODULE_ID,
            )
            .unwrap();
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
