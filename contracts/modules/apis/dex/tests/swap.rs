mod common;

use abstract_boot::*;
use abstract_os::EXCHANGE;
use boot_core::{
    prelude::{instantiate_default_mock_env, ContractInstance},
    Deploy,
};
use common::create_default_os;
use cosmwasm_std::{coin, Addr, Empty};

use wyndex_bundle::EUR;

#[test]
fn swap_native() -> anyhow::Result<()> {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;

    let deployment = Abstract::deploy_on(chain.clone(), "1.0.0".parse()?)?;
    let _root_os = create_default_os(&deployment.os_factory)?;

    deployment.deploy_modules()?;
    let os = create_default_os(&deployment.os_factory)?;

    chain.set_balance(&os.proxy.address()?, vec![coin(10_000, EUR)])?;

    // Set up the dex and staking contracts
    // let exchange_api = init_dex_api(chain.clone(), &deployment, None)?;
    // install dex
    os.manager.install_module(EXCHANGE, &Empty {})?;

    Ok(())
}
