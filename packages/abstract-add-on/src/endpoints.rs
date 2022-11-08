#[macro_export]
macro_rules! export_endpoints {
    ($add_on_const:expr, $add_on_type:ty) => {
        /// Instantiate entrypoint

        #[cfg_attr(entry_point)]
        pub fn instantiate(
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: <$add_on_type as InstantiateEndpoint>::InstantiateMsg,
        ) -> TemplateResult {
            $add_on_const.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        #[cfg_attr(entry_point)]
        pub fn execute(
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: <$add_on_type as ExecuteEndpoint>::ExecuteMsg,
        ) -> TemplateResult {
            $add_on_const.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        #[cfg_attr(entry_point)]
        pub fn query(
            deps: Deps,
            env: Env,
            msg: <$add_on_type as QueryEndpoint>::QueryMsg,
        ) -> StdResult<Binary> {
            $add_on_const.query(deps, env, msg)
        }

        /// Migrate entrypoint
        #[cfg_attr(entry_point)]
        pub fn migrate(
            deps: DepsMut,
            env: Env,
            msg: <$add_on_type as MigrateEndpoint>::MigrateMsg,
        ) -> TemplateResult {
            $add_on_const.migrate(deps, env, msg)
        }

        // Reply entrypoint
        #[cfg_attr(entry_point)]
        pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> TemplateResult {
            $add_on_const.reply(deps, env, msg)
        }
    };
}
