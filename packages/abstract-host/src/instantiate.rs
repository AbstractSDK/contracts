use abstract_os::host::BaseInstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, StdResult};
use serde::{de::DeserializeOwned, Serialize};

use abstract_sdk::memory::Memory;

use crate::state::{HostContract, HostState};

use cw2::set_contract_version;

impl<'a, T: Serialize + DeserializeOwned> HostContract<'a, T> {
    /// Instantiate the API
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: BaseInstantiateMsg,
        module_name: &str,
        module_version: &str,
        _api_dependencies: Vec<String>,
    ) -> StdResult<Self> {
        let api = Self::default();
        let memory = Memory {
            address: deps.api.addr_validate(&msg.memory_address)?,
        };

        // Base state
        let state = HostState {
            memory,
            cw1_code_id: msg.cw1_code_id,
        };

        set_contract_version(deps.storage, module_name, module_version)?;
        api.base_state.save(deps.storage, &state)?;

        Ok(api)
    }
}
