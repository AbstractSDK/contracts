//! # Governance structure object

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Governance types
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceDetails {
    /// A single address is admin
    Monarchy {
        /// The monarch's address
        monarch: String,
    },
    /// Fixed multi-sig governance
    MultiSignature {
        /// Number of signatures
        total_members: u8,
        /// Minimum amounts of votes for a proposal to pass
        threshold_votes: u8,
        /// Member addresses, must be of length total_members
        members: Vec<String>,
    },
    /// A token-weighted governance structure
    TokenWeighted {
        /// Governance token address
        token_addr: String,
    },
}
