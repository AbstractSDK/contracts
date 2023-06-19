#[macro_export]
/// Groups code that is needed on every app.
/// This registers the types for safety when using Messages. 
/// Example generated code : 
/// pub type InstantiateMsg =
///     <App as abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
/// pub type ExecuteMsg = <App as abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
/// pub type QueryMsg = <App as abstract_sdk::base::QueryEndpoint>::QueryMsg;
/// pub type MigrateMsg = <App as abstract_sdk::base::MigrateEndpoint>::MigrateMsg;
/// This allows users to directly import the right message types when using the module as a library

/// This is also used to indicate that The Query And Execute messages or used as app messages
/// Example generated code: 
/// impl abstract_core::app::AppExecuteMsg for AppExecuteMsg {}
/// impl abstract_core::app::AppQueryMsg for AppQueryMsg {}
/// This is internal to abstract and allows the Query and Execute Msgs to be used as app entry_point messages
macro_rules! app_msg_types {
    ($app_type:ty, $app_execute_msg: ty, $app_query_msg: ty) => {
        /// Abstract App instantiate msg
        pub type InstantiateMsg =
            <$app_type as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
        pub type ExecuteMsg = <$app_type as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
        pub type QueryMsg = <$app_type as ::abstract_sdk::base::QueryEndpoint>::QueryMsg;
        pub type MigrateMsg = <$app_type as ::abstract_sdk::base::MigrateEndpoint>::MigrateMsg;

        impl ::abstract_core::app::AppExecuteMsg for $app_execute_msg {}
        impl ::abstract_core::app::AppQueryMsg for $app_query_msg {}
    };
}
