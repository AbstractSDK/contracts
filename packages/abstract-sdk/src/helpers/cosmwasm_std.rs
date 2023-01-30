use cosmwasm_std::{to_binary, QueryRequest, StdResult, WasmQuery};
use serde::Serialize;

/// Shortcut helper as the construction of QueryRequest::Wasm(WasmQuery::Smart {...}) can be quite verbose in contract code
pub fn wasm_smart_query<C>(
    contract_addr: impl Into<String>,
    msg: &impl Serialize,
) -> StdResult<QueryRequest<C>> {
    let query_msg = to_binary(msg)?;
    Ok(QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.into(),
        msg: query_msg,
    }))
}

/// Shortcut helper for using [`from_binary`] on serializeable data that unwraps it.
#[macro_export]
macro_rules! unwrap_binary {
    (&$subject:tt) => {
        unwrap_binary!($subject)
    };
    ($subject:tt) => {{
        cosmwasm_std::from_binary(&$subject).unwrap()
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::Empty;
    use os::app;
    use os::app::BaseQueryMsg;

    #[test]
    fn test_wasm_smart_query() {
        let query_msg = app::QueryMsg::<Empty>::Base(BaseQueryMsg::Admin {});
        let query = wasm_smart_query::<Empty>("contract", &query_msg).unwrap();
        match query {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                assert_eq!(contract_addr, "contract");
                assert_eq!(msg, to_binary(&query_msg).unwrap());
            }
            _ => panic!("Unexpected query"),
        }
    }

    mod from_binary {
        use super::*;

        #[test]
        fn test_from_binary() {
            let binary = to_binary(&BaseQueryMsg::Admin {}).unwrap();
            let query_msg: BaseQueryMsg = unwrap_binary!(binary);
            assert_eq!(query_msg, BaseQueryMsg::Admin {});
        }

        #[test]
        fn test_from_binary_ref() {
            let binary = to_binary(&BaseQueryMsg::Admin {}).unwrap();
            let query_msg: BaseQueryMsg = unwrap_binary!(&binary);
            assert_eq!(query_msg, BaseQueryMsg::Admin {});
        }
    }
}
