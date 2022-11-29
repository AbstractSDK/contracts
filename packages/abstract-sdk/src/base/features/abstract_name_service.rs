use abstract_os::objects::ans_host::AnsHost;
use cosmwasm_std::{Deps, StdResult};

use crate::ans_resolve::Resolve;

/// Trait that enables APIs that depend on the Abstract Name Service.
pub trait AbstractNameService: Sized {
    fn name_service(&self, deps: Deps) -> StdResult<AnsHost>;

    fn query<R: Resolve>(&self, deps: Deps, entry: &R) -> StdResult<R::Output> {
        entry.resolve(&deps.querier, &self.name_service(deps)?)
    }
}
