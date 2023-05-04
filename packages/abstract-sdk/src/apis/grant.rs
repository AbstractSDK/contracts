//! # Grant
//! Interacts with the feegrant module of cosmos
//!

/*
impl TypeUrl for cosmos::feegrant::v1beta1::MsgGrantAllowance {
    const TYPE_URL: &'static str = "/cosmos.feegrant.v1beta1.MsgGrantAllowance";
}

impl TypeUrl for cosmos::feegrant::v1beta1::MsgRevokeAllowance {
    const TYPE_URL: &'static str = "/cosmos.feegrant.v1beta1.MsgRevokeAllowance";
}

impl TypeUrl for cosmos::feegrant::v1beta1::BasicAllowance {
    const TYPE_URL: &'static str = "/cosmos.feegrant.v1beta1.BasicAllowance";
}

impl TypeUrl for cosmos::feegrant::v1beta1::PeriodicAllowance {
    const TYPE_URL: &'static str = "/cosmos.feegrant.v1beta1.PeriodicAllowance";
}

impl TypeUrl for cosmos::feegrant::v1beta1::AllowedMsgAllowance {
    const TYPE_URL: &'static str = "/cosmos.feegrant.v1beta1.AllowedMsgAllowance";
}
*/

use std::time::Duration;

use crate::{
    features::{AbstractNameService, AccountIdentification},
    AbstractSdkResult,
};
use cosmos_sdk_proto::{prost, traits::Message, Any};
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Deps, Timestamp};

pub trait GrantInterface: AbstractNameService + AccountIdentification {
    fn grant<'a>(&'a self, deps: Deps<'a>) -> Grant<Self> {
        Grant { base: self, deps }
    }
}

impl<T> GrantInterface for T where T: AbstractNameService + AccountIdentification {}

pub struct Grant<'a, T: GrantInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: GrantInterface> Grant<'a, T> {
    pub fn basic_allowance(
        &self,
        granter: Addr,
        grantee: Addr,
        basic: BasicAllowance,
    ) -> AbstractSdkResult<CosmosMsg> {
        let allowance = Any {
            type_url: "/cosmos.feegrant.v1beta1.BasicAllowance".to_string(),
            value: build_basic_allowance(basic).encode_to_vec(),
        };

        let msg = cosmos_sdk_proto::cosmos::feegrant::v1beta1::MsgGrantAllowance {
            granter: granter.into(),
            grantee: grantee.into(),
            allowance: Some(allowance),
        }
        .encode_to_vec();

        Ok(CosmosMsg::Stargate {
            type_url: "/cosmos.feegrant.v1beta1.MsgGrantAllowance".to_string(),
            value: to_binary(&msg)?,
        })
    }

    pub fn periodic_allowance(
        &self,
        granter: Addr,
        grantee: Addr,
        basic: Option<BasicAllowance>,
        periodic: PeriodicAllowance,
    ) -> AbstractSdkResult<CosmosMsg> {
        let allowance = Any {
            type_url: "/cosmos.feegrant.v1beta1.PeriodicAllowance".to_string(),
            value: cosmos_sdk_proto::cosmos::feegrant::v1beta1::PeriodicAllowance {
                basic: Some(basic).map(|basic| build_basic_allowance(basic.unwrap())),
                period: periodic.period.map(|p| prost_types::Duration {
                    seconds: p.as_secs() as i64,
                    nanos: 0,
                }),
                period_spend_limit: convert_coins(periodic.period_spend_limit),
                period_can_spend: convert_coins(periodic.period_can_spend),
                period_reset: periodic.period_reset.map(convert_stamp),
            }
            .encode_to_vec(),
        };

        let msg = cosmos_sdk_proto::cosmos::feegrant::v1beta1::MsgGrantAllowance {
            granter: granter.into(),
            grantee: grantee.into(),
            allowance: Some(allowance),
        }
        .encode_to_vec();

        Ok(CosmosMsg::Stargate {
            type_url: "/cosmos.feegrant.v1beta1.MsgGrantAllowance".to_string(),
            value: to_binary(&msg)?,
        })
    }
}

pub struct BasicAllowance {
    pub spend_limit: Vec<Coin>,
    pub expiration: Option<Timestamp>,
}
pub struct PeriodicAllowance {
    pub period: Option<Duration>,
    pub period_spend_limit: Vec<Coin>,
    pub period_can_spend: Vec<Coin>,
    pub period_reset: Option<Timestamp>,
}

fn build_basic_allowance(
    basic: BasicAllowance,
) -> cosmos_sdk_proto::cosmos::feegrant::v1beta1::BasicAllowance {
    cosmos_sdk_proto::cosmos::feegrant::v1beta1::BasicAllowance {
        spend_limit: convert_coins(basic.spend_limit),
        expiration: basic.expiration.map(convert_stamp),
    }
}

fn convert_coins(coins: Vec<Coin>) -> Vec<cosmos_sdk_proto::cosmos::base::v1beta1::Coin> {
    coins
        .into_iter()
        .map(|item| cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
            denom: item.denom,
            amount: item.amount.to_string(),
        })
        .collect()
}

fn convert_stamp(stamp: Timestamp) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: stamp.seconds() as i64,
        nanos: stamp.nanos() as i32,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    mod basic_allowance {
        use super::*;

        #[test]
        fn basic_allowance() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let grant = app.grant(deps.as_ref());
            let granter = Addr::unchecked("granter");
            let grantee = Addr::unchecked("grantee");
            let spend_limit = coins(100, "asset");
            let expiration = Some(Timestamp::from_seconds(10));
            let basic = grant.basic_allowance(
                granter,
                grantee,
                BasicAllowance {
                    spend_limit,
                    expiration,
                },
            );
            assert_that!(&basic).is_ok();
        }
    }

    mod periodic_allowance {
        use super::*;

        #[test]
        fn periodic_allowance() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let grant = app.grant(deps.as_ref());
            let granter = Addr::unchecked("granter");
            let grantee = Addr::unchecked("grantee");
            let spend_limit = coins(100, "asset");
            let period_spend_limit = vec![];
            let period_can_spend = vec![];
            let expiration = Some(Timestamp::from_seconds(10));
            let basic = Some(BasicAllowance {
                spend_limit,
                expiration,
            });
            let periodic = PeriodicAllowance {
                period: None,
                period_spend_limit,
                period_can_spend,
                period_reset: None,
            };
            let periodic = grant.periodic_allowance(granter, grantee, basic, periodic);
            assert_that!(&periodic).is_ok();
        }
    }
}
