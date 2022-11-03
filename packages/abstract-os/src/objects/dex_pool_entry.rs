use std::{convert::TryInto, fmt::Display};

use cosmwasm_std::{StdError, StdResult};

use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Key to get the Address of a dex
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
pub struct UncheckedDexPoolEntry {
    pub dex: String,
    pub asset_pair: String,
}

impl UncheckedDexPoolEntry {
    pub fn new<T: ToString>(dex: T, asset_pair: T) -> Self {
        Self {
            dex: dex.to_string(),
            asset_pair: asset_pair.to_string(),
        }
    }
    pub fn check(self) -> DexPoolEntry {
        DexPoolEntry {
            dex: self.dex.to_ascii_lowercase(),
            asset_pair: self.asset_pair.to_ascii_lowercase(),
        }
    }
}

impl TryFrom<String> for UncheckedDexPoolEntry {
    type Error = StdError;
    fn try_from(entry: String) -> Result<Self, Self::Error> {
        let composite: Vec<&str> = entry.split('/').collect();
        if composite.len() != 2 {
            return Err(StdError::generic_err(
                "dex entry should be formatted as \"dex/asset_pair\".",
            ));
        }
        Ok(Self::new(composite[0], composite[1]))
    }
}

/// Key to get the Address of a dex
/// Use [`UncheckedDexPoolEntry`] to construct this type.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
pub struct DexPoolEntry {
    pub dex: String,
    pub asset_pair: String,
}

impl Display for DexPoolEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.dex, self.asset_pair)
    }
}

impl<'a> PrimaryKey<'a> for DexPoolEntry {
    type Prefix = String;

    type SubPrefix = ();

    type Suffix = String;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.dex.key();
        keys.extend(self.asset_pair.key());
        keys
    }
}

impl<'a> Prefixer<'a> for DexPoolEntry {
    fn prefix(&self) -> Vec<Key> {
        let mut res = self.dex.prefix();
        res.extend(self.asset_pair.prefix().into_iter());
        res
    }
}

impl KeyDeserialize for DexPoolEntry {
    type Output = Self;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tu = value.split_off(2);
        let t_len = parse_length(&value)?;
        let u = tu.split_off(t_len);

        Ok(Self {
            dex: String::from_vec(tu)?,
            asset_pair: String::from_vec(u)?,
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

    fn mock_key() -> DexPoolEntry {
        DexPoolEntry {
            dex: "osmosis".to_string(),
            asset_pair: "ics20".to_string(),
        }
    }

    fn mock_keys() -> (DexPoolEntry, DexPoolEntry, DexPoolEntry) {
        (
            DexPoolEntry {
                dex: "osmosis".to_string(),
                asset_pair: "ics20".to_string(),
            },
            DexPoolEntry {
                dex: "osmosis".to_string(),
                asset_pair: "ics".to_string(),
            },
            DexPoolEntry {
                dex: "cosmos".to_string(),
                asset_pair: "abstract".to_string(),
            },
        )
    }

    #[test]
    fn storage_key_works() {
        let mut deps = mock_dependencies();
        let key = mock_key();
        let map: Map<DexPoolEntry, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key.clone(), &42069)
            .unwrap();

        assert_eq!(map.load(deps.as_ref().storage, key.clone()).unwrap(), 42069);

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
        let map: Map<(DexPoolEntry, Addr), u64> = Map::new("map");

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("larry")),
            &42069,
        )
        .unwrap();

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("jake")),
            &69420,
        )
        .unwrap();

        let items = map
            .prefix(key)
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
        let map: Map<DexPoolEntry, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key1, &42069).unwrap();

        map.save(deps.as_mut().storage, key2, &69420).unwrap();

        map.save(deps.as_mut().storage, key3, &999).unwrap();

        let items = map
            .prefix("osmosis".to_string())
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ("ics".to_string(), 69420));
        assert_eq!(items[1], ("ics20".to_string(), 42069));
    }
}
