use crate::objects::pool_id::{PoolId, PoolIdBase, UncheckedPoolId};
use cosmwasm_std::{Addr, Api, StdError, StdResult, Uint128};
use cw_asset::Asset;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolBase<T> {
    pub id: PoolIdBase<T>,
    // TODO: use use something better than string
    pub assets: String,
}

impl<T> PoolBase<T> {
    pub fn new<P: Into<PoolBase<T>>>(pool: P) -> Self {
        pool.into()
    }
    pub fn contract<A: Into<T>, B: Into<String>>(contract: A, assets: String) -> Self {
        Self {
            id: PoolIdBase::Contract(contract.into()),
            assets,
        }
    }
    pub fn id<N: Into<u64>>(id: N, assets: String) -> Self {
        Self {
            id: PoolIdBase::Id(id.into()),
            assets,
        }
    }
}

/// Actual instance of a Pool with verified data
pub type Pool = PoolBase<Addr>;
/// Instance of a Pool passed around messages
pub type UncheckedPool = PoolBase<String>;

impl FromStr for UncheckedPool {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words: Vec<&str> = s.split(':').collect();

        let id = match words[0] {
            "contract" | "id" => {
                if words.len() != 3 {
                    return Err(StdError::generic_err(
                        format!("invalid pool id format `{}`; must be in format `contract:{{contract_addr/id}}:{{assets}}`", s)
                    ));
                }
                UncheckedPoolId::from_str(words[1])?
            }
            unknown => return Err(StdError::generic_err(format!(
                "invalid pool id type `{}`; must be `contract` or `id`",
                unknown
            ))),
        };

        let assets = String::from(words[words.len() - 1]);

        Ok(UncheckedPool { id, assets })
    }
}

impl From<Pool> for UncheckedPool {
    fn from(pool: Pool) -> Self {
        UncheckedPool {
            id: pool.id.into(),
            assets: pool.assets,
        }
    }
}

impl UncheckedPool {
    /// Validate data contained in an _unchecked_ **pool id** instance; return a new _checked_
    /// **pool id** instance:
    /// * For Contract addresses, assert its address is valid
    ///
    ///
    /// ```rust
    /// use cosmwasm_std::{Addr, Api, StdResult};
    /// use abstract_os::objects::pool_info::UncheckedPool;
    ///
    /// fn validate_pool(api: &dyn Api, pool_unchecked: &UncheckedPool) {
    ///     match pool_unchecked.check(api) {
    ///         Ok(info) => println!("pool id is valid: {}", info.to_string()),
    ///         Err(err) => println!("pool id is invalid! reason: {}", err),
    ///     }
    /// }
    /// ```
    pub fn check(&self, api: &dyn Api) -> StdResult<Pool> {
        Ok(Pool {
            id: self.id.check(api)?,
            assets: self.assets.clone(),
        })
    }
}

impl fmt::Display for Pool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.id, self.assets)
    }
}
