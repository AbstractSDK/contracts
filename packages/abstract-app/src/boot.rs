use std::{fmt::Debug, marker::PhantomData};

use abstract_sdk::base::{InstantiateEndpoint, ExecuteEndpoint};
use boot_core::prelude::*;
use serde::Serialize;

pub trait BaseExecute<Chain: BootEnvironment>: ExecuteEndpoint {
    type CustomExecuteMsg: Into<<Self as ExecuteEndpoint>::ExecuteMsg>;
    type BaseExecuteMsg: Into<<Self as ExecuteEndpoint>::ExecuteMsg>;
        fn execute<'a>(
            &self,
            execute_msg: &'a Self::CustomExecuteMsg,
            coins: Option<&[cosmwasm_std::Coin]>,
        ) -> Result<<Chain>::Response, BootError> {
            let exec_msg: <Self as ExecuteEndpoint>::ExecuteMsg = execute_msg.into();
            self.as_instance().execute(&exec_msg, coins)
        }
        fn configure<'a>(
            &self,
            base_execute_msg: &'a Self::BaseExecuteMsg,
            coins: Option<&[cosmwasm_std::Coin]>,
        ) -> Result<<Chain>::Response, BootError> {
            let exec_msg: <Self as ExecuteEndpoint>::ExecuteMsg = base_execute_msg.into();
            self.as_instance().execute(&exec_msg, coins)
        }
}

pub struct Booter<
    Chain: BootEnvironment,
    ExecuteMsg,
    InstantiateMsg,
    QueryMsg,
    MigrateMsg,
> {
    pub contract: Contract<Chain>,
    pub _phantom_data: PhantomData<(ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg)>,
}

impl<
        Chain: BootEnvironment,
        ExecuteMsg,
        InstantiateMsg,
        QueryMsg,
        MigrateMsg,
    > boot_core::interface::ContractInstance<Chain>
    for Booter<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg>
{
    fn as_instance(&self) -> &Contract<Chain> {
        &self.contract
    }
    fn as_instance_mut(&mut self) -> &mut Contract<Chain> {
        &mut self.contract
    }
}



impl<Chain: BootEnvironment,
ExecuteMsg: Serialize + Debug,
InstantiateMsg,
QueryMsg,
MigrateMsg > BootExecute<Chain> for Booter<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg> {
    type ExecuteMsg = ExecuteMsg;

    fn execute<'a>(
        &self,
        execute_msg: &'a Self::ExecuteMsg,
        coins: Option<&[cosmwasm_std::Coin]>,
    ) -> Result<<Chain>::Response, BootError> {
        let exec_msg = abstract_os::app::ExecuteMsg::<_>::Booter(execute_msg);
        self.as_instance().execute(&exec_msg, coins)
    }
}

impl<Chain: BootEnvironment,
ExecuteMsg,
InstantiateMsg,
QueryMsg: Serialize + Debug,
MigrateMsg > BootQuery<Chain> for Booter<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg> {
    type QueryMsg = QueryMsg;

    fn query<G: Serialize + serde::de::DeserializeOwned>(&self, query_msg: &Self::QueryMsg) -> Result<G, BootError> {
        let query = abstract_os::app::QueryMsg::Booter(query_msg);
        self.as_instance().query(&query)
    }
}

// impl<Chain: BootEnvironment,
// ExecuteMsg,
// InstantiateMsg: Serialize + Debug,
// QueryMsg,
// MigrateMsg > BootInstantiate<Chain> for Booter<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg> {
//     type InstantiateMsg = <Self as InstantiateEndpoint>::InstantiateMsg;

//     fn instantiate(
//         &self,
//         instantiate_msg: &Self::InstantiateMsg,
//         admin: Option<&cosmwasm_std::Addr>,
//         coins: Option<&[cosmwasm_std::Coin]>,
//     ) -> Result<<Chain>::Response, BootError> {
//         let init_msg = abstract_os::app::InstantiateMsg
//     }
// }
