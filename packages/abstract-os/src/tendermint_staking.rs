//! # Tendermint Staking API
//!
//! `abstract_os::tendermint_staking` exposes all the function of [`cosmwasm_std::CosmosMsg::Staking`], [`cosmwasm_std::CosmosMsg::Distribution`] and [`cosmwasm_std::CosmosMsg::StakingQuery`].

use cosmwasm_std::{CosmosMsg, Uint128, StakingQuery, Validator, FullDelegation, Delegation};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Delegate { validator: String, amount: Uint128 },
    Undelegate { validator: String, amount: Uint128 },
    Redelegate {
        source_validator: String,
        destination_validator: String,
        amount: Uint128,
    },
    SetWithdrawAddress {
        /// The new `withdraw_address`
        new_withdraw_address: String,
    },
    WithdrawDelegatorReward {
        /// The `validator_address`
        validator: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

/// These queries are available but you should prefer to use the native implementation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BondedDenom {},
    AllDelegations { delegator: String },
    Delegation {
        delegator: String,
        validator: String,
    },
    AllValidators {},
    Validator {
        address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryBondedDenomResponse {
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryAllDelegationsResponse {
    pub delegations: Vec<Delegation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryDelegationResponse {
    pub delegation: Option<FullDelegation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryAllValidatorsResponse {
    pub validators: Vec<Validator>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryValidatorResponse {
    pub validator: Option<Validator>,
}