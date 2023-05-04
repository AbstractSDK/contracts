//! # Staking
//! Interacts with the staking module of cosmos
//!

use cosmos_sdk_proto::{cosmos::base, cosmos::staking, traits::Message};
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Deps};

use crate::{
    features::{AbstractNameService, AccountIdentification},
    AbstractSdkResult,
};

pub trait StakingInterface: AbstractNameService + AccountIdentification {
    fn staking<'a>(&'a self, deps: Deps<'a>) -> Staking<Self> {
        Staking { base: self, deps }
    }
}

impl<T> StakingInterface for T where T: AbstractNameService + AccountIdentification {}

pub struct Staking<'a, T: StakingInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: StakingInterface> Staking<'a, T> {
    /// message for performing a delegation of coins from a delegator to a validator.
    pub fn delegate(
        &self,
        delegator: Addr,
        validator: Addr,
        amount: Option<Coin>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let msg = staking::v1beta1::MsgDelegate {
            delegator_address: delegator.into(),
            validator_address: validator.into(),
            amount: amount.map(|item| base::v1beta1::Coin {
                denom: item.denom,
                amount: item.amount.to_string(),
            }),
        }
        .encode_to_vec();

        Ok(CosmosMsg::Stargate {
            type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
            value: to_binary(&msg)?,
        })
    }

    /// message for performing an undelegation from a delegate and a validator.
    pub fn undelegate(
        &self,
        delegator: Addr,
        validator: Addr,
        amount: Option<Coin>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let msg = staking::v1beta1::MsgUndelegate {
            delegator_address: delegator.into(),
            validator_address: validator.into(),
            amount: amount.map(|item| base::v1beta1::Coin {
                denom: item.denom,
                amount: item.amount.to_string(),
            }),
        }
        .encode_to_vec();

        Ok(CosmosMsg::Stargate {
            type_url: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
            value: to_binary(&msg)?,
        })
    }

    /// message for performing a redelegation of coins from a delegator and source validator to a destination validator.
    pub fn redelegate(
        &self,
        delegator: Addr,
        src: Addr,
        dst: Addr,
        amount: Option<Coin>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let msg = staking::v1beta1::MsgBeginRedelegate {
            delegator_address: delegator.into(),
            validator_src_address: src.into(),
            validator_dst_address: dst.into(),
            amount: amount.map(|item| base::v1beta1::Coin {
                denom: item.denom,
                amount: item.amount.to_string(),
            }),
        }
        .encode_to_vec();

        Ok(CosmosMsg::Stargate {
            type_url: "/cosmos.staking.v1beta1.MsgBeginRedelegate".to_string(),
            value: to_binary(&msg)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    mod delegate {
        use super::*;

        #[test]
        fn delegate() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let staking = app.staking(deps.as_ref());
            let validator = Addr::unchecked("validator");
            let delegator = Addr::unchecked("delegator");
            let msg = staking.delegate(
                delegator,
                validator,
                Some(Coin {
                    denom: "denom".to_string(),
                    amount: 100u128.into(),
                }),
            );
            assert_that!(&msg).is_ok();
        }
    }

    mod undelegate {
        use super::*;

        #[test]
        fn undelegate() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let staking = app.staking(deps.as_ref());
            let validator = Addr::unchecked("validator");
            let delegator = Addr::unchecked("delegator");
            let msg = staking.undelegate(
                delegator,
                validator,
                Some(Coin {
                    denom: "denom".to_string(),
                    amount: 100u128.into(),
                }),
            );
            assert_that!(&msg).is_ok();
        }
    }

    mod redelegate {
        use super::*;

        #[test]
        fn redelegate() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let staking = app.staking(deps.as_ref());
            let delegator = Addr::unchecked("delegator");
            let validator_src = Addr::unchecked("validator_src");
            let validator_dst = Addr::unchecked("validator_dst");
            let msg = staking.redelegate(
                delegator,
                validator_src,
                validator_dst,
                Some(Coin {
                    denom: "denom".to_string(),
                    amount: 100u128.into(),
                }),
            );
            assert_that!(&msg).is_ok();
        }
    }
}
