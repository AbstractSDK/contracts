use std::{fmt::Display, str::FromStr};

use cosmwasm_std::{StdError, StdResult};
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
            AccountTrace::Remote(chain_id) => chain_id.iter().flat_map(|c| c.key()).collect(),
        }
    }
}

impl KeyDeserialize for &AccountTrace {
    type Output = AccountTrace;
    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
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
            AccountTrace::Remote(chain_id) => chain_id.iter().flat_map(|c| c.key()).collect(),
        }
    }
}

impl KeyDeserialize for AccountTrace {
    type Output = AccountTrace;
    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
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
    /// **No verification is done here**
    ///
    /// **only use this for deserialization**
    fn from(trace: String) -> Self {
        let acc = if trace == LOCAL {
            Self::Local
        } else {
            Self::Remote(trace.split(CHAIN_DELIMITER).map(|s| s.into()).collect())
        };
        acc
    }
}

impl FromStr for AccountTrace {
    type Err = StdError;
    fn from_str(trace: &str) -> Result<Self, Self::Err> {
        let acc = if trace == LOCAL {
            Self::Local
        } else {
            Self::Remote(trace.split(CHAIN_DELIMITER).map(|s| s.into()).collect())
        };
        acc.verify()
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        Ok(acc)
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    mod format {
        use super::*;

        #[test]
        fn local_works() {
            let trace = AccountTrace::from_str(LOCAL).unwrap();
            assert_eq!(trace, AccountTrace::Local);
        }

        #[test]
        fn remote_works() {
            let trace = AccountTrace::from_str("bitcoin").unwrap();
            assert_eq!(trace, AccountTrace::Remote(vec!["bitcoin".to_string()]));
        }

        #[test]
        fn remote_multi_works() {
            let trace = AccountTrace::from("bitcoin>ethereum".to_string());
            assert_eq!(
                trace,
                AccountTrace::Remote(vec!["bitcoin".to_string(), "ethereum".to_string()])
            );
        }

        #[test]
        fn remote_multi_multi_works() {
            let trace = AccountTrace::from_str("bitcoin>ethereum>cosmos").unwrap();
            assert_eq!(
                trace,
                AccountTrace::Remote(vec![
                    "bitcoin".to_string(),
                    "ethereum".to_string(),
                    "cosmos".to_string()
                ])
            );
        }

        // now test failures
        #[test]
        fn local_empty_fails() {
            AccountTrace::from_str("").unwrap_err();
        }

        #[test]
        fn local_too_short_fails() {
            AccountTrace::from_str("a").unwrap_err();
        }

        #[test]
        fn local_too_long_fails() {
            AccountTrace::from_str(&"a".repeat(MAX_CHAIN_ID_LENGTH + 1)).unwrap_err();
        }

        #[test]
        fn local_uppercase_fails() {
            AccountTrace::from_str("AAAAA").unwrap_err();
        }

        #[test]
        fn local_non_alphanumeric_fails() {
            AccountTrace::from_str("a!aoeuoau").unwrap_err();
        }
    }

    mod key {
        use super::*;

        fn mock_key() -> AccountTrace {
            AccountTrace::Remote(vec!["bitcoin".to_string()])
        }

        #[test]
        fn storage_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<&AccountTrace, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key, &42069).unwrap();

            assert_eq!(map.load(deps.as_ref().storage, &key).unwrap(), 42069);

            let items = map
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 1);
            assert_eq!(items[0], (key, 42069));
        }

        #[test]
        fn composite_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<(&AccountTrace, Addr), u64> = Map::new("map");

            map.save(
                deps.as_mut().storage,
                (&key, Addr::unchecked("larry")),
                &42069,
            )
            .unwrap();

            map.save(
                deps.as_mut().storage,
                (&key, Addr::unchecked("jake")),
                &69420,
            )
            .unwrap();

            let items = map
                .prefix(&key)
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 2);
            assert_eq!(items[0], (Addr::unchecked("jake"), 69420));
            assert_eq!(items[1], (Addr::unchecked("larry"), 42069));
        }
    }
}
