use cosmwasm_std::{MessageInfo, Uint128, Deps, Env, QueryRequest, StakingQuery};

use crate::contract::TendermintStakeResult;


pub fn delegate(deps: Deps, env: Env, info: MessageInfo, validator: String, amount: Uint128) -> TendermintStakeResult {
    deps.querier.query_bonded_denom();
    QueryRequest::Staking(StakingQuery::BondedDenom {  });
}