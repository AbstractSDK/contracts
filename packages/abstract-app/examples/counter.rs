pub use abstract_core::app;
    use abstract_interface::AppDeployer;
    pub use cosmwasm_std::testing::*;
    use cosmwasm_std::{from_binary, to_binary, Addr, Response, StdError};
    use cw_orch::prelude::*;

    pub type CounterResult = Result<(), CounterError>;

    #[cosmwasm_schema::cw_serde]
    pub struct CounterInitMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct CounterExecMsg;

    impl app::AppExecuteMsg for CounterExecMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct CounterQueryMsg;

    impl app::AppQueryMsg for CounterQueryMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct CounterMigrateMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct CounterReceiveMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct CounterSudoMsg;

    use abstract_app::{App, AppError};
    use abstract_core::{module_factory::ContextResponse, version_control::AccountBase};
    use abstract_sdk::{base::InstantiateEndpoint, AbstractSdkError};
    use abstract_testing::prelude::{
        CounterDeps, CounterQuerierBuilder, TEST_ANS_HOST, TEST_MANAGER, TEST_MODULE_FACTORY,
        TEST_MODULE_ID, TEST_PROXY, TEST_VERSION,
    };
    use thiserror::Error;

    // ANCHOR: error
    #[derive(Error, Debug, PartialEq)]
    pub enum CounterError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error("{0}")]
        DappError(#[from] AppError),

        #[error("{0}")]
        Abstract(#[from] abstract_core::AbstractError),

        #[error("{0}")]
        AbstractSdk(#[from] AbstractSdkError),
    }
    // ANCHOR_END: error

    // ANCHOR: counter_app
    pub type CounterApp = App<
        CounterError,
        CounterInitMsg,
        CounterExecMsg,
        CounterQueryMsg,
        CounterMigrateMsg,
        CounterReceiveMsg,
        CounterSudoMsg,
    >;
    // ANCHOR_END: counter_app

    pub const COUNTER_APP: CounterApp =
        CounterApp::new(COUNTER_ID, MODULE_VERSION, None);

    // ANCHOR: handlers
    // ANCHOR: new
    pub const COUNTER_APP: CounterApp = CounterApp::new(COUNTER_ID, MODULE_VERSION, None)
        // ANCHOR_END: new
        .with_instantiate(handlers::instantiate)
        .with_execute(handlers::execute)
        .with_query(handlers::query)
        .with_sudo(handlers::sudo)
        .with_receive(handlers::receive)
        .with_replies(&[(1u64,handlers::reply)])
        .with_migrate(handlers::migrate);
    // ANCHOR_END: handlers

    // ANCHOR: export
    abstract_app::export_endpoints!(COUNTER_APP, CounterApp);
    // ANCHOR_END: export

mod handlers {
    fn instantiate = |_, _, _, _, _| Ok(Response::new().set_data("counter_init".as_bytes())); 
    fn execute = |_, _, _, _, _| Ok(Response::new().set_data("counter_exec".as_bytes())); 
    fn query = |_, _, _, _| to_binary("counter_query").map_err(Into::into);
    fn sudo = |_, _, _, _| Ok(Response::new().set_data("counter_sudo".as_bytes()));
    fn receive = |_, _, _, _, _| Ok(Response::new().set_data("counter_receive".as_bytes()));
    fn reply = |_, _, _, msg| {
        Ok(Response::new().set_data(msg.result.unwrap().data.unwrap()))
    };
    fn migrate = |_, _, _, _| Ok(Response::new().set_data("counter_migrate".as_bytes()));
}