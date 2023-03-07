mod common;

use abstract_boot::{Abstract, AbstractBootError, Manager, OS};
use abstract_manager::error::ManagerError;
use abstract_os::app::{self, BaseInstantiateMsg};
use abstract_os::objects::module::ModuleVersion;
use abstract_testing::prelude::TEST_VERSION;
use boot_core::{
    instantiate_default_mock_env, Addr, BootError, ContractInstance, Deploy, Empty, Mock,
};
use common::mock_modules::*;
use common::{create_default_os, init_abstract_env, init_mock_api, AResult, TEST_COIN};
use speculoos::prelude::*;

fn install_module_version(
    manager: &Manager<Mock>,
    abstr: &Abstract<Mock>,
    module: &str,
    version: &str,
) -> anyhow::Result<String> {
    manager.install_module_version(
        module,
        ModuleVersion::Version(version.to_string()),
        &app::InstantiateMsg {
            app: Empty {},
            base: BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.addr_str()?,
            },
        },
    )?;

    Ok(manager.module_info(module)?.unwrap().address)
}

#[test]
fn install_app_successful() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    deploy_modules(&chain);
    let os = create_default_os(&abstr.os_factory)?;
    let OS { manager, proxy: _ } = &os;
    // dependency for mock_api1 not met
    let res = install_module_version(manager, &abstr, MOCK_APP1_ID, V1);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string()).contains(
        "module tester:mock-api1 is a dependency of tester:mock-app1 and is not installed.",
    );

    // install api 1
    let api1 = install_module_version(manager, &abstr, MOCK_API1_ID, V1)?;

    // second dependency still not met
    let res = install_module_version(manager, &abstr, MOCK_APP1_ID, V1);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string()).contains(
        "module tester:mock-api2 is a dependency of tester:mock-app1 and is not installed.",
    );

    // install api 2
    let api2 = install_module_version(manager, &abstr, MOCK_API2_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(manager, &abstr, MOCK_APP1_ID, V1)?;

    os.expect_modules(vec![api1, api2, app1])?;
    Ok(())
}

#[test]
fn install_app_versions_not_met() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    deploy_modules(&chain);
    let os = create_default_os(&abstr.os_factory)?;
    let OS { manager, proxy: _ } = &os;
    
}