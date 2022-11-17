// pub mod execute;
// pub mod instantiate;
// pub mod migrate;
// pub mod query;
pub mod reply;
pub mod execute;
pub mod query;
pub mod instantiate;
pub mod receive;

pub use crate::handlers::{
    execute::execute_handler, instantiate::instantiate_handler,
    query::query_handler, reply::*, receive::receive_cw20,
};
