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

use crate::{features::{AccountIdentification, AbstractNameService}, AbstractSdkResult};
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
    pub fn basic(
        &self,
        granter: Addr,
        grantee: Addr,
        spend_limit: Vec<Coin>,
        expiration: Option<Timestamp>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let expiry = Some(expiration).map(|stamp| {
            convert_stamp(stamp.unwrap())
        });

        let allowance = Any {
            type_url: "/cosmos.feegrant.v1beta1.BasicAllowance".to_string(),
            value: prost::Message::encode_to_vec(
                &cosmos_sdk_proto::cosmos::feegrant::v1beta1::BasicAllowance {
                    spend_limit: spend_limit
                        .into_iter()
                        .map(|item| cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                            denom: item.denom,
                            amount: item.amount.to_string(),
                        })
                        .collect(),
                    expiration: expiry,
                },
            ),
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
    // use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    mod transfer_coins {
        use super::*;

        #[test]
        fn basic_allowance() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let grant = app.grant(deps.as_ref());
            let granter = Addr::unchecked("granter");
            let grantee = Addr::unchecked("grantee");
            let spend_limit = coins(100, "asset");
            let expiration = Timestamp::from_seconds(10);
            let basic_msg = grant.basic(granter, grantee, spend_limit, Some(expiration));
            assert_that!(&basic_msg).is_ok();
        }
    }
}
