use abstract_os::objects::ans_host::AnsHost;
use cosmwasm_std::{Deps, StdResult};

use crate::ans_resolve::Resolve;

/// Trait that enables APIs that depend on the Abstract Name Service.
pub trait AbstractNameServiceProvider: Sized {
    fn ans_host(&self, deps: Deps) -> StdResult<AnsHost>;

    fn name_service<'a>(&'a self, deps: Deps<'a>) -> AbstractNameService<Self> {
        AbstractNameService {
            base: self,
            deps,
            host: self.ans_host(deps).unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct AbstractNameService<'a, T: AbstractNameServiceProvider> {
    base: &'a T,
    deps: Deps<'a>,
    pub host: AnsHost,
}

impl<'a, T: AbstractNameServiceProvider> AbstractNameService<'a, T> {
    pub fn query<R: Resolve>(&self, entry: &R) -> StdResult<R::Output> {
        entry.resolve(&self.deps.querier, &self.host)
    }
    pub fn host(&self) -> &AnsHost {
        &self.host
    }
}