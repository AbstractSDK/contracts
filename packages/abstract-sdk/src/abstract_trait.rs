use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
use serde::Serialize;


// pub trait AbstractContract<T>{
//     type ContractError: From<StdError>; 
//     type ExecuteMsg<T>: Serialize;
//     type RequestMsg: Serialize;
//     type RequestHandlerFn: FnOnce(
//         DepsMut,
//         Env,
//         MessageInfo,
//         Self,
//         Self::RequestMsg,
//     ) -> Result<Response, Self::ContractError>;

//     fn handle_request(
//         mut self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         msg: Self::ExecuteMsg,
//         request_handler: Self::RequestHandlerFn,
//     ) -> Result<Response, Self::ContractError> {

//     type BaseInstantiateMsg: Serialize;
    
//     fn instantiate(&self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         msg: Self::BaseInstantiateMsg,
//         module_name: &str,
//         module_version: &str) -> Result<(),Self::ContractError>;

    


    // fn instantiate: FnOnce(
    //     DepsMut,
    //     Env,
    //     MessageInfo,
    //     // Name
    //     &str,
    //     // Version
    //     &str,
    //     Self,
    //     Self::BaseInstantiateMsg,
    // ) -> Result<Response, Self::ContractError>;



}