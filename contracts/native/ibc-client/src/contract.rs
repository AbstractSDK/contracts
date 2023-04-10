use crate::{commands, error::IbcClientError, queries};
use abstract_core::objects::module_version::assert_cw_contract_upgrade;
use abstract_core::{
    ibc_client::{state::*, *},
    objects::{
        ans_host::AnsHost,
        module_version::{migrate_module_data, set_module_data},
    },
    IBC_CLIENT,
};
use abstract_macros::abstract_response;
use cosmwasm_std::{
    to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdResult,
};
use cw2::{set_contract_version};
use cw_semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const MAX_RETRIES: u8 = 5;

pub(crate) type IbcClientResult<T = Response> = Result<T, IbcClientError>;

#[abstract_response(IBC_CLIENT)]
pub(crate) struct IbcClientResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> IbcClientResult {
    set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
    set_module_data(
        deps.storage,
        IBC_CLIENT,
        CONTRACT_VERSION,
        &[],
        None::<String>,
    )?;
    let cfg = Config {
        chain: msg.chain,
        version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
    };
    CONFIG.save(deps.storage, &cfg)?;
    ANS_HOST.save(
        deps.storage,
        &AnsHost {
            address: deps.api.addr_validate(&msg.ans_host_address)?,
        },
    )?;

    ADMIN.set(deps, Some(info.sender))?;
    Ok(IbcClientResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> IbcClientResult {
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => {
            let new_admin = deps.api.addr_validate(&admin)?;
            ADMIN
                .execute_update_admin(deps, info, Some(new_admin))
                .map_err(Into::into)
        }
        ExecuteMsg::UpdateConfig {
            ans_host,
            version_control,
        } => commands::execute_update_config(deps, info, ans_host, version_control)
            .map_err(Into::into),
        ExecuteMsg::SendPacket {
            host_chain,
            action,
            callback_info,
            retries,
        } => commands::execute_send_packet(
            deps,
            env,
            info,
            host_chain,
            action,
            callback_info,
            retries,
        ),
        ExecuteMsg::SendFunds { host_chain, funds } => {
            commands::execute_send_funds(deps, env, info, host_chain, funds).map_err(Into::into)
        }
        ExecuteMsg::Register { host_chain } => {
            commands::execute_register_os(deps, env, info, host_chain)
        }
        ExecuteMsg::RemoveHost { host_chain } => {
            commands::execute_remove_host(deps, info, host_chain).map_err(Into::into)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::Account { chain, account_id } => {
            to_binary(&queries::query_account(deps, chain, account_id)?)
        }
        QueryMsg::ListAccounts {} => to_binary(&queries::query_list_accounts(deps)?),
        QueryMsg::LatestQueryResult { chain, account_id } => to_binary(
            &queries::query_latest_ibc_query_result(deps, chain, account_id)?,
        ),
        QueryMsg::ListChannels {} => to_binary(&queries::query_list_channels(deps)?),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> IbcClientResult {
    let to_version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_cw_contract_upgrade(deps.storage, to_version, IBC_CLIENT)?;
    set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
    migrate_module_data(deps.storage, IBC_CLIENT, CONTRACT_VERSION, None::<String>)?;
    Ok(IbcClientResponse::action("migrate"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queries::query_config;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };
    use cw2::CONTRACT;

    const CREATOR: &str = "creator";

    use abstract_core::AbstractError;
    use abstract_testing::prelude::{TEST_ANS_HOST, TEST_VERSION_CONTROL};
    use speculoos::prelude::*;

    fn mock_init(deps: DepsMut) -> IbcClientResult {
        let msg = InstantiateMsg {
            chain: "test_chain".into(),
            ans_host_address: TEST_ANS_HOST.into(),
            version_control_address: TEST_VERSION_CONTROL.into(),
        };
        let info = mock_info(CREATOR, &[]);
        instantiate(deps, mock_env(), info, msg)
    }

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            chain: "test_chain".into(),
            ans_host_address: TEST_ANS_HOST.into(),
            version_control_address: TEST_VERSION_CONTROL.into(),
        };
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_that!(res.messages).is_empty();

        // config
        let expected_config = Config {
            chain: "test_chain".into(),
            version_control_address: Addr::unchecked(TEST_VERSION_CONTROL),
        };

        let config_resp = query_config(deps.as_ref()).unwrap();
        assert_that!(config_resp.admin.as_str()).is_equal_to(CREATOR);

        let actual_config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_that!(actual_config).is_equal_to(expected_config);

        // CW2
        let cw2_info = CONTRACT.load(&deps.storage).unwrap();
        assert_that!(cw2_info.version).is_equal_to(CONTRACT_VERSION.to_string());
        assert_that!(cw2_info.contract).is_equal_to(IBC_CLIENT.to_string());

        // ans host
        let actual_ans_host = ANS_HOST.load(deps.as_ref().storage).unwrap();
        assert_that!(actual_ans_host.address.as_str()).is_equal_to(TEST_ANS_HOST);
    }

    #[test]
    fn migrate_disallows_downgrade() -> IbcClientResult<()> {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let migrate_msg = MigrateMsg {};
        let res = migrate(deps.as_mut(), mock_env(), migrate_msg);
        assert_that!(res)
            .is_err()
            .is_equal_to(IbcClientError::Abstract(
                AbstractError::CannotDowngradeContract {
                    from: CONTRACT_VERSION.parse().unwrap(),
                    to: CONTRACT_VERSION.parse().unwrap(),
                },
            ));

        Ok(())
    }
}
