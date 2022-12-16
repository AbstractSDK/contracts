type Res = anyhow::Result<()>;

use abstract_boot::boot::*;
use abstract_boot::{Deployment, os_factory::{OSFactory, OsFactoryExecFns}};


use cosmwasm_std::Addr;

use crate::tests::common;

#[test]
fn instantiate() -> Res {
    // Now we do the same but on a cw-multi-test environment!
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;

    let mut deployment = Deployment::new(&chain, common::DEFAULT_VERSION.to_string().parse()?);
    deployment.deploy()?;

    let factory: OSFactory<Mock> = deployment.os_factory;
    factory.create_os()?;
    Ok(())
}
