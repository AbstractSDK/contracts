use crate::state::*;
use abstract_core::objects::account::AccountTrace;
use abstract_sdk::core::account_factory::*;
use cosmwasm_std::{Deps, StdError, StdResult};

use cw_storage_plus::Bound;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let admin = ADMIN.get(deps)?.unwrap();
    let resp = ConfigResponse {
        owner: admin.into(),
        version_control_contract: state.version_control_contract.into(),
        ans_host_contract: state.ans_host_contract.into(),
        module_factory_address: state.module_factory_address.into(),
    };

    Ok(resp)
}

pub fn query_sequences(
    deps: Deps,
    start_after: Option<AccountTrace>,
    limit: Option<u8>,
) -> StdResult<SequencesResponse> {
    let start_after = start_after.as_ref().map(Bound::exclusive);
    let sequences: Result<Vec<(AccountTrace, u32)>, StdError> = cw_paginate::paginate_map(
        &ACCOUNT_SEQUENCES,
        deps.storage,
        start_after,
        limit.map(|e| e as u32),
        |k, v| Ok((k, v)),
    );
    Ok(SequencesResponse {
        sequences: sequences?,
    })
}

pub fn query_sequence(deps: Deps, origin: AccountTrace) -> StdResult<SequenceResponse> {
    let sequence = ACCOUNT_SEQUENCES.load(deps.storage, &origin)?;
    Ok(SequenceResponse { sequence })
}
