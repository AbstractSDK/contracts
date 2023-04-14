use cosmwasm_std::{StdResult, CosmosMsg, StdError, Empty};
use serde::Serialize;

// trait for converting CosmosMsg<T> to CosmosMsg<Value>
pub trait CustomCosmosMsg {
    type Out;
    type OutEmpty;

    fn into_value(self) -> StdResult<Self::Out>;
    fn into_empty(self) -> StdResult<Self::OutEmpty>;
}

impl<T, G> CustomCosmosMsg for G
where
    T: Serialize,
    G: IntoIterator<Item = CosmosMsg<T>>,
{
    type Out = Vec<CosmosMsg<serde_cw_value::Value>>;
    type OutEmpty = Vec<CosmosMsg>;

    fn into_value(self) -> StdResult<Vec<CosmosMsg<serde_cw_value::Value>>> {
        self.into_iter()
            .map(into_value)
            .collect::<StdResult<Vec<_>>>()
    }
    
    fn into_empty(self) -> StdResult<Vec<CosmosMsg>> {
        self.into_iter()
            .map(into_empty)
            .collect::<StdResult<Vec<_>>>()
    }
}

/// Convert the cosmos custom message into the [`serde_cw_value::Value`] type.
#[inline(always)]
pub fn into_value<T>(msg: CosmosMsg<T>) -> StdResult<CosmosMsg<serde_cw_value::Value>>
where
    T: Serialize,
{
    Ok(match msg {
        CosmosMsg::Custom(custom) => CosmosMsg::Custom(
            serde_cw_value::to_value(custom)
                .map_err(|e| StdError::generic_err(e.to_string()))?,
        ),
        CosmosMsg::Wasm(wasm) => CosmosMsg::Wasm(wasm),
        CosmosMsg::Bank(bank) => CosmosMsg::Bank(bank),
        CosmosMsg::Staking(staking) => CosmosMsg::Staking(staking),
        CosmosMsg::Distribution(distribution) => CosmosMsg::Distribution(distribution),
        CosmosMsg::Stargate { type_url, value } => CosmosMsg::Stargate { type_url, value },
        CosmosMsg::Ibc(ibc) => CosmosMsg::Ibc(ibc),
        CosmosMsg::Gov(gov) => CosmosMsg::Gov(gov),
        _ => return Err(StdError::generic_err("Unsupported CosmosMsg")),
    })
}

/// Convert the cosmos custom message into the [`cosmwasm_std::Empty`] type
#[inline(always)]
pub fn into_empty<T>(msg: CosmosMsg<T>) -> StdResult<CosmosMsg<Empty>>
where
    T: Serialize,
{
    Ok(match msg {
        CosmosMsg::Wasm(wasm) => CosmosMsg::Wasm(wasm),
        CosmosMsg::Bank(bank) => CosmosMsg::Bank(bank),
        CosmosMsg::Staking(staking) => CosmosMsg::Staking(staking),
        CosmosMsg::Distribution(distribution) => CosmosMsg::Distribution(distribution),
        CosmosMsg::Stargate { type_url, value } => CosmosMsg::Stargate { type_url, value },
        CosmosMsg::Ibc(ibc) => CosmosMsg::Ibc(ibc),
        CosmosMsg::Gov(gov) => CosmosMsg::Gov(gov),
        CosmosMsg::Custom(..) => return Err(StdError::generic_err("cannot convert custom message".to_string())),
        _ => return Err(StdError::generic_err("Unsupported CosmosMsg")),
    })
}