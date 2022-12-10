use crate::objects::pool_type::PoolType;
use cosmwasm_std::StdError;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

type DexName = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PoolMetadata {
    pub dex: DexName,
    pub pool_type: PoolType,
    pub assets: Vec<String>,
}

const ATTRIBUTE_COUNT: usize = 3;
const ATTTRIBUTE_SEPARATOR: &str = ":";
const ASSET_SEPARATOR: &str = "_";

impl FromStr for PoolMetadata {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let attributes: Vec<&str> = s.split(ATTTRIBUTE_SEPARATOR).collect();

        if attributes.len() != ATTRIBUTE_COUNT {
            return Err(StdError::generic_err(format!(
                "invalid pool metadata format `{}`; must be in format `{{dex}}:{{pool_type}}:{{asset1}}_{{asset2}}_...`",
                s
            )));
        }

        let dex = String::from(attributes[0]);
        let pool_type = PoolType::from_str(attributes[1])?;
        let assets = String::from(attributes[2])
            .split(ASSET_SEPARATOR)
            .map(String::from)
            .collect();

        Ok(PoolMetadata {
            dex,
            pool_type,
            assets,
        })
    }
}

/// To string
/// Ex: "junoswap:stable:uusd,uust"
impl fmt::Display for PoolMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let assets_str = self.assets.join(ASSET_SEPARATOR);
        let pool_type_str = self.pool_type.to_string();

        write!(
            f,
            "{}",
            vec![self.dex.clone(), pool_type_str, assets_str].join(ATTTRIBUTE_SEPARATOR)
        )
    }
}
