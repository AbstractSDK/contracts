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

/// Trait for adding `abstract` attributes to a [`cosmwasm_std::Response`]
pub trait AbstractAttributes {
    fn add_abstract_attributes(
        self,
        attrs: impl IntoIterator<Item = impl Into<cosmwasm_std::Attribute>>,
    ) -> Self;
}

impl AbstractAttributes for cosmwasm_std::Response {
    fn add_abstract_attributes(
        mut self,
        attrs: impl IntoIterator<Item = impl Into<cosmwasm_std::Attribute>>,
    ) -> Self {
        // Find the index of the first abstract event in the events vector
        let index = self.events.iter().position(|e| e.ty == "abstract");

        if let Some(index) = index {
            // If an abstract event exists, replace it with a new event that has the additional attributes
            let event = self.events.remove(index);
            let new_event = event.add_attributes(attrs);
            self.events.insert(index, new_event);
        } else {
            // If an abstract event does not exist, create a new one with the additional attributes
            let new_event = cosmwasm_std::Event::new("abstract").add_attributes(attrs);
            self.events.push(new_event);
        }

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::{app, app::BaseQueryMsg};
    use cosmwasm_std::Empty;

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
}
