use abstract_os::objects::ans_host::AnsHost;
use cosmwasm_std::StdResult;

use super::contract_deps::ContractDeps;

/// Trait that enables API's that depend on the Abstract Name System.
pub trait AbstractNameSystem: ContractDeps {
    fn ans_host(&self) -> StdResult<AnsHost>;
}
