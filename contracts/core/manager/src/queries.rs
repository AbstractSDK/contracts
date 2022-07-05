use abstract_os::manager::state::{OsInfo, INFO, OS_MODULES};
use abstract_os::manager::{
    ManagerModuleInfo, QueryInfoResponse, QueryModuleAddressesResponse, QueryModuleInfosResponse,
    QueryModuleVersionsResponse,
};
use abstract_sdk::manager::{query_module_addresses, query_module_version, query_module_versions};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

const DEFAULT_LIMIT: u8 = 5;
const MAX_LIMIT: u8 = 10;

pub fn handle_module_address_query(deps: Deps, env: Env, names: Vec<String>) -> StdResult<Binary> {
    let contracts = query_module_addresses(deps, &env.contract.address, &names)?;
    let vector = contracts
        .into_iter()
        .map(|(v, k)| (v, k.to_string()))
        .collect();
    to_binary(&QueryModuleAddressesResponse { modules: vector })
}

pub fn handle_contract_versions_query(
    deps: Deps,
    env: Env,
    names: Vec<String>,
) -> StdResult<Binary> {
    let response = query_module_versions(deps, &env.contract.address, &names)?;
    let versions = response.into_values().collect();
    to_binary(&QueryModuleVersionsResponse { versions })
}

pub fn handle_os_info_query(deps: Deps) -> StdResult<Binary> {
    let info: OsInfo = INFO.load(deps.storage)?;
    to_binary(&QueryInfoResponse { info })
}

pub fn handle_module_info_query(
    deps: Deps,
    last_module_name: Option<String>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_module_name.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(String, Addr)>, _> = OS_MODULES
        .range(deps.storage, start_bound, None, Order::Descending)
        .take(limit)
        .collect();

    let names_and_addr = res?;
    let mut resp_vec: Vec<ManagerModuleInfo> = vec![];
    for (name, address) in names_and_addr.into_iter() {
        let version = query_module_version(&deps, address.clone())?;
        resp_vec.push(ManagerModuleInfo {
            name,
            version,
            address: address.to_string(),
        })
    }

    to_binary(&QueryModuleInfosResponse {
        module_infos: resp_vec,
    })
}
