use std::fmt::Display;

use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::{Item, Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::AbstractError;

const MAX_CHAIN_ID_LENGTH: usize = 20;
const MIN_CHAIN_ID_LENGTH: usize = 3;
const LOCAL: &str = "local";
pub const TEST_ACCOUNT_ID: AccountId = AccountId::const_new(1, AccountOrigin::Local);

/// Identifier for a chain
/// Example: "juno", "terra", "osmosis", ...
pub type ChainId = String;

/// Unique identifier for an account.
/// On each chain this is unique, but not across chains.
#[cosmwasm_schema::cw_serde]
pub struct AccountId {
    /// Chain id of the chain that triggered the account creation
    /// `AccountOrigin::Local` if the account was created locally
    origin: AccountOrigin,
    /// Unique identifier for the account
    /// Account factory sequence number for the origin chain
    id: u32,
}

impl Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.origin, self.id)
    }
}

impl AccountId {
    pub fn new(id: u32, origin: AccountOrigin) -> Result<Self, AbstractError> {
        origin.verify()?;
        Ok(Self { id, origin })
    }
    // used internally for testing
    pub(crate) const fn const_new(id: u32, origin: AccountOrigin) -> Self {
        Self { id, origin }
    }
}

/// The identifier of chain that triggered the account creation
#[cosmwasm_schema::cw_serde]
pub enum AccountOrigin {
    Local,
    Remote(ChainId),
}

impl<'a> PrimaryKey<'a> for AccountOrigin {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        match &self {
            AccountOrigin::Local => LOCAL.key(),
            AccountOrigin::Remote(chain_id) => chain_id.key(),
        }
    }
}

impl KeyDeserialize for AccountOrigin {
    type Output = AccountOrigin;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        value.drain(0..2);
        Ok(AccountOrigin::from(String::from_vec(value)?))
    }
}

impl<'a> Prefixer<'a> for AccountOrigin {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Ref(self.as_str().as_bytes())]
    }
}

impl AccountOrigin {
    /// verify the formatting of the Account origin chain
    pub fn verify(&self) -> Result<(), AbstractError> {
        match self {
            AccountOrigin::Local => Ok(()),
            AccountOrigin::Remote(chain_id) => {
                if chain_id.is_empty()
                    || chain_id.len() < MIN_CHAIN_ID_LENGTH
                    || chain_id.len() > MAX_CHAIN_ID_LENGTH
                {
                    Err(AbstractError::FormattingError {
                        object: "chain-id".into(),
                        expected: format!(
                            "between {MIN_CHAIN_ID_LENGTH} and {MAX_CHAIN_ID_LENGTH}"
                        ),
                        actual: chain_id.len().to_string(),
                    })
                } else if chain_id
                    .contains(|c: char| !c.is_ascii_alphanumeric() || c.is_ascii_uppercase())
                {
                    Err(AbstractError::FormattingError {
                        object: "chain-id".into(),
                        expected: "alphanumeric characters".into(),
                        actual: chain_id.clone(),
                    })
                } else if chain_id.eq(LOCAL) {
                    Err(AbstractError::FormattingError {
                        object: "chain-id".into(),
                        expected: "not 'local'".into(),
                        actual: chain_id.clone(),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AccountOrigin::Local => LOCAL,
            AccountOrigin::Remote(chain_id) => chain_id.as_str(),
        }
    }
}

impl Display for AccountOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountOrigin::Local => write!(f, "{}", LOCAL),
            AccountOrigin::Remote(chain_id) => write!(f, "{}", chain_id),
        }
    }
}

impl From<ChainId> for AccountOrigin {
    fn from(chain_id: ChainId) -> Self {
        let acc = if chain_id == LOCAL {
            Self::Local
        } else {
            Self::Remote(chain_id)
        };
        acc.verify().unwrap();
        acc
    }
}

/// Account Id storage key
pub const ACCOUNT_ID: Item<AccountId> = Item::new("acc_id");

impl<'a> PrimaryKey<'a> for &AccountId {
    type Prefix = u32;

    type SubPrefix = ();

    type Suffix = AccountOrigin;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.origin.key();
        keys.extend(self.id.key());
        keys
    }
}

impl<'a> Prefixer<'a> for &AccountId {
    fn prefix(&self) -> Vec<Key> {
        let mut res = self.id.prefix();
        res.extend(self.origin.prefix().into_iter());
        res
    }
}

impl KeyDeserialize for &AccountId {
    type Output = AccountId;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tu = value.split_off(2);
        let t_len = parse_length(&value)?;
        let u = tu.split_off(t_len);

        Ok(AccountId {
            id: u32::from_vec(tu)?,
            origin: AccountOrigin::from(String::from_vec(u)?),
        })
    }
}

#[inline(always)]
fn parse_length(value: &[u8]) -> StdResult<usize> {
    Ok(u16::from_be_bytes(
        value
            .try_into()
            .map_err(|_| StdError::generic_err("Could not read 2 byte length"))?,
    )
    .into())
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    mod key {
        use super::*;

        fn mock_key() -> AccountId {
            AccountId {
                id: 1,
                origin: AccountOrigin::Remote("bitcoin".to_string()),
            }
        }

        fn mock_keys() -> (AccountId, AccountId, AccountId) {
            (
                AccountId {
                    id: 1,
                    origin: AccountOrigin::Local,
                },
                AccountId {
                    id: 1,
                    origin: AccountOrigin::Remote("bitcoin".to_string()),
                },
                AccountId {
                    id: 2,
                    origin: AccountOrigin::Remote("ethereum".to_string()),
                },
            )
        }

        #[test]
        fn storage_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<&AccountId, u64> = Map::new("map");

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
            let map: Map<(&AccountId, Addr), u64> = Map::new("map");

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

        #[test]
        fn partial_key_works() {
            let mut deps = mock_dependencies();
            let (key1, key2, key3) = mock_keys();
            let map: Map<&AccountId, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key1, &42069).unwrap();

            map.save(deps.as_mut().storage, &key2, &69420).unwrap();

            map.save(deps.as_mut().storage, &key3, &999).unwrap();

            let items = map
                .prefix(1)
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 2);
            assert_eq!(
                items[0],
                (AccountOrigin::Remote("bitcoin".to_string()), 69420)
            );
            assert_eq!(items[1], (AccountOrigin::Local, 42069));
        }
    }
}
