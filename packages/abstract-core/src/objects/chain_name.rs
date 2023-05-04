use cosmwasm_schema::cw_serde;
use cosmwasm_std::Env;

#[cw_serde]
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

    pub fn as_str(&self) -> &str {
        self.0.as_str()
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
