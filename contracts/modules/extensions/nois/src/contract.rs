use abstract_os::NOIS;
use cosmwasm_std::Response;

use abstract_extension::{export_endpoints, ExtensionContract};

use crate::error::NoisError;
use crate::{handlers, NoisReceiveMsg};

use abstract_os::nois::{NoisInstantiateMsg, NoisQueryMsg, NoisRequestMsg};

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
