use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ManagerError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Cannot add two modules of the same kind")]
    ModuleAlreadyAdded {},

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),

    #[error("The name of the proposed module can not have length 0.")]
    InvalidModuleName {},

    #[error("Registering module fails because caller is not module factory")]
    CallerNotFactory {},

    #[error("only the subscription contract can change the OS status")]
    CallerNotSubscriptionContract {},

    #[error("A migratemsg is required when when migrating this module")]
    MsgRequired {},

    #[error("you need a subscription to use this contract")]
    NotSubscribed {},

    #[error("A valid subscriber addr is required")]
    NoSubscriptionAddrProvided {},

    #[error("The provided contract version {0} is lower than the current version {1}")]
    OlderVersion(String, String),

    #[error("The provided API key ({0},{1}) was not found in version control")]
    ApiNotFound(String, String),
}
impl From<semver::Error> for ManagerError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
