use abstract_core::objects::validation::ValidationError;
use abstract_core::AbstractError;
use abstract_sdk::core::objects::module::ModuleInfo;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ManagerError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Validation(#[from] ValidationError),

    #[error("{0}")]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error("Module with id: {0} is already installed")]
    ModuleAlreadyInstalled(String),

    #[error("Cannot remove module because {0:?} depend(s) on it.")]
    ModuleHasDependents(Vec<String>),

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),

    #[error("The name of the proposed module can not have length 0.")]
    InvalidModuleName {},

    #[error("Registering module fails because caller is not module factory")]
    CallerNotModuleFactory {},

    #[error("A migrate msg is required when when migrating this module")]
    MsgRequired {},

    #[error("{0} not upgradable")]
    NotUpgradeable(ModuleInfo),

    #[error("Cannot migrate {} twice", module_id)]
    DuplicateModuleMigration { module_id: String },

    #[error("Your account is currently suspended")]
    AccountSuspended {},

    #[error("The provided contract version {0} is lower than the current version {1}")]
    OlderVersion(String, String),

    #[error("The provided module {0} was not found")]
    ModuleNotFound(String),

    #[error("Module {module_id} with version {version} does not fit requirement {comp}, post_migration: {post_migration}")]
    VersionRequirementNotMet {
        module_id: String,
        version: String,
        comp: String,
        post_migration: bool,
    },

    #[error("module {0} is a dependency of {1} and is not installed.")]
    DependencyNotMet(String, String),

    #[error("The provided module {0} has an invalid module reference.")]
    InvalidReference(ModuleInfo),

    #[error("Cannot remove proxy")]
    CannotRemoveProxy {},

    #[error("No updates were included")]
    NoUpdates {},

    #[error("invalid configuration action, {}", error)]
    InvalidConfigAction { error: StdError },

    #[error("Must use SetOwner to change owner")]
    MustUseSetOwner {},
}
