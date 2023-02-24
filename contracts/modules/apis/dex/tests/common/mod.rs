pub const ROOT_USER: &str = "root_user";
pub const TEST_COIN: &str = "ucoin";
use abstract_boot::{Abstract, AnsHost, Manager, ModuleFactory, OSFactory, Proxy, VersionControl};
use abstract_boot::{DexApi, OS};
use abstract_os::{
    api::InstantiateMsg, objects::gov_type::GovernanceDetails, PROXY, TENDERMINT_STAKING,
};
use abstract_os::{ANS_HOST, MANAGER, MODULE_FACTORY, OS_FACTORY, VERSION_CONTROL};
use boot_core::{
    prelude::{BootInstantiate, BootUpload, ContractInstance},
    Mock,
};
use cosmwasm_std::{Addr, Empty};
use cw_multi_test::ContractWrapper;
use semver::Version;

pub fn create_default_os(factory: &OSFactory<Mock>) -> anyhow::Result<OS<Mock>> {
    let os = factory.create_default_os(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(ROOT_USER).to_string(),
    })?;
    Ok(os)
}

/// Instantiates the dex api and registers it with the version control
#[allow(dead_code)]
pub fn init_dex_api(
    chain: Mock,
    deployment: &Abstract<Mock>,
    version: Option<String>,
) -> anyhow::Result<DexApi<Mock>> {
    let mut dex_api = DexApi::new(TENDERMINT_STAKING, chain);
    dex_api
        .as_instance_mut()
        .set_mock(Box::new(cw_multi_test::ContractWrapper::new_with_empty(
            ::dex::contract::execute,
            ::dex::contract::instantiate,
            ::dex::contract::query,
        )));
    dex_api.upload()?;
    dex_api.instantiate(
        &InstantiateMsg {
            app: Empty {},
            base: abstract_os::api::BaseInstantiateMsg {
                ans_host_address: deployment.ans_host.addr_str()?,
                version_control_address: deployment.version_control.addr_str()?,
            },
        },
        None,
        None,
    )?;

    let version: Version = version
        .unwrap_or_else(|| deployment.version.to_string())
        .parse()?;

    deployment
        .version_control
        .register_apis(vec![dex_api.as_instance()], &version)?;
    Ok(dex_api)
}
