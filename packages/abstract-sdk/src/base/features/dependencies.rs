use crate::base::{Handler, contract_base::Dependency};

pub trait Dependencies: Sized {
    fn dependencies<'a>(&self) -> &[Dependency<'a>];
}

impl<T: Handler> Dependencies for T {
    fn dependencies<'a>(&self) -> &[Dependency<'a>] {
        Handler::dependencies(self)
    }
}
