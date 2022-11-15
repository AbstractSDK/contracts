// /// Execute an action on the OS or over IBC on a remote chain.
// pub trait OsExecute {
//     fn os_execute(&self, deps: Deps, msgs: Vec<CosmosMsg>) -> Result<SubMsg, StdError>;
//     fn os_ibc_execute(
//         &self,
//         deps: Deps,
//         msgs: Vec<abstract_os::ibc_client::ExecuteMsg>,
//     ) -> Result<SubMsg, StdError>;
//     fn os_execute_with_reply(
//         &self,
//         deps: Deps,
//         msgs: Vec<CosmosMsg>,
//         reply_on: ReplyOn,
//         id: u64,
//     ) -> Result<SubMsg, StdError> {
//         let mut msg = self.os_execute(deps, msgs)?;
//         msg.reply_on = reply_on;
//         msg.id = id;
//         Ok(msg)
//     }
// }

// /// easily retrieve the ans_host object from the contract to perform queries
// pub trait AnsHostOperation {
//     /// Load the AnsHost object
//     fn load_ans_host(&self, store: &dyn Storage) -> StdResult<AnsHost>;
//     /// Resolve a query on the ans_host contract
//     /// Use if only 1-2 queries are required
//     /// loads the AnsHost var every call
//     fn resolve<T: Resolve>(
//         &self,
//         deps: Deps,
//         ans_host_entry: &T,
//     ) -> StdResult<<T as Resolve>::Output> {
//         ans_host_entry.resolve(&deps.querier, &self.load_ans_host(deps.storage)?)
//     }
// }

// Call functions on dependencies
// pub trait ModuleDependency {
//     fn dependency_address(&self, deps: Deps, dependency_name: &str) -> StdResult<Addr>;
//     fn call_api_dependency<E: Serialize>(
//         &self,
//         deps: Deps,
//         dependency_name: &str,
//         request_msg: &E,
//     ) -> StdResult<CosmosMsg>;
//     fn call_add_on_dependency<E: Serialize>(
//         &self,
//         deps: Deps,
//         dependency_name: &str,
//         app_msg: &E,
//     ) -> StdResult<CosmosMsg> {
//         let dep_addr = self.dependency_address(deps, dependency_name)?;
//         wasm_execute(dep_addr, &ExecuteMsg::<_, Empty>::App(&app_msg), vec![]).map(Into::into)
//     }
// }
