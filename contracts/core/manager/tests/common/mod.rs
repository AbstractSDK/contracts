pub const ROOT_USER: &str = "root_user";
pub const TEST_COIN: &str = "ucoin";
use ::abstract_manager::contract::CONTRACT_VERSION;
use abstract_boot::OS;
use abstract_boot::{Abstract, AnsHost, Manager, ModuleFactory, OSFactory, Proxy, VersionControl};
use abstract_os::{api::InstantiateMsg, objects::gov_type::GovernanceDetails, PROXY};
use abstract_os::{ANS_HOST, MANAGER, MODULE_FACTORY, OS_FACTORY, VERSION_CONTROL};
use boot_core::{
    boot_contract,
    prelude::{BootInstantiate, BootUpload, ContractInstance},
    Contract, Mock,
};
use cosmwasm_std::{Addr, Empty};
use cw_multi_test::ContractWrapper;
use semver::Version;

pub fn init_abstract_env(chain: Mock) -> anyhow::Result<(Abstract<Mock>, OS<Mock>)> {
    let mut ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let mut os_factory = OSFactory::new(OS_FACTORY, chain.clone());
    let mut version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let mut module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    let mut manager = Manager::new(MANAGER, chain.clone());
    let mut proxy = Proxy::new(PROXY, chain.clone());

    ans_host.as_instance_mut().set_mock(Box::new(
        ContractWrapper::new_with_empty(
            ::ans_host::contract::execute,
            ::ans_host::contract::instantiate,
            ::ans_host::contract::query,
        )
        .with_migrate_empty(::ans_host::contract::migrate),
    ));

    os_factory.as_instance_mut().set_mock(Box::new(
        ContractWrapper::new_with_empty(
            ::os_factory::contract::execute,
            ::os_factory::contract::instantiate,
            ::os_factory::contract::query,
        )
        .with_migrate_empty(::os_factory::contract::migrate)
        .with_reply_empty(::os_factory::contract::reply),
    ));

    module_factory.as_instance_mut().set_mock(Box::new(
        cw_multi_test::ContractWrapper::new_with_empty(
            ::module_factory::contract::execute,
            ::module_factory::contract::instantiate,
            ::module_factory::contract::query,
        )
        .with_migrate_empty(::module_factory::contract::migrate)
        .with_reply_empty(::module_factory::contract::reply),
    ));

    version_control.as_instance_mut().set_mock(Box::new(
        cw_multi_test::ContractWrapper::new_with_empty(
            ::version_control::contract::execute,
            ::version_control::contract::instantiate,
            ::version_control::contract::query,
        )
        .with_migrate_empty(::version_control::contract::migrate),
    ));

    manager.as_instance_mut().set_mock(Box::new(
        cw_multi_test::ContractWrapper::new_with_empty(
            ::abstract_manager::contract::execute,
            ::abstract_manager::contract::instantiate,
            ::abstract_manager::contract::query,
        )
        .with_migrate_empty(::abstract_manager::contract::migrate),
    ));

    proxy.as_instance_mut().set_mock(Box::new(
        cw_multi_test::ContractWrapper::new_with_empty(
            ::proxy::contract::execute,
            ::proxy::contract::instantiate,
            ::proxy::contract::query,
        )
        .with_migrate_empty(::proxy::contract::migrate),
    ));

    // do as above for the rest of the contracts

    let deployment = Abstract {
        chain,
        version: "1.0.0".parse()?,
        ans_host,
        os_factory,
        version_control,
        module_factory,
    };

    let os_core = OS { manager, proxy };

    Ok((deployment, os_core))
}

pub(crate) type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

pub(crate) fn create_default_os(factory: &OSFactory<Mock>) -> anyhow::Result<OS<Mock>> {
    let os = factory.create_default_os(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(ROOT_USER).to_string(),
    })?;
    Ok(os)
}

/// Instantiates the mock api and registers it with the version control
pub fn init_mock_api(
    chain: Mock,
    deployment: &Abstract<Mock>,
    version: Option<String>,
) -> anyhow::Result<BootMockApi<Mock>> {
    let mut staking_api = BootMockApi::new(TEST_MODULE_ID, chain);
    staking_api.upload()?;
    staking_api.instantiate(
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
        .unwrap_or_else(|| CONTRACT_VERSION.to_string())
        .parse()?;

    deployment
        .version_control
        .register_apis(vec![staking_api.as_instance()], &version)?;
    Ok(staking_api)
}

use abstract_api::{ApiContract, ApiError};
use abstract_os::api::{self, BaseInstantiateMsg};
use abstract_sdk::base::InstantiateEndpoint;
use abstract_sdk::AbstractSdkError;
use abstract_testing::{
    TEST_ADMIN, TEST_ANS_HOST, TEST_MODULE_ID, TEST_VERSION, TEST_VERSION_CONTROL,
};
use cosmwasm_std::{
    testing::{mock_env, mock_info},
    DepsMut, Env, MessageInfo, Response, StdError,
};
use thiserror::Error;

pub const TEST_METADATA: &str = "test_metadata";
pub const TEST_TRADER: &str = "test_trader";

#[derive(Error, Debug, PartialEq)]
pub enum MockError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Api(#[from] ApiError),

    #[error("{0}")]
    Abstract(#[from] abstract_os::AbstractOsError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),
}

#[cosmwasm_schema::cw_serde]
pub struct MockApiExecMsg;

impl api::ApiExecuteMsg for MockApiExecMsg {}

/// Mock API type
pub type MockApi = ApiContract<MockError, Empty, MockApiExecMsg, Empty>;

/// use for testing
pub const MOCK_API: MockApi = MockApi::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
    .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_response".as_bytes())))
    .with_instantiate(mock_init_handler);

pub type ApiMockResult = Result<(), MockError>;

pub fn mock_init(deps: DepsMut) -> Result<Response, MockError> {
    let api = MOCK_API;
    let info = mock_info(TEST_ADMIN, &[]);
    let init_msg = InstantiateMsg {
        base: BaseInstantiateMsg {
            ans_host_address: TEST_ANS_HOST.into(),
            version_control_address: TEST_VERSION_CONTROL.into(),
        },
        app: Empty {},
    };
    api.instantiate(deps, mock_env(), info, init_msg)
}

fn mock_init_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _api: MockApi,
    _msg: Empty,
) -> Result<Response, MockError> {
    Ok(Response::new().set_data("mock_response".as_bytes()))
}

use abstract_os::api::{ExecuteMsg as ApiExecMsg, QueryMsg};

abstract_api::export_endpoints!(MOCK_API, MockApi);

type ExecuteMsg = ApiExecMsg<MockApiExecMsg>;
#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct BootMockApi;

impl<Chain: boot_core::BootEnvironment> BootMockApi<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(
            Contract::new(name, chain).with_mock(Box::new(ContractWrapper::new_with_empty(
                self::execute,
                self::instantiate,
                self::query,
            ))),
        )
    }
}
