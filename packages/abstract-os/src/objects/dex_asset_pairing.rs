use std::{convert::TryInto, fmt::Display};

use cosmwasm_std::{StdError, StdResult};

use cw_storage_plus::{KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

type DexName = String;

/// The key for an asset pairing
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
pub struct DexAssetPairing(String, String, DexName);

/// new impl
impl DexAssetPairing {
    pub fn new(asset_x: &str, asset_y: &str, dex_name: &str) -> Self {
        Self(
            str::to_ascii_lowercase(asset_x),
            str::to_ascii_lowercase(asset_y),
            str::to_ascii_lowercase(dex_name),
        )
    }

    pub fn asset_x(&self) -> &str {
        &self.0
    }

    pub fn asset_y(&self) -> &str {
        &self.1
    }

    pub fn dex(&self) -> &str {
        &self.2
    }
}

impl Into<(String, String, String)> for DexAssetPairing {
    fn into(self) -> (String, String, String) {
        (self.0, self.1, self.2)
    }
}

impl From<(String, String, String)> for DexAssetPairing {
    fn from((asset_x, asset_y, dex_name): (String, String, String)) -> Self {
        Self::new(&asset_x, &asset_y, &dex_name)
    }
}

impl Display for DexAssetPairing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}-{}", self.dex(), self.asset_x(), self.asset_y())
    }
}

impl<'a> PrimaryKey<'a> for DexAssetPairing {
    type Prefix = (String, String);
    type SubPrefix = String;
    type Suffix = DexName;
    type SuperSuffix = (String, DexName);

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.0.key();
        keys.extend(self.1.key());
        keys.extend(self.2.key());
        keys
    }
}

impl<'a> Prefixer<'a> for DexAssetPairing {
    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
        let mut res = self.0.prefix();
        res.extend(self.1.prefix().into_iter());
        res.extend(self.2.prefix().into_iter());
        res
    }
}

fn parse_length(value: &[u8]) -> StdResult<usize> {
    Ok(u16::from_be_bytes(
        value
            .try_into()
            .map_err(|_| StdError::generic_err("Could not read 2 byte length"))?,
    )
    .into())
}

impl KeyDeserialize for DexAssetPairing {
    type Output = DexAssetPairing;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tuv = value.split_off(2);
        let t_len = parse_length(&value)?;
        let mut len_uv = tuv.split_off(t_len);

        let mut uv = len_uv.split_off(2);
        let u_len = parse_length(&len_uv)?;
        let v = uv.split_off(u_len);

        Ok((
            String::from_vec(tuv)?,
            String::from_vec(uv)?,
            String::from_vec(v)?,
        )
            .into())
    }
}
