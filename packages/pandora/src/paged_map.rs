#![allow(dead_code)]
use cosmwasm_std::{DepsMut, Order, StdResult};
use cw_storage_plus::{Bound, Item, Map};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub struct PagedMap<'a, T> {
    pub data: Map<'a, &'a [u8], T>,
    pub status: Item<'a, PaginationInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub is_locked: bool,
    pub size: u32,
    pub last_item: Option<String>,
}
#[allow(unused)]
impl<'a, T> PagedMap<'a, T> {
    pub const fn new(storage_key: &'a str, namespace: &'a str) -> Self {
        PagedMap {
            data: Map::new(namespace),
            status: Item::new(storage_key),
        }
    }

    fn page_with_accumulator<R>(
        &self,
        deps: DepsMut,
        limit: Option<u32>,
        f: fn(T, &mut R),
        mut accumulator: R,
    ) -> StdResult<()>
    where
        T: Serialize + DeserializeOwned,
    {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let status = self.status.load(deps.storage)?;

        let start = status.last_item.map(Bound::exclusive);
        let result: Vec<Vec<u8>> = self
            .data
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                let (key, element) = item.unwrap();
                f(element, &mut accumulator);
                key
            })
            .collect();

        let last_item: Option<String> = result
            .last()
            .map(|key| String::from(std::str::from_utf8(key).unwrap()));

        self.status.save(
            deps.storage,
            &PaginationInfo {
                last_item,
                ..status
            },
        )?;
        Ok(())
    }
}
