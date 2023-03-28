use crate::contract::{VCResult, ABSTRACT_NAMESPACE};
use crate::error::VCError;
use abstract_core::objects::AccountId;
use abstract_macros::abstract_response;
use abstract_sdk::core::{
    objects::{module::ModuleInfo, module_reference::ModuleReference},
    version_control::{state::*, AccountBase},
    VERSION_CONTROL,
};
use cosmwasm_std::{DepsMut, Empty, MessageInfo};

#[abstract_response(VERSION_CONTROL)]
pub struct VcResponse;

/// Add new Account to version control contract
/// Only Factory can add Account
pub fn add_account(
    deps: DepsMut,
    msg_info: MessageInfo,
    account_id: AccountId,
    account_base: AccountBase,
) -> VCResult {
    // Only Factory can add new Account
    FACTORY.assert_admin(deps.as_ref(), &msg_info.sender)?;
    ACCOUNT_ADDRESSES.save(deps.storage, account_id, &account_base)?;

    Ok(VcResponse::new(
        "add_os",
        vec![
            ("account_id", account_id.to_string().as_str()),
            ("manager", account_base.manager.as_ref()),
            ("proxy", account_base.proxy.as_ref()),
        ],
    ))
}

/// Here we can add logic to allow subscribers to claim a namespace and upload contracts to that namespace
pub fn add_modules(
    deps: DepsMut,
    msg_info: MessageInfo,
    modules: Vec<(ModuleInfo, ModuleReference)>,
) -> VCResult {
    for (module, mod_ref) in modules {
        if MODULE_LIBRARY.has(deps.storage, &module) || YANKED_MODULES.has(deps.storage, &module) {
            return Err(VCError::NotUpdateableModule(module));
        }
        module.validate()?;
        mod_ref.validate(deps.as_ref())?;
        // version must be set in order to add the new version
        module.assert_version_variant()?;

        if module.provider == ABSTRACT_NAMESPACE {
            // Only Admin can update abstract contracts
            ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
        }
        MODULE_LIBRARY.save(deps.storage, &module, &mod_ref)?;
    }

    Ok(VcResponse::action("add_modules"))
}

/// Remove a module
pub fn remove_module(
    deps: DepsMut,
    msg_info: MessageInfo,
    module: ModuleInfo,
    yank: bool,
) -> VCResult {
    // Only Admin can update code-ids
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    module.assert_version_variant()?;
    if let Some(mod_ref) = MODULE_LIBRARY.may_load(deps.storage, &module)? {
        if yank {
            YANKED_MODULES.save(deps.storage, &module, &mod_ref)?;
        }
        MODULE_LIBRARY.remove(deps.storage, &module);
    } else {
        return Err(VCError::ModuleNotFound(module));
    }

    Ok(VcResponse::new(
        "remove_module",
        vec![
            ("module", &module.to_string()),
            ("yank", &(if yank { "yes" } else { "no" }).to_string()),
        ],
    ))
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> VCResult {
    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    // Admin is asserted here
    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
    Ok(VcResponse::new(
        "set_admin",
        vec![
            ("previous_admin", previous_admin.to_string()),
            ("admin", admin),
        ],
    ))
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;

    use abstract_core::version_control::*;

    use crate::contract;
    use speculoos::prelude::*;

    use super::*;
    use abstract_testing::prelude::{TEST_ACCOUNT_FACTORY, TEST_ADMIN, TEST_VERSION};

    type VersionControlTestResult = Result<(), VCError>;

    const TEST_OTHER: &str = "test-other";
    const TEST_MODULE: &str = "provider:test";

    const TEST_PROXY_ADDR: &str = "proxy";
    const TEST_MANAGER_ADDR: &str = "manager";

    /// Initialize the version_control with admin as creator and factory
    fn mock_init(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})
    }

    /// Initialize the version_control with admin and updated account_factory
    fn mock_init_with_factory(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})?;
        execute_as_admin(
            deps,
            ExecuteMsg::SetFactory {
                new_factory: TEST_ACCOUNT_FACTORY.to_string(),
            },
        )
    }

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> VCResult {
        contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> VCResult {
        execute_as(deps, TEST_ADMIN, msg)
    }

    fn test_only_admin(msg: ExecuteMsg) -> VersionControlTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let res = execute_as(deps.as_mut(), "not_admin", msg);
        assert_that!(&res)
            .is_err()
            .is_equal_to(VCError::Admin(AdminError::NotAdmin {}));

        Ok(())
    }
    use cw_controllers::AdminError;

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
            assert_that!(&res).is_ok();

            let actual_admin = ADMIN.get(deps.as_ref())?.unwrap();

            assert_that!(&actual_admin).is_equal_to(Addr::unchecked(new_admin));

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
            assert_that!(&res).is_ok();

            let actual_factory = FACTORY.get(deps.as_ref())?.unwrap();

            assert_that!(&actual_factory).is_equal_to(Addr::unchecked(new_factory));
            Ok(())
        }
    }

    mod add_modules {
        use super::*;
        use abstract_core::objects::{module::*, module_reference::ModuleReference};
        use abstract_core::AbstractError;

        fn test_module() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Version(TEST_VERSION.into())).unwrap()
        }

        // - Query latest

        #[test]
        fn add_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let new_module = test_module();
            let msg = ExecuteMsg::AddModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res).is_ok();
            let module = MODULE_LIBRARY.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn remove_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let rm_module = test_module();

            // first add module
            let msg = ExecuteMsg::AddModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OTHER, msg)?;
            let module = MODULE_LIBRARY.load(&deps.storage, &rm_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));

            // then remove
            let msg = ExecuteMsg::RemoveModule {
                module: rm_module.clone(),
                yank: false,
            };
            // as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            execute_as_admin(deps.as_mut(), msg)?;

            let module = MODULE_LIBRARY.load(&deps.storage, &rm_module);
            assert_that!(&module).is_err();
            Ok(())
        }

        #[test]
        fn yank_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let rm_module = test_module();

            // first add module
            let msg = ExecuteMsg::AddModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OTHER, msg)?;
            let module = MODULE_LIBRARY.load(&deps.storage, &rm_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));

            // then remove
            let msg = ExecuteMsg::RemoveModule {
                module: rm_module.clone(),
                yank: true,
            };
            // as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            execute_as_admin(deps.as_mut(), msg)?;

            let module = MODULE_LIBRARY.load(&deps.storage, &rm_module);
            assert_that!(&module).is_err();
            let module = YANKED_MODULES.load(&deps.storage, &rm_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn bad_version() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let bad_version_module = ModuleInfo::from_id(
                TEST_MODULE,
                ModuleVersion::Version("non_compliant_version".into()),
            )?;
            let msg = ExecuteMsg::AddModules {
                modules: vec![(bad_version_module, ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .matches(|e| e.to_string().contains("Invalid version"));

            let latest_version_module = ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Latest)?;
            let msg = ExecuteMsg::AddModules {
                modules: vec![(latest_version_module, ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Abstract(AbstractError::Assert(
                    "Module version must be set to a specific version".into(),
                )));
            Ok(())
        }

        #[test]
        fn abstract_namespace() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            let abstract_contract_id = format!("{}:{}", ABSTRACT_NAMESPACE, "test-module");
            mock_init(deps.as_mut())?;
            let new_module = ModuleInfo::from_id(&abstract_contract_id, TEST_VERSION.into())?;
            let msg = ExecuteMsg::AddModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // execute as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            execute_as_admin(deps.as_mut(), msg)?;
            let module = MODULE_LIBRARY.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn validates_module_info() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let bad_modules = vec![
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    provider: "".to_string(),
                },
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    provider: "".to_string(),
                },
                ModuleInfo {
                    name: "".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    provider: "test".to_string(),
                },
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("aoeu".to_string()),
                    provider: "".to_string(),
                },
            ];

            for bad_module in bad_modules {
                let msg = ExecuteMsg::AddModules {
                    modules: vec![(bad_module.clone(), ModuleReference::App(0))],
                };
                let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
                assert_that!(&res)
                    .named(&format!("ModuleInfo validation failed for {bad_module}"))
                    .is_err()
                    .is_equal_to(&VCError::Abstract(AbstractError::FormattingError {
                        object: "module name".into(),
                        expected: "with content".into(),
                        actual: "empty".into(),
                    }));
            }

            Ok(())
        }
    }

    mod register_os {
        use super::*;

        #[test]
        fn add_os() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init_with_factory(deps.as_mut())?;

            let test_core: AccountBase = AccountBase {
                manager: Addr::unchecked(TEST_MANAGER_ADDR),
                proxy: Addr::unchecked(TEST_PROXY_ADDR),
            };
            let msg = ExecuteMsg::AddAccount {
                account_id: 0,
                account_base: test_core.clone(),
            };

            // as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            // as admin
            let res = execute_as_admin(deps.as_mut(), msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            // as factory
            execute_as(deps.as_mut(), TEST_ACCOUNT_FACTORY, msg)?;

            let account = ACCOUNT_ADDRESSES.load(&deps.storage, 0)?;
            assert_that!(&account).is_equal_to(&test_core);
            Ok(())
        }
    }

    mod configure {

        use super::*;

        #[test]
        fn set_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::SetAdmin {
                new_admin: TEST_OTHER.into(),
            };

            // as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            execute_as_admin(deps.as_mut(), msg)?;
            let new_admin = ADMIN.query_admin(deps.as_ref())?.admin;
            assert_that!(new_admin).is_equal_to(&Some(TEST_OTHER.into()));
            Ok(())
        }

        #[test]
        fn set_factory() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::SetFactory {
                new_factory: TEST_ACCOUNT_FACTORY.into(),
            };

            // as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            execute_as_admin(deps.as_mut(), msg)?;
            let new_factory = FACTORY.query_admin(deps.as_ref())?.admin;
            assert_that!(new_factory).is_equal_to(&Some(TEST_ACCOUNT_FACTORY.into()));
            Ok(())
        }
    }
}
