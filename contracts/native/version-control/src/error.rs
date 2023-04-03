use abstract_core::objects::AccountId;
use abstract_core::AbstractError;
use abstract_sdk::core::objects::module::ModuleInfo;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VCError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Module {0} does not have a stored module reference")]
    ModuleNotFound(ModuleInfo),

    #[error("Module {0} cannot be updated")]
    NotUpdateableModule(ModuleInfo),

    #[error("Account ID {} is not in version control register", id)]
    MissingAccountId { id: AccountId },

    #[error("Namespace {} is not in version control register", namespace)]
    MissingNamespace { namespace: String },

    #[error("Account owner mismatch sender: {}, owner: {}", sender, owner)]
    AccountOwnerMismatch { sender: String, owner: String },

    #[error("Namespace {} is already occupied by {}", namespace, id)]
    NamespaceOccupied { namespace: String, id: AccountId },

    #[error("Exceeds namespace limit: {}, current: {}", limit, current)]
    ExceedsNamespaceLimit { limit: usize, current: usize },
}
impl From<cw_semver::Error> for VCError {
    fn from(err: cw_semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
