use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{DepsMut, Env, Response, StdError};
use semver;
use serde::Serialize;
use speculoos::prelude::*;

pub fn run_migrate_tests<MigrateMsg, E>(
    mock_init: &dyn Fn(DepsMut) -> Result<Response, E>,
    contract_migrate: &dyn Fn(DepsMut, Env, MigrateMsg) -> Result<cosmwasm_std::Response, E>,
    contract_name: impl Into<String>,
    to_version: &str,
    expected_version: &str,
    expected_error: Option<E>,
    migrate_msg: MigrateMsg,
) -> Result<(), E>
where
    MigrateMsg: Serialize,
    E: std::error::Error + PartialEq + 'static + From<StdError>,
{
    let mut deps = mock_dependencies();
    mock_init(deps.as_mut())?;

    cw2::set_contract_version(deps.as_mut().storage, contract_name, to_version)?;

    let migrate_result = contract_migrate(deps.as_mut(), mock_env(), migrate_msg);

    match expected_error {
        Some(error) => {
            assert_that!(migrate_result).is_err().is_equal_to(error);
        }
        None => {
            let version: semver::Version = expected_version.parse().unwrap();
            let res = migrate_result?;
            assert_that!(res.messages).has_length(0);
            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
        }
    };

    Ok(())
}
