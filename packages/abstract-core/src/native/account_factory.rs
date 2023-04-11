//! # Account Factory
//!
//! `abstract_core::account_factory` handles Account creation and registration.
//!
//! ## Description
//! The Account factory instantiates a new Account instance and registers it with the [`crate::version_control`] contract. It then forwards the payment to the main account's subscription module.  
//! ## Create a new Account
//! Call [`ExecuteMsg::CreateAccount`] on this contract along with a [`crate::objects::gov_type`] and name you'd like to display on your Account.
//!
pub mod state {
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    use serde::{Deserialize, Serialize};

    use crate::objects::{
        account::{AccountSequence, AccountTrace},
        common_namespace::ADMIN_NAMESPACE,
        AccountId,
    };

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_contract: Addr,
        pub ans_host_contract: Addr,
        pub module_factory_address: Addr,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Context {
        pub account_manager_address: Option<Addr>,
        pub account_id: AccountId,
    }

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
    pub const CONTEXT: Item<Context> = Item::new("\u{0}{6}context");
    /// Next account id for a specific origin.
    pub const ACCOUNT_SEQUENCES: Map<&AccountTrace, AccountSequence> = Map::new("acc_seq");
}

use crate::objects::{
    account::{AccountSequence, AccountTrace},
    gov_type::GovernanceDetails,
};
use cosmwasm_schema::QueryResponses;

/// Msg used on instantiation
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Version control contract used to get code-ids and register Account
    pub version_control_address: String,
    /// AnsHost contract
    pub ans_host_address: String,
    /// AnsHosts of module factory. Used for instantiating manager.
    pub module_factory_address: String,
}

/// Execute function entrypoint.
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
pub enum ExecuteMsg {
    /// Update config
    UpdateConfig {
        // New admin
        admin: Option<String>,
        // New ans_host contract
        ans_host_contract: Option<String>,
        // New version control contract
        version_control_contract: Option<String>,
        // New module factory contract
        module_factory_address: Option<String>,
    },
    /// Creates the core contracts and sets the permissions.
    /// [`crate::manager`] and [`crate::proxy`]
    CreateAccount {
        // Governance details
        governance: GovernanceDetails<String>,
        name: String,
        description: Option<String>,
        link: Option<String>,
        /// Creator chain of the account. AccountTrace::Local if not specified.
        origin: Option<AccountTrace>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    /// Retrieve the sequence numbers for new accounts from different origins.
    #[returns(SequencesResponse)]
    Sequences {
        filter: Option<AccountTraceFilter>,
        start_after: Option<AccountTrace>,
        limit: Option<u8>,
    },
    #[returns(SequenceResponse)]
    Sequence { origin: AccountTrace },
}

// Response for Config query
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub ans_host_contract: String,
    pub version_control_contract: String,
    pub module_factory_address: String,
}

/// Sequence numbers for each origin.
#[cosmwasm_schema::cw_serde]
pub struct SequencesResponse {
    pub sequences: Vec<(AccountTrace, AccountSequence)>,
}

#[cosmwasm_schema::cw_serde]
pub struct SequenceResponse {
    pub sequence: AccountSequence,
}

/// We currently take no arguments for migrations
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

/// UNUSED - stub for future use
#[cosmwasm_schema::cw_serde]
pub struct AccountTraceFilter {}
