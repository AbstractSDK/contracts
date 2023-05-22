use std::error::Error;
use crate::{AbstractSdkResult, AbstractSdkError};

use cosmwasm_std::{CosmosMsg, wasm_execute, WasmMsg};
use abstract_core::proxy::ExecuteMsg;

#[derive(Debug, PartialEq)]
pub enum AbstractMessage{
	ProxyMessage{
		msgs: Vec<CosmosMsg>,
		proxy: String,
	},
	Raw(CosmosMsg)
}

impl From<WasmMsg> for AbstractMessage{
	fn from(m: WasmMsg) -> Self { 
		AbstractMessage::Raw(CosmosMsg::Wasm(m))
	}
}

impl From<CosmosMsg> for AbstractMessage{
	fn from(m: CosmosMsg) -> Self { 
		AbstractMessage::Raw(m)
	}
}

impl From<AbstractMessage> for CosmosMsg{
	fn from(msg: AbstractMessage) -> Self { 
		match msg{
			AbstractMessage::ProxyMessage{
				msgs,
				proxy
			} => wasm_execute(proxy, &ExecuteMsg::ModuleAction { msgs }, vec![]).unwrap().into(),
			AbstractMessage::Raw(msg) => msg
		}
	}
}

impl AbstractMessage{
	pub fn from_proxy_msg(m: CosmosMsg, proxy: String) -> Self { 
		Self::from_proxy_msgs(vec![m], proxy)
	}

	pub fn from_proxy_msgs(m: Vec<CosmosMsg>, proxy: String) -> Self { 
		AbstractMessage::ProxyMessage{
			msgs: m,
			proxy
		}
	}
}

pub trait AbstractMessageMerge{
	fn merge(&self) -> AbstractSdkResult<Vec<CosmosMsg>>;
}

impl AbstractMessageMerge for Vec<AbstractMessage>{

	fn merge(&self) -> AbstractSdkResult<Vec<CosmosMsg>> {
		
		let mut merged_proxy_messages = Vec::new();
	    let mut all_proxy = "";

	    for message in self {
	        match message {
	            AbstractMessage::ProxyMessage { msgs, proxy } => {
	            	if all_proxy.ne("") && proxy.ne(all_proxy){
	            		return Err(AbstractSdkError::generic_err("Multiple proxy addresses were defined, not possible"))
	            	}
	                merged_proxy_messages.extend(msgs.clone());
	                all_proxy = proxy;
	            }
	            AbstractMessage::Raw(cosmos_msg) => {
	                merged_proxy_messages.push(cosmos_msg.clone());
	            }
	        }
	    }
	    Ok(merged_proxy_messages)
	}
}

pub trait ConvertToAbstractMessage{
	fn into_abstract_messages(self) -> AbstractSdkResult<Vec<AbstractMessage>>;
}

impl<T: Error> ConvertToAbstractMessage for Result<Vec<CosmosMsg>, T>{

	fn into_abstract_messages(self) -> AbstractSdkResult<Vec<AbstractMessage>> {
		
		match self{
			Ok(messages) => {
				Ok(messages.iter().map(|msg| msg.clone().into()).collect())
			},
			Err(e) => Err(AbstractSdkError::generic_err(e.to_string()))
		}
	}
}


