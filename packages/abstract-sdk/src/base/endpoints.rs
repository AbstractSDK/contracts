mod execute;
mod ibc_callback;
mod instantiate;
pub(crate) mod migrate;
mod query;
mod receive;
mod reply;

// Provide endpoints under ::base::traits::
pub use {
    execute::ExecuteEndpoint, ibc_callback::IbcCallbackEndpoint, instantiate::InstantiateEndpoint,
    migrate::MigrateEndpoint, query::QueryEndpoint, receive::ReceiveEndpoint, reply::ReplyEndpoint,
};