use crate::apis::ModuleIdentification;
use abstract_macros::with_abstract_event;
use cosmwasm_std::{Attribute, Response};

pub trait AbstractResponse: ModuleIdentification {
    /// Respond with an abstract-specific event that contains the contract name and the action.
    fn response(&self, action: impl Into<String>) -> Response {
        self.response_with(action, Vec::<Attribute>::new())
    }

    fn response_with(
        &self,
        action: impl Into<String>,
        attributes: impl IntoIterator<Item = impl Into<Attribute>>,
    ) -> Response {
        let response = Response::default();
        let module_id = self.module_id();
        with_abstract_event!(response, module_id, action, attributes)
    }
}

impl<T> AbstractResponse for T where T: ModuleIdentification {}
