type Res = anyhow::Result<()>;

use abstract_boot::boot::*;
use abstract_boot::{Deployment, OSFactory};
use abstract_os::objects::gov_type::GovernanceDetails;
use abstract_os::os_factory::*;
use cosmwasm_std::Addr;

use crate::tests::common;

#[test]
fn instantiate() -> Res {
    // Now we do the same but on a cw-multi-test environment!
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;

    let mut deployment = Deployment::new(&chain, common::DEFAULT_VERSION.to_string().parse()?);
    deployment.deploy()?;

    deployment.os_factory;

    Ok(())
}
