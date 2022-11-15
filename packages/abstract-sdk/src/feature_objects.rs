use cosmwasm_std::{Addr, Deps};

pub use abstract_os::objects::ans_host::AnsHost;

#[derive(Clone)]
pub struct VersionControlContract {
    pub contract_address: Addr,
}

impl crate::base::features::RegisterAccess for VersionControlContract {
    fn registry(&self, _deps: Deps) -> cosmwasm_std::StdResult<Addr> {
        Ok(self.contract_address.clone())
    }
}

#[derive(Clone)]
pub struct ProxyContract {
    pub contract_address: Addr,
}

impl crate::base::features::Identification for ProxyContract {
    fn proxy_address(&self, _deps: Deps) -> cosmwasm_std::StdResult<Addr> {
        Ok(self.contract_address.clone())
    }
}

impl crate::base::features::AbstractNameSystem for AnsHost {
    fn ans_host(
        &self,
        _deps: Deps,
    ) -> cosmwasm_std::StdResult<abstract_os::objects::ans_host::AnsHost> {
        Ok(self.clone())
    }
}
