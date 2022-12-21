use cosmwasm_std::{DepsMut, Empty, MessageInfo, Response};

use crate::contract::{VCResult, ABSTRACT_NAMESPACE};
use crate::error::VCError;
use abstract_sdk::os::{
    objects::{module::ModuleInfo, module_reference::ModuleReference},
    version_control::{state::*, Core},
};

/// Add new OS to version control contract
/// Only Factory can add OS
pub fn add_os(deps: DepsMut, msg_info: MessageInfo, os_id: u32, core: Core) -> VCResult {
    // Only Factory can add new OS
    FACTORY.assert_admin(deps.as_ref(), &msg_info.sender)?;
    OS_ADDRESSES.save(deps.storage, os_id, &core)?;

    Ok(Response::new().add_attributes(vec![
        ("Action", "Add OS"),
        ("ID:", &os_id.to_string()),
        ("Manager:", core.manager.as_ref()),
        ("Proxy", core.proxy.as_ref()),
    ]))
}

/// Here we can add logic to allow subscribers to claim a namespace and upload contracts to that namespace
pub fn add_modules(
    deps: DepsMut,
    msg_info: MessageInfo,
    modules: Vec<(ModuleInfo, ModuleReference)>,
) -> VCResult {
    for (module, mod_ref) in modules {
        if MODULE_LIBRARY.has(deps.storage, module.clone()) {
            return Err(VCError::ModuleUpdate(module));
        }
        // version must be set in order to add the new version
        module.assert_version_variant()?;
        if module.provider == ABSTRACT_NAMESPACE {
            // Only Admin can update abstract contracts
            ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
        }
        MODULE_LIBRARY.save(deps.storage, module, &mod_ref)?;
    }

    Ok(Response::new().add_attributes(vec![("action", "add module")]))
}

/// Remove a module
pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module: ModuleInfo) -> VCResult {
    // Only Admin can update code-ids
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    module.assert_version_variant()?;
    if MODULE_LIBRARY.has(deps.storage, module.clone()) {
        MODULE_LIBRARY.remove(deps.storage, module.clone());
    } else {
        return Err(VCError::MissingModule(module));
    }

    Ok(Response::new().add_attributes(vec![
        ("action", "remove module"),
        ("module:", &module.to_string()),
    ]))
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> VCResult {
    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    // Admin is asserted here
    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{Addr, Order, OwnedDeps, StdError, Storage};

    use abstract_os::version_control::*;

    use crate::contract;
    use speculoos::prelude::*;

    use super::*;

    type VersionControlTestResult = Result<(), VCError>;

    const TEST_OS_FACTORY: &str = "os_factory";
    const TEST_ADMIN: &str = "testadmin";
    const TEST_MODULE: &str = "provider:test";
    const TEST_VERSION: &str = "verren";

    const TEST_VERSION_CONTROL: &str = "version_control";

    const TEST_PROXY_ADDR: &str = "proxy";

    /// Initialize the version_control with admin as creator and factory
    fn mock_init(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})
    }

    /// Initialize the version_control with admin and updated os_factory
    fn mock_init_with_factory(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})?;
        execute_as_admin(
            deps,
            ExecuteMsg::SetFactory {
                new_factory: TEST_OS_FACTORY.to_string(),
            },
        )
    }

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> VCResult {
        contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> VCResult {
        execute_as(deps, TEST_ADMIN, msg)
    }

    fn execute_as_factory(deps: DepsMut, msg: ExecuteMsg) -> VCResult {
        execute_as(deps, TEST_OS_FACTORY, msg)
    }

    fn test_only_admin(msg: ExecuteMsg) -> VersionControlTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let res = execute_as(deps.as_mut(), "not_admin", msg);
        assert_that(&res)
            .is_err()
            .is_equal_to(VCError::Admin(AdminError::NotAdmin {}));

        Ok(())
    }
    use cw_controllers::AdminError;

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    mod set_admin_and_factory {
        use super::*;

        #[test]
        fn only_admin_admin() -> VersionControlTestResult {
            let msg = ExecuteMsg::SetAdmin {
                new_admin: "new_admin".to_string(),
            };
            test_only_admin(msg)
        }

        #[test]
        fn only_admin_factory() -> VersionControlTestResult {
            let msg = ExecuteMsg::SetFactory {
                new_factory: "new_factory".to_string(),
            };
            test_only_admin(msg)
        }

        #[test]
        fn updates_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_admin = "new_admin";
            let msg = ExecuteMsg::SetAdmin {
                new_admin: new_admin.to_string(),
            };

            let res = execute_as_admin(deps.as_mut(), msg);
            assert_that(&res).is_ok();

            let actual_admin = ADMIN.get(deps.as_ref())?.unwrap();

            assert_that(&actual_admin).is_equal_to(Addr::unchecked(new_admin));

            Ok(())
        }

        #[test]
        fn updates_factory() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_factory = "new_factory";
            let msg = ExecuteMsg::SetFactory {
                new_factory: new_factory.to_string(),
            };

            let res = execute_as_admin(deps.as_mut(), msg);
            assert_that(&res).is_ok();

            let actual_factory = FACTORY.get(deps.as_ref())?.unwrap();

            assert_that(&actual_factory).is_equal_to(Addr::unchecked(new_factory));
            Ok(())
        }
    }

    mod update_modules {
        use super::*;
        use abstract_os::objects::{module_reference::ModuleReference,module::*};
        fn test_module() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Version(TEST_VERSION.into())).unwrap()
        }

        fn test_module_latest() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Version(TEST_VERSION.into())).unwrap()

        }

        // - bad version nr
        // - try add a "latest"
        // - Query latest
        // - add under Abstract namespace

        #[test]
        fn add_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let new_module = test_module();
            let msg = ExecuteMsg::AddModules {
                modules: vec![(new_module.clone(),ModuleReference::App(0))]
            };
            let res = execute_as(deps.as_mut(), "test_sender", msg);
            assert_that(&res).is_ok();
            let module = MODULE_LIBRARY.load(&deps.storage, new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn bad_version() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let bad_version_module = ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Version("non_compliant_version".into()))?;
            let msg = ExecuteMsg::AddModules {
                modules: vec![(bad_version_module.clone(),ModuleReference::App(0))]
            };
            let latest_version_module = ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Latest)?;
            let msg = ExecuteMsg::AddModules {
                modules: vec![(bad_version_module.clone(),ModuleReference::App(0))]
            };
            let res = execute_as(deps.as_mut(), "test_sender", msg);
            assert_that(&res).is_ok();
            let module = MODULE_LIBRARY.load(&deps.storage, new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }
    }

    mod register_os {
        use super::*;
    }
}
