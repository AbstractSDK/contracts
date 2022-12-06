use std::str::FromStr;
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Response};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cw_controllers::AdminError;
use abstract_os::etf::EtfExecuteMsg;
use abstract_os::etf::state::FEE;
use crate::contract::{ETF_ADDON, EtfResult};
use crate::error::EtfError;
use crate::handlers;
use crate::tests::common::TEST_CREATOR;
use crate::tests::handlers::instantiate::etf_init_msg;


fn mock_init(mut deps: DepsMut) -> Response {
    let init_fee = Decimal::from_str("0.01").unwrap();

    let etf_init = etf_init_msg(init_fee, TEST_CREATOR);
    let info = mock_info(TEST_CREATOR, &[]);

    // Set the admin to the creator
    ETF_ADDON.admin.set(deps.branch(), Some(Addr::unchecked(TEST_CREATOR))).unwrap();

     handlers::instantiate_handler(deps, mock_env(), info, ETF_ADDON, etf_init).unwrap()
}

mod test_set_fee {
    use super::*;

    fn set_fee_helper(deps: DepsMut, fee: Decimal) -> EtfResult {
        let msg = EtfExecuteMsg::SetFee {
            fee,
        };
        let info = mock_info(TEST_CREATOR, &[]);

        handlers::execute_handler(deps, mock_env(), info, ETF_ADDON, msg)
    }


    fn assert_eq_fee(deps: Deps, new_fee: Decimal) {
        assert_eq!(new_fee, FEE.load(deps.storage).unwrap().share());
    }

    #[test]
    fn happy_path() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());

        let new_fee = Decimal::percent(20u64);

        let _res = set_fee_helper(deps.as_mut(), new_fee).unwrap();

        assert_eq_fee(deps.as_ref(), new_fee);
    }

    #[test]
    fn zero() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());

        let zero = Decimal::percent(0u64);

        let res = set_fee_helper(deps.as_mut(), zero);
        assert!(res.is_ok());

        assert_eq_fee(deps.as_ref(), zero);
    }

    #[test]
    fn too_high() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());

        let high_fee = Decimal::percent(100u64);

        let res = set_fee_helper(deps.as_mut(), high_fee).unwrap_err();

        // TODO: invalid fee
        assert!(matches!(res, EtfError::Std(..)));
    }


    #[test]
    fn not_admin() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        let new_fee = Decimal::percent(20u64);

        let msg = EtfExecuteMsg::SetFee {
            fee: new_fee,
        };
        let info = mock_info("not_admin", &[]);

        let res = handlers::execute_handler(deps.as_mut(), mock_env(), info, ETF_ADDON, msg).unwrap_err();

        assert_eq!(res, EtfError::Admin(AdminError::NotAdmin {}));
    }
}
