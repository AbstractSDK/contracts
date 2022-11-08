#[macro_export]
macro_rules! export_endpoints {
    ($add_on_const:expr, $add_on_type:ty) => {
        use abstract_sdk::{ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint, ReplyEndpoint, MigrateEndpoint, Handler};
        use cosmwasm_std::{
            entry_point, Reply, StdResult, Env, MessageInfo, Deps, DepsMut, Response,
        };
        /// Instantiate entrypoint
        #[entry_point]
        pub fn instantiate(
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: <$add_on_type as InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<Response,<$add_on_type as Handler>::Error> {
            $add_on_const.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        #[entry_point]
        pub fn execute(
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: <$add_on_type as ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<Response,<$add_on_type as Handler>::Error> {
            $add_on_const.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        #[entry_point]
        pub fn query(
            deps: Deps,
            env: Env,
            msg: <$add_on_type as QueryEndpoint>::QueryMsg,
        ) -> StdResult<Binary> {
            $add_on_const.query(deps, env, msg)
        }

        /// Migrate entrypoint
        #[entry_point]
        pub fn migrate(
            deps: DepsMut,
            env: Env,
            msg: <$add_on_type as MigrateEndpoint>::MigrateMsg,
        ) -> Result<Response,<$add_on_type as Handler>::Error> {
            $add_on_const.migrate(deps, env, msg)
        }

        // Reply entrypoint
        #[entry_point]
        pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response,<$add_on_type as Handler>::Error> {
            $add_on_const.reply(deps, env, msg)
        }
    };
}
