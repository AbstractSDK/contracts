/// The delimiter between assets in lists
pub const ASSET_DELIMITER: &str = ",";
/// The delimited between types like contract_type/asset1,asset2
pub const TYPE_DELIMITER: &str = "/";
/// The delimiter between attributes like contract:protocol
pub const ATTRIBUTE_DELIMITER: &str = ":";

// chain-id prefixes based on `https://cosmos.directory/`
pub const JUNO: &[&str] = &["juno", "uni"];
pub const OSMOSIS: &[&str] = &["osmosis", "osmo"];
pub const TERRA: &[&str] = &["phoenix", "pisco"];
pub const KUJIRA: &[&str] = &["kaiyo", "harpoon"];
pub const ARCHWAY: &[&str] = &["constantine"];
