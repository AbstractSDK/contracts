use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Env, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::AbstractResult;

#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
/// The name of a chain, aka the chain-id without the post-fix number.
/// ex. `cosmoshub-4` -> `cosmoshub`, `juno-1` -> `juno`
pub struct ChainName(String);

impl ChainName {
    // Construct the chain name from the environment (chain-id)
    pub fn new(env: &Env) -> Self {
        let chain_id = &env.block.chain_id;
        // split on the first -
        let parts: Vec<&str> = chain_id.splitn(2, '-').collect();
        Self(parts[0].to_string())
    }

    /// check the formatting of the chain name
    pub fn check(&self) -> AbstractResult<()> {
        if self.0.contains("-") || !self.0.as_str().is_ascii() {
            return Err(crate::AbstractError::FormattingError {
                object: "chain_name".into(),
                expected: "chain_name-351".into(),
                actual: self.0.clone(),
            });
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    // used for key implementation
    pub(crate) fn str_ref(&self) -> &String {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<&str> for ChainName {
    /// unchecked conversion!
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ChainName {
    /// unchecked conversion!
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ToString for ChainName {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}


