use super::contract_base::{
    AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
    MigrateHandlerFn, QueryHandlerFn, ReceiveHandlerFn,
};
use crate::base::contract_base::{ContractMetadata, ContractName, VersionString};
use crate::base::ReplyHandlerFn;
use crate::error::EndpointError;
use crate::{SdkError, SdkResult};
use abstract_os::objects::dependency::StaticDependency;
use cosmwasm_std::Storage;
use cw2::ContractVersion;

pub trait Handler
where
    Self: Sized + 'static,
{
    type Error: From<SdkError>;
    type CustomExecMsg;
    type CustomInitMsg;
    type CustomQueryMsg;
    type CustomMigrateMsg;
    type ReceiveMsg;
    #[allow(clippy::type_complexity)]
    fn contract(
        &self,
    ) -> &AbstractContract<
        Self,
        Self::Error,
        Self::CustomExecMsg,
        Self::CustomInitMsg,
        Self::CustomQueryMsg,
        Self::CustomMigrateMsg,
        Self::ReceiveMsg,
    >;

    fn stored_version(&self, store: &dyn Storage) -> SdkResult<ContractVersion> {
        let contract = self.contract();
        contract.version.load(store).map_err(Into::into)
    }

    fn info(&self) -> (ContractName, VersionString, ContractMetadata) {
        let contract = self.contract();
        contract.info
    }

    fn dependencies(&self) -> &'static [StaticDependency] {
        let contract = self.contract();
        contract.dependencies
    }
    // Execute
    fn maybe_execute_handler(
        &self,
    ) -> Option<ExecuteHandlerFn<Self, Self::CustomExecMsg, Self::Error>> {
        let contract = self.contract();
        contract.execute_handler
    }
    fn execute_handler(
        &self,
    ) -> SdkResult<ExecuteHandlerFn<Self, Self::CustomExecMsg, Self::Error>> {
        let Some(handler) = self.maybe_execute_handler() else {
            return Err(SdkError::MissingHandler{ endpoint: "execution handler".to_string()})
        };
        Ok(handler)
    }

    // Instantiate
    fn maybe_instantiate_handler(
        &self,
    ) -> Option<InstantiateHandlerFn<Self, Self::CustomInitMsg, Self::Error>> {
        let contract = self.contract();
        contract.instantiate_handler
    }
    fn instantiate_handler(
        &self,
    ) -> SdkResult<InstantiateHandlerFn<Self, Self::CustomInitMsg, Self::Error>> {
        let Some(handler) = self.maybe_instantiate_handler() else {
            return Err(SdkError::MissingHandler{ endpoint: "instantiate".to_string()})
        };
        Ok(handler)
    }

    // Query
    fn maybe_query_handler(
        &self,
    ) -> Option<QueryHandlerFn<Self, Self::CustomQueryMsg, Self::Error>> {
        let contract = self.contract();
        contract.query_handler
    }
    fn query_handler(&self) -> SdkResult<QueryHandlerFn<Self, Self::CustomQueryMsg, Self::Error>> {
        let Some(handler) = self.maybe_query_handler() else {
            return Err(SdkError::MissingHandler{ endpoint: "query".to_string()})
        };
        Ok(handler)
    }

    // Migrate
    fn maybe_migrate_handler(
        &self,
    ) -> Option<MigrateHandlerFn<Self, Self::CustomMigrateMsg, Self::Error>> {
        let contract = self.contract();
        contract.migrate_handler
    }
    fn migrate_handler(
        &self,
    ) -> SdkResult<MigrateHandlerFn<Self, Self::CustomMigrateMsg, Self::Error>> {
        let Some(handler) = self.maybe_migrate_handler() else {
            return Err(SdkError::MissingHandler{ endpoint: "migrate".to_string()})
        };
        Ok(handler)
    }

    // Receive
    fn maybe_receive_handler(
        &self,
    ) -> Option<ReceiveHandlerFn<Self, Self::ReceiveMsg, Self::Error>> {
        let contract = self.contract();
        contract.receive_handler
    }
    fn receive_handler(&self) -> SdkResult<ReceiveHandlerFn<Self, Self::ReceiveMsg, Self::Error>> {
        let Some(handler) = self.maybe_receive_handler() else {
            return Err(SdkError::MissingHandler{ endpoint: "receive".to_string()})
        };
        Ok(handler)
    }
    fn maybe_ibc_callback_handler(
        &self,
        id: &str,
    ) -> Option<IbcCallbackHandlerFn<Self, Self::Error>> {
        let contract = self.contract();
        for ibc_callback_handler in contract.ibc_callback_handlers {
            if ibc_callback_handler.0 == id {
                return Some(ibc_callback_handler.1);
            }
        }
        None
    }

    fn maybe_reply_handler(&self, id: u64) -> Option<ReplyHandlerFn<Self, Self::Error>> {
        let contract = self.contract();
        for reply_handlers in contract.reply_handlers {
            for handler in reply_handlers {
                if handler.0 == id {
                    return Some(handler.1);
                }
            }
        }
        None
    }

    fn reply_handler(&self, id: u64) -> SdkResult<ReplyHandlerFn<Self, Self::Error>> {
        let Some(handler) = self.maybe_reply_handler(id) else {
            return Err(SdkError::MissingHandler{ endpoint: format! {"reply with id {id}"}})
        };
        Ok(handler)
    }
}
