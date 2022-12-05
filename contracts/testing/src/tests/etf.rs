use std::str::FromStr;

use abstract_os::{app, ETF};
use abstract_sdk::os::{
    etf as msgs,
    etf::state,
    objects::module::{ModuleInfo, ModuleVersion},
    SUBSCRIPTION,
};
use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, Decimal, DepsMut, Empty, Env, Reply, Uint128};
use cw_asset::{AssetInfo, AssetInfoBase};
use cw_controllers::AdminError;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use abstract_os::app::{BaseInstantiateMsg, InstantiateMsg};
use abstract_os::version_control::Core;
use etf::contract::{ETF_ADDON, EtfResult};

use crate::tests::{
    common::{DEFAULT_PAY, RANDOM_USER, SUBSCRIPTION_COST},
    testing_infrastructure::env::{exec_msg_on_manager, mint_tokens},
};
use crate::tests::testing_infrastructure::env::NativeContracts;
use crate::tests::testing_infrastructure::module_installer::install_module;

use super::{
    common::{DEFAULT_VERSION, TEST_CREATOR},
    testing_infrastructure::env::{AbstractEnv, get_os_modules, init_os, mock_app, register_app},
};

pub fn etf_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new_with_empty(
            etf::contract::execute,
            etf::contract::instantiate,
            etf::contract::query,
        )
            .with_migrate_empty(etf::contract::migrate)
            .with_reply(etf::contract::reply),
    )
}


pub fn register_etf(
    app: &mut App,
    sender: &Addr,
    version_control: &Addr,
) -> AnyResult<()> {
    let module_info = ModuleInfo::from_id(
        ETF,
        ModuleVersion::Version(DEFAULT_VERSION.to_string()),
    )
    .unwrap();

    register_app(app, sender, version_control, module_info, etf_contract()).unwrap();
    Ok(())
}

const TEST_TOKEN_NAME: &'static str = "Test";
const TEST_TOKEN_SYMBOL: &'static str = "TEST";

pub fn etf_init_msg(token_code_id: u64, provider_addr: String, fee: Decimal) -> msgs::EtfInstantiateMsg {
    msgs::EtfInstantiateMsg {
        token_code_id,
        fee,
        provider_addr,
        token_name: Some(TEST_TOKEN_NAME.to_string()),
        token_symbol: Some(TEST_TOKEN_SYMBOL.to_string()),
    }
}

pub fn app_init_msg(native_contracts: &NativeContracts, etf_init_msg: msgs::EtfInstantiateMsg) -> InstantiateMsg<msgs::EtfInstantiateMsg> {
    InstantiateMsg {
        base: BaseInstantiateMsg {
            ans_host_address: native_contracts.ans_host.to_string(),
        },
        app: etf_init_msg,
    }
}

pub fn install_etf(
    app: &mut App,
    sender: &Addr,
    env: &AbstractEnv,
    os_id: &u32,
    etf_init_msg: msgs::EtfInstantiateMsg,
) -> AnyResult<()> {
    let app_init_msg = app_init_msg(&env.native_contracts, etf_init_msg);

    let etf_module = ModuleInfo::from_id(
        ETF,
        ModuleVersion::Latest {},
    )?;

    let os_core = env.os_store.get(os_id).unwrap();

    install_module(app, sender, etf_module, os_core, &app_init_msg).unwrap();

    Ok(())
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let mut env = AbstractEnv::new(&mut app, &sender);

    // Register the ETF module with version control
    register_etf(&mut app, &sender, &env.native_contracts.version_control).unwrap();

    // Create a new OS
    let test_os_id = init_os(&mut app, &sender, &env.native_contracts, &mut env.os_store).unwrap();

    let installed_modules = get_os_modules(&app, &env.os_store, &test_os_id).unwrap();

    // By default, only the proxy is installed
    assert_eq!(installed_modules.len(), 1);

    let expected_fee = Decimal::from_str("0.01").unwrap();
    let etf_init_msg = etf_init_msg(*env.code_ids.get("cw_plus:cw20").unwrap(), TEST_CREATOR.to_string(), expected_fee);

    // Install the ETF module
    install_etf(&mut app, &sender, &env, &test_os_id, etf_init_msg).unwrap();

    let installed_modules = get_os_modules(&app, &env.os_store, &test_os_id).unwrap();

    // Now the ETF module is installed
    assert_eq!(installed_modules.len(), 2);

    let etf_addr = installed_modules.get(ETF).unwrap();

    let state: msgs::StateResponse = app
        .wrap()
        .query_wasm_smart(
            etf_addr,
            &app::QueryMsg::App(msgs::EtfQueryMsg::State {}),
        )
        .unwrap();

    // Ensure that the liquidity token contract is a cw20 and has the proper name
    let actual_token_info: cw20::TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &state.liquidity_token,
            &cw20_base::msg::QueryMsg::TokenInfo {},
        )
        .unwrap();
    assert_eq!(actual_token_info.name, TEST_TOKEN_NAME.to_string());
    assert_eq!(actual_token_info.symbol, TEST_TOKEN_SYMBOL.to_string());

    assert_eq!(state.fee, expected_fee);
}
