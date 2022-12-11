use abstract_os::{
    manager::state::{DEPENDENTS, OS_MODULES},
    objects::{
        dependency::Dependency,
        module_version::{query_module_data, MODULE},
    },
};
use abstract_sdk::base::features::Dependencies;
use cosmwasm_std::{Deps, DepsMut, Querier, StdError, StdResult, Storage};
use cw2::query_contract_info;
use semver::{Comparator, Version};

/// Assert the dependencies that this app relies on are installed.
pub fn assert_install_requirements(deps: Deps, module_id: &str) -> StdResult<Vec<Dependency>> {
    let querier = &deps.querier;
    let module_addr = OS_MODULES.load(deps.storage, module_id)?;
    let module_data = MODULE.query(&querier, module_addr)?;
    for dep in &module_data.dependencies {
        let maybe_dep_addr = OS_MODULES.load(deps.storage, &dep.id);
        if maybe_dep_addr.is_err() {
            return Err(StdError::generic_err(format!(
                "Module {} not enabled on OS.",
                dep.id
            )));
        };
        let dep_version = cw2::CONTRACT.query(&querier, maybe_dep_addr?)?;
        let version: Version = dep_version.version.parse().unwrap();
        // assert requirements
        assert_comparators(&dep.version_req, version, &dep.id)?;
    }
    Ok(module_data.dependencies)
}

/// Assert the dependencies that this app relies on are installed.
pub fn assert_migrate_requirements(
    deps: Deps,
    module_id: &str,
    new_version: Version,
) -> StdResult<Vec<Dependency>> {
    // load all the modules that depend on this module
    let dependents = DEPENDENTS.load(deps.storage, module_id)?;

    // for each module that depends on this module, check if it supports the new version.
    for dependent_module in dependents {
        let dependent_address = OS_MODULES.load(deps.storage, &dependent_module)?;
        let module_data = MODULE.query(&deps.querier, dependent_address)?;
        // filter the dependencies and assert version comparison when applicable
        let applicable_bounds = module_data
            .dependencies
            .iter()
            .filter(|dep| dep.id == module_id);
        // extra check for bound validity
        if applicable_bounds.count() == 0 {
            return Err(StdError::generic_err(format!("Listed dependency {} does not depend on {}.",dependent_module, module_id)));
        } 
        // assert bounds
        applicable_bounds.try_for_each(|dep| assert_comparators(&dep.version_req, new_version, &module_id));
    }
    // TODO:add check for dependencies
    Ok(())
}

/// Add module as dependent on its dependencies.
pub fn set_dependencies(
    store: &mut dyn Storage,
    module_id: String,
    dependencies: Vec<Dependency>,
) -> StdResult<()> {
    for dep in dependencies {
        DEPENDENTS.update(store, &dep.id, |mut dependents| {
            let mut dependents = dependents.unwrap_or_default();
            dependents.push(module_id);
            Ok::<_, StdError>(dependents)
        });
    }
    Ok(())
}

fn assert_comparators(bounds: &[Comparator], version: Version, module_id: &str) -> StdResult<()> {
    // assert requirements
    bounds.iter().try_for_each(|comp| {
        if comp.matches(&version) {
            Ok(())
        } else {
            Err(StdError::generic_err(format!(
                "Module {} with version {} does not fit requirement {}.",
                module_id, version, comp
            )))
        }
    })?;
}
