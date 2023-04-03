use crate::contract::{VCResult, ABSTRACT_NAMESPACE};
use crate::error::VCError;

use abstract_core::objects::module::ModuleVersion;
use abstract_macros::abstract_response;
use abstract_sdk::core::{
    manager::{ConfigResponse as ManagerConfigResponse, QueryMsg as ManagerQueryMsg},
    objects::{
        module::ModuleInfo, module_reference::ModuleReference, namespace::Namespace, AccountId,
    },
    version_control::{namespaces_info, state::*, AccountBase},
    VERSION_CONTROL,
};
use cosmwasm_std::{Addr, Deps, DepsMut, Empty, MessageInfo, Order, QuerierWrapper, StdResult};

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
        } else {
            // Only owner can add modules
            validate_account_owner(deps.as_ref(), &module.provider, &msg_info.sender)?;
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
    // Only Admin or owner can update code-ids
    if !ADMIN.is_admin(deps.as_ref(), &msg_info.sender)? {
        validate_account_owner(deps.as_ref(), &module.provider, &msg_info.sender)?;
    }

    module.assert_version_variant()?;
    let mod_ref = MODULE_LIBRARY
        .may_load(deps.storage, &module)?
        .ok_or_else(|| VCError::ModuleNotFound(module.clone()))?;
    if yank {
        YANKED_MODULES.save(deps.storage, &module, &mod_ref)?;
    }
    MODULE_LIBRARY.remove(deps.storage, &module);

    Ok(VcResponse::new(
        "remove_module",
        vec![
            ("module", &module.to_string()),
            ("yank", &(if yank { "yes" } else { "no" }).to_string()),
        ],
    ))
}

/// Claim namespaces
/// Only the Account Owner can do this
pub fn claim_namespaces(
    deps: DepsMut,
    msg_info: MessageInfo,
    account_id: AccountId,
    namespaces: Vec<String>,
) -> VCResult {
    // verify account owner
    let account_base = ACCOUNT_ADDRESSES.load(deps.storage, account_id)?;
    let account_owner = query_account_owner(&deps.querier, &account_base.manager)?;
    if msg_info.sender != account_owner {
        return Err(VCError::AccountOwnerMismatch {
            sender: msg_info.sender.into_string(),
            owner: account_owner,
        });
    }

    let limit = NAMESPACES_LIMIT.load(deps.storage)? as usize;
    let current = namespaces_info()
        .idx
        .account_id
        .prefix(account_id)
        .range(deps.storage, None, None, Order::Ascending)
        .count();
    if current + namespaces.len() > limit {
        return Err(VCError::ExceedsNamespaceLimit { limit, current });
    }

    for namespace in namespaces.iter() {
        let item = Namespace::from(namespace);
        item.validate()?;
        if let Some(id) = namespaces_info().may_load(deps.storage, &item)? {
            return Err(VCError::NamespaceOccupied {
                namespace: namespace.to_string(),
                id,
            });
        }
        namespaces_info().save(deps.storage, &item, &account_id)?;
    }

    Ok(VcResponse::new(
        "claim_namespaces",
        vec![
            ("account_id", &account_id.to_string()),
            ("namespaces", &namespaces.join(",")),
        ],
    ))
}

/// Remove namespaes
/// Only admin or the account owner can do this
pub fn remove_namespaces(
    deps: DepsMut,
    msg_info: MessageInfo,
    namespaces: Vec<String>,
) -> VCResult {
    let is_admin = ADMIN.is_admin(deps.as_ref(), &msg_info.sender)?;

    for namespace in namespaces.iter() {
        if !is_admin {
            validate_account_owner(deps.as_ref(), namespace, &msg_info.sender)?;
        }

        for ((name, version), mod_ref) in MODULE_LIBRARY
            .sub_prefix(namespace.to_owned())
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?
            .into_iter()
        {
            let module = ModuleInfo {
                provider: namespace.to_owned(),
                name,
                version: ModuleVersion::Version(version),
            };
            MODULE_LIBRARY.remove(deps.storage, &module);
            YANKED_MODULES.save(deps.storage, &module, &mod_ref)?;
        }

        namespaces_info().remove(deps.storage, &Namespace::from(namespace))?;
    }

    Ok(VcResponse::new(
        "remove_namespaces",
        vec![("namespaces", &namespaces.join(","))],
    ))
}

pub fn update_namespaces_limit(deps: DepsMut, info: MessageInfo, new_limit: u32) -> VCResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let previous_limit = NAMESPACES_LIMIT.load(deps.storage)?;
    NAMESPACES_LIMIT.save(deps.storage, &new_limit)?;

    Ok(VcResponse::new(
        "update_namespaces_limit",
        vec![
            ("previous_limit", previous_limit.to_string()),
            ("limit", new_limit.to_string()),
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

pub fn query_account_owner(querier: &QuerierWrapper, contract_addr: &Addr) -> StdResult<String> {
    let config_resp: ManagerConfigResponse =
        querier.query_wasm_smart(contract_addr, &ManagerQueryMsg::Config {})?;
    Ok(config_resp.owner)
}

pub fn validate_account_owner(deps: Deps, provider: &str, sender: &Addr) -> Result<(), VCError> {
    let sender = sender.clone();
    let namespace = Namespace::from(provider);
    let account_id = namespaces_info()
        .may_load(deps.storage, &namespace)?
        .ok_or_else(|| VCError::MissingNamespace {
            namespace: provider.to_string(),
        })?;
    let account_base = ACCOUNT_ADDRESSES.load(deps.storage, account_id)?;
    let account_owner = query_account_owner(&deps.querier, &account_base.manager)?;
    if sender != account_owner {
        return Err(VCError::AccountOwnerMismatch {
            sender: sender.into_string(),
            owner: account_owner,
        });
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use abstract_testing::MockQuerierBuilder;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, to_binary, Addr, Uint64};

    use abstract_core::version_control::*;

    use crate::contract;
    use speculoos::prelude::*;

    use super::*;
    use abstract_testing::prelude::{
        TEST_ACCOUNT_FACTORY, TEST_ACCOUNT_ID, TEST_ADMIN, TEST_MODULE_FACTORY, TEST_VERSION,
        TEST_VERSION_CONTROL,
    };

    type VersionControlTestResult = Result<(), VCError>;

    const TEST_OTHER: &str = "test-other";
    const TEST_MODULE: &str = "provider:test";
    const TEST_OWNER: &str = "test-owner";

    const TEST_PROXY_ADDR: &str = "proxy";
    const TEST_MANAGER_ADDR: &str = "manager";

    pub fn mock_manager_querier() -> MockQuerierBuilder {
        MockQuerierBuilder::default().with_smart_handler(TEST_MANAGER_ADDR, |msg| {
            match from_binary(msg).unwrap() {
                ManagerQueryMsg::Config {} => {
                    let resp = ManagerConfigResponse {
                        owner: TEST_OWNER.to_owned(),
                        version_control_address: TEST_VERSION_CONTROL.to_owned(),
                        module_factory_address: TEST_MODULE_FACTORY.to_owned(),
                        account_id: Uint64::from(TEST_ACCOUNT_ID), // mock value, not used
                    };
                    Ok(to_binary(&resp).unwrap())
                }
                _ => panic!("unexpected message"),
            }
        })
    }

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

    /// Initialize the version_control with admin as creator and test account
    fn mock_init_with_account(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})?;
        execute_as_admin(
            deps.branch(),
            ExecuteMsg::SetFactory {
                new_factory: TEST_ACCOUNT_FACTORY.to_string(),
            },
        )?;
        execute_as(
            deps.branch(),
            TEST_ACCOUNT_FACTORY,
            ExecuteMsg::AddAccount {
                account_id: TEST_ACCOUNT_ID,
                account_base: AccountBase {
                    manager: Addr::unchecked(TEST_MANAGER_ADDR),
                    proxy: Addr::unchecked(TEST_PROXY_ADDR),
                },
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

    mod claim_namespaces {

        use super::*;
        use abstract_core::objects::module_reference::ModuleReference;

        fn test_module() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Version(TEST_VERSION.into())).unwrap()
        }

        // - Query latest

        #[test]
        fn claim_namespaces() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
            let new_namespace1 = Namespace::from("namespace1");
            let new_namespace2 = Namespace::from("namespace2");
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![new_namespace1.to_string(), new_namespace2.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::AccountOwnerMismatch {
                    sender: TEST_OTHER.to_string(),
                    owner: TEST_OWNER.to_string(),
                });

            let res = execute_as(deps.as_mut(), TEST_OWNER, msg);
            assert_that!(&res).is_ok();
            let account_id = namespaces_info().load(&deps.storage, &new_namespace1)?;
            assert_that!(account_id).is_equal_to(TEST_ACCOUNT_ID);
            let account_id = namespaces_info().load(&deps.storage, &new_namespace2)?;
            assert_that!(account_id).is_equal_to(TEST_ACCOUNT_ID);
            Ok(())
        }

        #[test]
        fn remove_namespaces() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
            let new_namespace1 = Namespace::from("namespace1");
            let new_namespace2 = Namespace::from("namespace2");

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![new_namespace1.to_string(), new_namespace2.to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg.clone())?;

            // remove as other
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace1.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::AccountOwnerMismatch {
                    sender: TEST_OTHER.to_string(),
                    owner: TEST_OWNER.to_string(),
                });

            // remove as admin
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace1.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_ADMIN, msg);
            assert_that!(&res).is_ok();
            let exists = namespaces_info().has(&deps.storage, &new_namespace1);
            assert_that!(exists).is_equal_to(false);

            // remove same again
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace1.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::MissingNamespace {
                    namespace: new_namespace1.to_string(),
                });

            // remove as owner
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace2.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_ADMIN, msg);
            assert_that!(&res).is_ok();
            let exists = namespaces_info().has(&deps.storage, &new_namespace1);
            assert_that!(exists).is_equal_to(false);

            Ok(())
        }

        #[test]
        fn yank_orphaned_modules() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;

            // add namespaces
            let new_namespace1 = Namespace::from("namespace1");
            let new_namespace2 = Namespace::from("namespace2");
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![new_namespace1.to_string(), new_namespace2.to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // first add module
            let mut new_module = test_module();
            new_module.provider = new_namespace1.to_string();
            let msg = ExecuteMsg::AddModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // remove as admin
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace1.to_string()],
            };
            execute_as(deps.as_mut(), TEST_ADMIN, msg)?;

            let module = MODULE_LIBRARY.load(&deps.storage, &new_module);
            assert_that!(&module).is_err();
            let module = YANKED_MODULES.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }
    }

    mod add_modules {
        use super::*;
        use abstract_core::objects::module_reference::ModuleReference;
        use abstract_core::AbstractError;

        fn test_module() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Version(TEST_VERSION.into())).unwrap()
        }

        // - Query latest

        #[test]
        fn add_module_by_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
            let mut new_module = test_module();
            new_module.provider = ABSTRACT_NAMESPACE.to_owned();
            let msg = ExecuteMsg::AddModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_ADMIN, msg);
            assert_that!(&res).is_ok();
            let module = MODULE_LIBRARY.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn add_module_by_account_owner() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
            let new_module = test_module();
            let msg = ExecuteMsg::AddModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // try while no namespace
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::MissingNamespace {
                    namespace: new_module.provider.clone(),
                });

            // add namespaces
            execute_as(
                deps.as_mut(),
                TEST_OWNER,
                ExecuteMsg::ClaimNamespaces {
                    account_id: TEST_ACCOUNT_ID,
                    namespaces: vec![new_module.provider.clone()],
                },
            )?;

            // add modules
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg.clone());
            assert_that!(&res).is_ok();
            let module = MODULE_LIBRARY.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn remove_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
            let rm_module = test_module();

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![rm_module.provider.clone()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg.clone())?;

            // first add module
            let msg = ExecuteMsg::AddModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;
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
                .is_equal_to(&VCError::AccountOwnerMismatch {
                    sender: TEST_OTHER.to_string(),
                    owner: TEST_OWNER.to_string(),
                });

            execute_as_admin(deps.as_mut(), msg)?;

            let module = MODULE_LIBRARY.load(&deps.storage, &rm_module);
            assert_that!(&module).is_err();
            Ok(())
        }

        #[test]
        fn yank_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
            let rm_module = test_module();

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![rm_module.provider.clone()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg.clone())?;

            // first add module
            let msg = ExecuteMsg::AddModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;
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
                .is_equal_to(&VCError::AccountOwnerMismatch {
                    sender: TEST_OTHER.to_string(),
                    owner: TEST_OWNER.to_string(),
                });

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
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec!["provider".to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg.clone())?;

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
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
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
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
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
