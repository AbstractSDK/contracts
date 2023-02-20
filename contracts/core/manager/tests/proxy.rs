mod common;
use abstract_boot::*;
use abstract_manager::contract::CONTRACT_VERSION;
use abstract_os::{manager::ManagerModuleInfo, PROXY};
use boot_core::prelude::{instantiate_default_mock_env, ContractInstance};
use common::{create_default_os, init_abstract_env, AResult, TEST_COIN};
use cosmwasm_std::{Addr, Coin, CosmosMsg};
use speculoos::prelude::*;

#[test]
fn instantiate() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&deployment.os_factory)?;

    let modules = os.manager.module_infos(None, None)?.module_infos;

    // assert proxy module
    assert_that!(&modules).has_length(1);
    assert_that(&modules[0]).is_equal_to(&ManagerModuleInfo {
        address: os.proxy.address()?.into_string(),
        id: PROXY.to_string(),
        version: cw2::ContractVersion {
            contract: PROXY.into(),
            version: CONTRACT_VERSION.into(),
        },
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
fn exec_through_manager() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(chain.clone())?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&deployment.os_factory)?;

    // mint coins to proxy address
    chain.init_balance(&os.proxy.address()?, vec![Coin::new(100_000, TEST_COIN)])?;

    // burn coins from proxy
    let proxy_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(os.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000, TEST_COIN)]);

    let burn_amount: Vec<Coin> = vec![Coin::new(10_000, TEST_COIN)];

    os.manager.exec_on_module(
        cosmwasm_std::to_binary(&abstract_os::proxy::ExecuteMsg::ModuleAction {
            msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
                amount: burn_amount,
            })],
        })?,
        PROXY.to_string(),
    )?;

    let proxy_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(os.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000 - 10_000, TEST_COIN)]);

    Ok(())
}

/// This basically just checks that the proxy is able to be migrated .... but the actual version cannot change... unless we mock the responses from the version queries
#[test]
fn migrate_proxy() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&deployment.os_factory)?;

    let new_version = "1.0.1".parse().unwrap();
    deployment
        .version_control
        .register_cores(vec![os.proxy.as_instance()], &new_version)?;

    os.manager
        .upgrade_module(PROXY, &abstract_os::proxy::MigrateMsg {})?;

    let module = os.manager.module_info(PROXY)?;

    assert_that!(module)
        .is_some()
        .map(|m| &m.version)
        .is_equal_to(cw2::ContractVersion {
            contract: PROXY.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        });

    Ok(())
}
