use os::manager::state::ModuleId;

use crate::base::Handler;

pub trait Dependencies: Sized {
    fn dependencies(&self) -> &[ModuleId];
}

impl<T: Handler> Dependencies for T {
    fn dependencies(&self) -> &[ModuleId] {
        Handler::dependencies(self)
    }
}
