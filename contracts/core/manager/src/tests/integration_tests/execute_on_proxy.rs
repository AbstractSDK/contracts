use super::init::init_abstract_env;
use crate::{tests::common::{self, TEST_COIN}, contract::CONTRACT_VERSION};
use abstract_boot::{os_factory::OsFactoryQueryFns, OsFactoryExecFns, VCQueryFns, OS, *};
use abstract_os::{objects::{gov_type::GovernanceDetails, module::ModuleInfo}, os_factory, version_control::Core, manager::{ManagerModuleInfo, state::ModuleId}, PROXY};
use anyhow::Chain;
use boot_core::{
    prelude::{instantiate_default_mock_env, ContractInstance},
    IndexResponse, Mock,
};
use cosmwasm_std::{Addr, Uint64, Coin, CosmosMsg};
use speculoos::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

fn create_default_os(chain: &Mock, factory: &OSFactory<Mock>) -> anyhow::Result<OS<Mock>> {
    let os = factory.create_default_os(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(common::ROOT_USER).to_string(),
    })?;
    Ok(os)
}


#[test]
fn instantiate() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;

    let modules = os.manager.module_infos(None, None)?.module_infos;
    
    // assert proxy module
    assert_that(&modules.len()).is_equal_to(1);
    assert_that(&modules[0]).is_equal_to(&ManagerModuleInfo {
        address: os.proxy.address()?.into_string(),
        id: PROXY.to_string(),
        version: cw2::ContractVersion { contract: PROXY.into(), version: CONTRACT_VERSION.into() },
    });

    // assert manager config
    assert_that!(os.manager.config()?).is_equal_to(abstract_os::manager::ConfigResponse {
        root: sender.to_string(),
        version_control_address: deployment.version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
        os_id: 0u32.into(),
    });
    Ok(())
}

#[test]
fn execute_on_proxy_through_manager() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;

    // mint coins to proxy address
    chain.init_balance(&os.proxy.address()?, vec![Coin::new(100_000, TEST_COIN)])?;

    // burn coins from proxy
    let proxy_balance = chain.app.borrow().wrap().query_all_balances(&os.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000, TEST_COIN)]);

    let burn_amount: Vec<Coin> = vec![Coin::new(10_000, TEST_COIN)];

    os.manager.exec_on_module(cosmwasm_std::to_binary(&abstract_os::proxy::ExecuteMsg::ModuleAction { msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn { amount: burn_amount })] })?, PROXY.to_string())?;

    let proxy_balance = chain.app.borrow().wrap().query_all_balances(&os.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000 - 10_000, TEST_COIN)]);

    Ok(())
}

#[test]
fn create_two_os_s() -> AResult {
    let _not_owner = Addr::unchecked("not_owner");
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;

    let factory = &deployment.os_factory;
    let version_control = &deployment.version_control;
    // first os
    let os_1 = factory.create_os(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        String::from("first_os"),
        Some(String::from("os_description")),
        Some(String::from("os_link_of_at_least_11_char")),
    )?;
    // second os
    let os_2 = factory.create_os(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        String::from("second_os"),
        Some(String::from("os_description")),
        Some(String::from("os_link_of_at_least_11_char")),
    )?;

    let manager1 = os_1.event_attr_value("wasm", "manager_address")?;
    let proxy1 = os_1.event_attr_value("wasm", "proxy_address")?;

    let manager2 = os_2.event_attr_value("wasm", "manager_address")?;
    let proxy2 = os_2.event_attr_value("wasm", "proxy_address")?;

    let factory_config = factory.config()?;
    let expected = os_factory::ConfigResponse {
        owner: sender.clone().into_string(),
        ans_host_contract: deployment.ans_host.address()?.into(),
        version_control_contract: deployment.version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
        subscription_address: None,
        next_os_id: 2,
    };

    assert_that!(&factory_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_os::version_control::ConfigResponse {
        admin: sender.clone().into_string(),
        factory: factory.address()?.into_string(),
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let os_1 = version_control.os_core(0)?.os_core;
    assert_that!(&os_1).is_equal_to(Core {
        manager: Addr::unchecked(manager1),
        proxy: Addr::unchecked(proxy1),
    });

    let os_2 = version_control.os_core(1)?.os_core;
    assert_that!(&os_2).is_equal_to(Core {
        manager: Addr::unchecked(manager2),
        proxy: Addr::unchecked(proxy2),
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_monarchy() -> AResult {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;

    let factory = &deployment.os_factory;
    let version_control = &deployment.version_control;
    let os_creation = factory.create_os(
        GovernanceDetails::Monarchy {
            monarch: owner.to_string(),
        },
        String::from("first_os"),
        Some(String::from("os_description")),
        Some(String::from("os_link_of_at_least_11_char")),
    )?;

    let manager = os_creation.event_attr_value("wasm", "manager_address")?;
    let proxy = os_creation.event_attr_value("wasm", "proxy_address")?;

    let os = version_control.os_core(0)?.os_core;

    let os_1 = OS::new(&chain, Some(0));
    assert_that!(Core {
        manager: os_1.manager.address()?,
        proxy: os_1.proxy.address()?,
    })
    .is_equal_to(&os);

    assert_that!(Core {
        manager: Addr::unchecked(manager),
        proxy: Addr::unchecked(proxy),
    })
    .is_equal_to(&os);

    let os_config = os_1.manager.config()?;

    assert_that!(os_config).is_equal_to(abstract_os::manager::ConfigResponse {
        root: owner.into_string(),
        os_id: Uint64::from(0u64),
        version_control_address: version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_external() -> AResult {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;

    let factory = &deployment.os_factory;
    let version_control = &deployment.version_control;
    factory.create_os(
        GovernanceDetails::External {
            governance_address: owner.to_string(),
            governance_type: "some_gov_description".to_string(),
        },
        String::from("first_os"),
        Some(String::from("os_description")),
        Some(String::from("os_link_of_at_least_11_char")),
    )?;

    let os = OS::new(&chain, Some(0));
    let os_config = os.manager.config()?;

    assert_that!(os_config).is_equal_to(abstract_os::manager::ConfigResponse {
        root: owner.into_string(),
        os_id: Uint64::from(0u64),
        version_control_address: version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
    });

    Ok(())
}
