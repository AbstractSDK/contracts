use crate::objects::pool_id::PoolId;
use crate::objects::unique_pool_id::UniquePoolId;

#[cosmwasm_schema::cw_serde]
pub struct PoolReference {
    pub id: UniquePoolId,
    pub pool_id: PoolId,
}
