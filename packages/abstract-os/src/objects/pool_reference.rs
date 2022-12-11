use crate::ans_host::UniquePoolId;
use crate::objects::pool_id::PoolId;

#[cosmwasm_schema::cw_serde]
pub struct PoolReference {
    pub id: UniquePoolId,
    pub pool_id: PoolId,
}
