use abstract_sdk::{Handler, QueryEndpoint};
use cosmwasm_std::{to_binary, Binary, Deps, Empty, Env, StdError, StdResult};

use abstract_os::api::{ApiConfigResponse, BaseQueryMsg, QueryMsg, TradersResponse};

use crate::{state::ApiContract, ApiError};

/// Where we dispatch the queries for the ApiContract
/// These ApiQueryMsg declarations can be found in `abstract_os::common_module::add_on_msg`
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > QueryEndpoint
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    type QueryMsg<Msg> = QueryMsg<Msg>;
    fn query(
        &self,
        deps: Deps,
        env: Env,
        msg: Self::QueryMsg<CustomQueryMsg>,
    ) -> Result<Binary, StdError> {
        match msg {
            QueryMsg::Api(msg) => self.query_handler()?(deps, env, self, msg),
            QueryMsg::Base(msg) => self.base_query(deps, env, msg),
        }
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn base_query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            BaseQueryMsg::Traders { proxy_address } => {
                let traders = self
                    .traders
                    .may_load(deps.storage, deps.api.addr_validate(&proxy_address)?)?
                    .unwrap_or_default();
                to_binary(&TradersResponse {
                    traders: traders.into_iter().collect(),
                })
            }
        }
    }

    fn dapp_config(&self, deps: Deps) -> StdResult<ApiConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        Ok(ApiConfigResponse {
            version_control_address: state.version_control,
            memory_address: state.memory.address,
            dependencies: self
                .dependencies()
                .iter()
                .map(|dep| dep.to_string())
                .collect(),
        })
    }
}
