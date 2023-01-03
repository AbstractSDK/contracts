use crate::objects::pool_id::{PoolId, UncheckedPoolId};
use crate::objects::unique_pool_id::UniquePoolId;
use cosmwasm_std::{Api, StdResult};

#[cosmwasm_schema::cw_serde]
pub struct PoolReference {
    pub unique_id: UniquePoolId,
    pub pool_id: PoolId,
}

impl PoolReference {
    pub fn new(unique_id: UniquePoolId, pool_id: PoolId) -> Self {
        Self { unique_id, pool_id }
    }
}

/// Pool referenced with an unchecked pool ID
pub struct UncheckedPoolReference {
    pub unique_id: u64,
    pub pool_id: UncheckedPoolId,
}

impl UncheckedPoolReference {
    pub fn new(unique_id: u64, pool_id: UncheckedPoolId) -> Self {
        Self { unique_id, pool_id }
    }

    pub fn check(&self, api: &dyn Api) -> StdResult<PoolReference> {
        let checked_pool_id = self.pool_id.check(api)?;

        Ok(PoolReference::new(
            UniquePoolId::new(self.unique_id),
            checked_pool_id,
        ))
    }
}
