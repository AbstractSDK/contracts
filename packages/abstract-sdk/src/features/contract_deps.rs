use cosmwasm_std::{Api, Deps, QuerierWrapper};

pub trait ContractDeps: Sized {
    fn deps(&self) -> &Deps;
    fn querier(&self) -> &QuerierWrapper {
        &self.deps().querier
    }
    fn api(&self) -> &dyn Api {
        self.deps().api
    }
}
