use cw_orch::{interface, prelude::*};

pub use abstract_core::ibc_client::{
    ExecuteMsg, ExecuteMsgFns as IbcClientExecFns, InstantiateMsg, MigrateMsg, QueryMsg,
    QueryMsgFns as IbcClientQueryFns,
};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;