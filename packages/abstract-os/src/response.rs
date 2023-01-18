use cosmwasm_std::{Attribute, Event, Response};

pub struct AbstractResponse;

impl AbstractResponse {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<A: Into<Attribute>>(
        contract: impl Into<String>,
        action: impl Into<String>,
        attrs: impl IntoIterator<Item = A>,
    ) -> Response {
        Response::new().add_event(
            Event::new("abstract")
                .add_attributes(vec![("contract", contract)])
                .add_attributes(vec![("action", action)])
                .add_attributes(attrs),
        )
    }
}
