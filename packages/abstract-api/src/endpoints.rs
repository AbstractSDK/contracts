mod execute;
mod ibc_callback;
pub mod instantiate;
mod query;
mod receive;

#[macro_export]
macro_rules! export_endpoints {
    ($add_on_const:expr, $add_on_type:ty) => {
        /// Instantiate entrypoint
        #[cosmwasm_std::entry_point]
        pub fn instantiate(
            deps: cosmwasm_std::DepsMut,
            env: cosmwasm_std::Env,
            info: cosmwasm_std::MessageInfo,
            msg: <$add_on_type as abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<cosmwasm_std::Response, <$add_on_type as abstract_sdk::base::Handler>::Error> {
            use abstract_sdk::base::InstantiateEndpoint;
            $add_on_const.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        #[cosmwasm_std::entry_point]
        pub fn execute(
            deps: cosmwasm_std::DepsMut,
            env: cosmwasm_std::Env,
            info: cosmwasm_std::MessageInfo,
            msg: <$add_on_type as abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<cosmwasm_std::Response, <$add_on_type as abstract_sdk::base::Handler>::Error> {
            use abstract_sdk::base::ExecuteEndpoint;
            $add_on_const.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        #[cosmwasm_std::entry_point]
        pub fn query(
            deps: cosmwasm_std::Deps,
            env: cosmwasm_std::Env,
            msg: <$add_on_type as abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> cosmwasm_std::StdResult<cosmwasm_std::Binary> {
            use abstract_sdk::base::QueryEndpoint;
            $add_on_const.query(deps, env, msg)
        }
    };
}
