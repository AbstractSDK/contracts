use std::str::FromStr;

use cosmwasm_std::{Addr, CosmosMsg, Decimal, from_binary, WasmMsg};
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_dependencies, mock_env, mock_info};
use cw20::MinterResponse;



use abstract_os::etf::state::{FEE, STATE, State};
use abstract_sdk::os::etf::*;

use crate::contract::{ETF_ADDON};
use crate::handlers;
use crate::tests::common::TEST_CREATOR;

const TEST_TOKEN_CODE_ID: u64 = 0;
const TEST_TOKEN_NAME: &str = "test";
const TEST_TOKEN_SYMBOL: &str = "TEST";

#[track_caller]
pub fn etf_init_msg(fee: Decimal, provider_addr: &str) -> EtfInstantiateMsg {
    EtfInstantiateMsg {
        token_code_id: TEST_TOKEN_CODE_ID,
        fee,
        provider_addr: provider_addr.to_string(),
        token_name: Some(TEST_TOKEN_NAME.to_string()),
        token_symbol: Some(TEST_TOKEN_SYMBOL.to_string()),
    }
}

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies();

    let expected_fee = Decimal::from_str("0.01").unwrap();

    let etf_init = etf_init_msg(expected_fee, TEST_CREATOR);
    let info = mock_info(TEST_CREATOR, &[]);

    ////// Check the LP token response
    let res = handlers::instantiate_handler(deps.as_mut(), mock_env(), info, ETF_ADDON, etf_init).unwrap();

    // Response should have 1 msg instantiating the cw20
    assert_eq!(1, res.messages.len());

    let wasm_msg = match res.messages[0].msg.clone() {
        CosmosMsg::Wasm(msg) => msg,
        _ => panic!("Expected WasmMsg"),
    };

    let expected_cw20_init = cw20_base::msg::InstantiateMsg {
        name: TEST_TOKEN_NAME.to_string(),
        symbol: TEST_TOKEN_SYMBOL.to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: MOCK_CONTRACT_ADDR.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    match wasm_msg {
        WasmMsg::Instantiate { code_id, msg, .. } => {
            assert_eq!(code_id, TEST_TOKEN_CODE_ID);
            // Ensure that the msg is the expected instantiate msg
            assert_eq!(from_binary::<cw20_base::msg::InstantiateMsg>(&msg).unwrap(), expected_cw20_init);
        }
        _ => panic!("Expected WasmMsg::Instantiate"),
    };

    // Check fee
    let actual_fee = FEE.load(deps.as_ref().storage).unwrap().share();
    assert_eq!(actual_fee, expected_fee);

    // Check state
    let actual_state = STATE.load(deps.as_ref().storage).unwrap();

    let expected_state = State {
        // The actual addr is set on response
        liquidity_token_addr: Addr::unchecked(""),
        provider_addr: Addr::unchecked(TEST_CREATOR),
    };
    assert_eq!(actual_state, expected_state);
}
