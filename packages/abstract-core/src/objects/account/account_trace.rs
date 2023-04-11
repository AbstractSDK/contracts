use std::fmt::Display;

use cosmwasm_std::StdResult;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::{constants::CHAIN_DELIMITER, AbstractError};

use super::ChainId;
const MAX_CHAIN_ID_LENGTH: usize = 20;
const MIN_CHAIN_ID_LENGTH: usize = 3;
const LOCAL: &str = "local";

/// The identifier of chain that triggered the account creation
#[cosmwasm_schema::cw_serde]
pub enum AccountTrace {
    Local,
    // path of the chains that triggered the account creation
    Remote(Vec<ChainId>),
}

impl<'a> PrimaryKey<'a> for &'a AccountTrace {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        match &self {
            AccountTrace::Local => LOCAL.key(),
            AccountTrace::Remote(chain_id) => chain_id.iter().map(|c| c.key()).flatten().collect(),
        }
    }
}

impl KeyDeserialize for &AccountTrace {
    type Output = AccountTrace;
    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        value.drain(0..2);
        Ok(AccountTrace::from(String::from_vec(value)?))
    }
}

impl<'a> Prefixer<'a> for &AccountTrace {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

impl<'a> PrimaryKey<'a> for AccountTrace {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        match &self {
            AccountTrace::Local => LOCAL.key(),
            AccountTrace::Remote(chain_id) => chain_id.iter().map(|c| c.key()).flatten().collect(),
        }
    }
}

impl KeyDeserialize for AccountTrace {
    type Output = AccountTrace;
    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        value.drain(0..2);
        Ok(AccountTrace::from(String::from_vec(value)?))
    }
}

impl<'a> Prefixer<'a> for AccountTrace {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

impl AccountTrace {
    /// verify the formatting of the Account trace chain
    pub fn verify(&self) -> Result<(), AbstractError> {
        match self {
            AccountTrace::Local => Ok(()),
            AccountTrace::Remote(chain_trace) => {
                for chain in chain_trace {
                    if chain.is_empty()
                        || chain.len() < MIN_CHAIN_ID_LENGTH
                        || chain.len() > MAX_CHAIN_ID_LENGTH
                    {
                        return Err(AbstractError::FormattingError {
                            object: "chain-seq".into(),
                            expected: format!(
                                "between {MIN_CHAIN_ID_LENGTH} and {MAX_CHAIN_ID_LENGTH}"
                            ),
                            actual: chain.len().to_string(),
                        });
                    } else if chain
                        .contains(|c: char| !c.is_ascii_alphanumeric() || c.is_ascii_uppercase())
                    {
                        return Err(AbstractError::FormattingError {
                            object: "chain-seq".into(),
                            expected: "alphanumeric characters".into(),
                            actual: chain.clone(),
                        });
                    } else if chain.eq(LOCAL) {
                        return Err(AbstractError::FormattingError {
                            object: "chain-seq".into(),
                            expected: "not 'local'".into(),
                            actual: chain.clone(),
                        });
                    }
                }
                Ok(())
            }
        }
    }

    // pub fn as_str(&self) -> &str {
    //     match self {
    //         AccountTrace::Local => LOCAL,
    //         AccountTrace::Remote(chain_id) => chain_id.join(CHAIN_DELIMITER).as_str(),
    //     }
    // }
}

impl Display for AccountTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountTrace::Local => write!(f, "{}", LOCAL),
            AccountTrace::Remote(chain_id) => write!(f, "{}", chain_id.join(CHAIN_DELIMITER)),
        }
    }
}

impl From<String> for AccountTrace {
    fn from(trace: String) -> Self {
        let acc = if trace == LOCAL {
            Self::Local
        } else {
            Self::Remote(trace.split(CHAIN_DELIMITER).map(|s| s.into()).collect())
        };
        acc.verify().unwrap();
        acc
    }
}
