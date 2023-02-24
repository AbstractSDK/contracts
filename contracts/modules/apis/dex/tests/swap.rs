mod common;

use abstract_boot::*;
use abstract_os::PROXY;
use boot_core::{prelude::{instantiate_default_mock_env, ContractInstance}, Deploy};
use common::{create_default_os, TEST_COIN};
use cosmwasm_std::{Addr, Coin, CosmosMsg};
use speculoos::prelude::*;

#[test]
fn execute_on_proxy_through_manager() -> anyhow::Result<()> {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;

    let deployment = Abstract::deploy_on(chain.clone(), "1.0.0".parse()?)?;
    let os = create_default_os(&deployment.os_factory)?;

    // mint coins to proxy address
    chain.set_balance(&os.proxy.address()?, vec![Coin::new(100_000, TEST_COIN)])?;

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
