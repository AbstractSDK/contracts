//! # Distribution
//! Interacts with the distribution module of cosmos
//!

use cosmos_sdk_proto::{
    cosmos::{base, distribution},
    traits::Message,
};
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg};

use crate::{
    AbstractSdkResult,
};

pub trait DistributionInterface {
    fn distribution() -> Distribution {
        Distribution {}
    }
}

#[derive(Clone)]
pub struct Distribution {}

impl Distribution {
    /// sets the withdraw address for a delegator (or validator self-delegation).
    pub fn set_withdraw_address(delegator: Addr, withdraw: Addr) -> AbstractSdkResult<CosmosMsg> {
        let msg = distribution::v1beta1::MsgSetWithdrawAddress {
            delegator_address: delegator.into(),
            withdraw_address: withdraw.into(),
        }
        .encode_to_vec();

        Ok(CosmosMsg::Stargate {
            type_url: "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
            value: to_binary(&msg)?,
        })
    }

    /// represents delegation withdrawal to a delegator from a single validator.
    pub fn withdraw_delegator_reward(
        validator: Addr,
        delegator: Addr,
    ) -> AbstractSdkResult<CosmosMsg> {
        let msg = distribution::v1beta1::MsgWithdrawDelegatorReward {
            validator_address: validator.into(),
            delegator_address: delegator.into(),
        }
        .encode_to_vec();

        Ok(CosmosMsg::Stargate {
            type_url: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
            value: to_binary(&msg)?,
        })
    }

    /// withdraws the full commission to the validator address.
    pub fn withdraw_delegator_comission(validator: Addr) -> AbstractSdkResult<CosmosMsg> {
        let msg = distribution::v1beta1::MsgWithdrawValidatorCommission {
            validator_address: validator.into(),
        }
        .encode_to_vec();

        Ok(CosmosMsg::Stargate {
            type_url: "/cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission".to_string(),
            value: to_binary(&msg)?,
        })
    }

    /// allows an account to directly fund the community pool.
    pub fn fund_community_pool(amount: Vec<Coin>, depositor: Addr) -> AbstractSdkResult<CosmosMsg> {
        let msg = distribution::v1beta1::MsgFundCommunityPool {
            amount: amount
                .into_iter()
                .map(|item| base::v1beta1::Coin {
                    denom: item.denom,
                    amount: item.amount.to_string(),
                })
                .collect(),
            depositor: depositor.into(),
        }
        .encode_to_vec();

        Ok(CosmosMsg::Stargate {
            type_url: "/cosmos.distribution.v1beta1.MsgFundCommunityPool".to_string(),
            value: to_binary(&msg)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    // use crate::mock_module::*;
    // use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    mod set_withdraw_address {
        use super::*;

        #[test]
        fn set_withdraw_address() {
            let delegator = Addr::unchecked("delegator");
            let withdraw = Addr::unchecked("withdraw");
            let msg = Distribution::set_withdraw_address(delegator, withdraw);
            assert_that!(&msg).is_ok();
        }
    }

    mod withdraw_delegator_reward {
        use super::*;

        #[test]
        fn withdraw_delegator_reward() {
            let validator = Addr::unchecked("validator");
            let delegator = Addr::unchecked("delegator");
            let msg = Distribution::withdraw_delegator_reward(validator, delegator);
            assert_that!(&msg).is_ok();
        }
    }

    mod withdraw_delegator_comission {
        use super::*;

        #[test]
        fn withdraw_delegator_comission() {
            let validator = Addr::unchecked("validator");
            let msg = Distribution::withdraw_delegator_comission(validator);
            assert_that!(&msg).is_ok();
        }
    }

    mod fund_community_pool {
        use super::*;
        use cosmwasm_std::coins;

        #[test]
        fn fund_community_pool() {
            let depositor = Addr::unchecked("depositor");
            let amount = coins(1000, "coin");
            let msg = Distribution::fund_community_pool(amount, depositor);
            assert_that!(&msg).is_ok();
        }
    }
}
