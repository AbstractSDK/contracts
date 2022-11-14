use cosmwasm_std::{Addr, StdResult};

use super::contract_deps::ContractDeps;

pub trait Versioning: ContractDeps {
    fn version_registry(&self) -> StdResult<Addr>;
}

// / Query module information
// pub fn get_module(
//     querier: &QuerierWrapper,
//     module_info: ModuleInfo,
//     version_control_addr: &Addr,
// ) -> StdResult<Module> {
//     let resp: ModuleResponse = querier.query_wasm_smart(
//         version_control_addr,
//         &QueryMsg::Module {
//             module: module_info,
//         },
//     )?;
//     Ok(resp.module)
// }
