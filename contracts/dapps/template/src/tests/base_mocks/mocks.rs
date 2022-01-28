use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{DepsMut, Env};

use crate::dapp_base::common::{MEMORY_CONTRACT, TEST_CREATOR, TRADER_CONTRACT};
use crate::msg::ExecuteMsg;
use pandora::treasury::dapp_base::msg::{BaseExecuteMsg, BaseInstantiateMsg};

use crate::contract::{execute, instantiate};

pub(crate) fn instantiate_msg() -> BaseInstantiateMsg {
    BaseInstantiateMsg {
        memory_addr: MEMORY_CONTRACT.to_string(),
    }
}

/**
 * Mocks instantiation of the contract.
 */
pub fn mock_instantiate(mut deps: DepsMut) -> Env {
    let info = mock_info(TEST_CREATOR, &[]);
    let env = mock_env();
    let _res = instantiate(deps.branch(), mock_env(), info.clone(), instantiate_msg())
        .expect("contract successfully handles InstantiateMsg");

    // Add one trader
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: Some(vec![TRADER_CONTRACT.to_string()]),
        to_remove: None,
    });

    execute(deps.branch(), env.clone(), info.clone(), msg).unwrap();

    // Set treasury addr
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: Some("new_treasury_address".to_string()),
    });

    execute(deps, env.clone(), info, msg).unwrap();
    env
}

// /**
//  * Mocks adding asset to the [ADDRESS_BOOK].
//  */
// #[allow(dead_code)]
// pub fn mock_add_to_address_book(deps: DepsMut, asset_address_pair: (String, String)) {
//     let env = mock_env();

//     let (asset, address) = asset_address_pair;
//     // add address
//     let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateAddressBook {
//         to_add: vec![(asset, address)],
//         to_remove: vec![],
//     });

//     let info = mock_info(TEST_CREATOR, &[]);
//     execute(deps, env.clone(), info, msg).unwrap();
// }
