use crate::{state::ApiContract, ApiError};
use abstract_core::api::{ApiConfigResponse, ApiQueryMsg, BaseQueryMsg, QueryMsg, TradersResponse};
use abstract_sdk::{
    base::{endpoints::QueryEndpoint, Handler},
    AbstractSdkError,
};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult};
use std::collections::{BTreeSet, Bound};

pub(crate) const DEFAULT_LIMIT: u8 = 15;
pub(crate) const MAX_LIMIT: u8 = 25;

/// Where we dispatch the queries for the ApiContract
/// These ApiQueryMsg declarations can be found in `abstract_sdk::core::common_module::app_msg`
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg: ApiQueryMsg,
        ReceiveMsg,
    > QueryEndpoint
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    type QueryMsg = QueryMsg<CustomQueryMsg>;
    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, Error> {
        match msg {
            QueryMsg::Module(msg) => self.query_handler()?(deps, env, self, msg),
            QueryMsg::Base(msg) => self.base_query(deps, env, msg),
        }
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    fn base_query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> Result<Binary, Error> {
        match query {
            BaseQueryMsg::Config {} => {
                to_binary(&self.dapp_config(deps).map_err(Error::from)?).map_err(Into::into)
            }
            BaseQueryMsg::Traders {
                proxy_address,
                limit,
                start_after,
            } => {
                let start_after = start_after
                    .map(|s| deps.api.addr_validate(&s))
                    .transpose()?;
                let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

                let proxy_address = deps.api.addr_validate(&proxy_address)?;
                let traders: BTreeSet<Addr> = self
                    .traders
                    .may_load(deps.storage, proxy_address)?
                    .unwrap_or_default();

                let traders_iter = traders.range((
                    start_after.map_or(Bound::Unbounded, Bound::Excluded),
                    Bound::Unbounded,
                ));

                let traders: Vec<Addr> = traders_iter.take(limit).cloned().collect();

                to_binary(&TradersResponse { traders }).map_err(Into::into)
            }
        }
    }

    fn dapp_config(&self, deps: Deps) -> StdResult<ApiConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        Ok(ApiConfigResponse {
            version_control_address: state.version_control,
            ans_host_address: state.ans_host.address,
            dependencies: self
                .dependencies()
                .iter()
                .map(|dep| dep.id.to_string())
                .collect(),
        })
    }
}
