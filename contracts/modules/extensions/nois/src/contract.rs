use abstract_os::{NOIS, TENDERMINT_STAKING};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use cosmos_nois::{NoisCallback, ReceiverExecuteMsg};

use abstract_extension::{export_endpoints, ExtensionContract};
use semver::Version;

use crate::error::NoisError;
use crate::{handlers, NoisReceiveMsg};
use abstract_os::nois::state::*;
use abstract_os::nois::{
    MigrateMsg, NoisInstantiateMsg, NoisQueryMsg, NoisRequestMsg, StateResponse,
};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type NoisExtension =
    ExtensionContract<NoisError, NoisRequestMsg, NoisInstantiateMsg, NoisQueryMsg, NoisReceiveMsg>;
pub type NoisResult = Result<Response, NoisError>;

const NOIS_EXTENSION: NoisExtension = NoisExtension::new(NOIS, CONTRACT_VERSION)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_receive(handlers::nois_callback_handler);

// Export handlers
#[cfg(not(feature = "library"))]
export_endpoints!(NOIS_EXTENSION, NoisExtension);
