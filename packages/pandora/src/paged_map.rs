#![allow(dead_code)]
use cosmwasm_std::{DepsMut, Order, StdResult};
use cw_storage_plus::{Bound, Item, Map};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub struct PagedMap<'a, T, R> {
    pub data: Map<'a, &'a [u8], T>,
    pub status: Item<'a, PaginationInfo<R>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationInfo <R> {
    pub is_locked: bool,
    pub size: u32,
    pub last_item: Option<String>,
    pub accumulator: Option<R>,
}
#[allow(unused)]
impl<'a, T, R> PagedMap<'a, T, R> {
    pub const fn new(namespace: &'a str) -> Self {
        let status_key = String::from(namespace) + "status".to_string();
        PagedMap {
            data: Map::new(namespace),
            status: Item::new(&status_key),
        }
    }

    fn page_with_accumulator(
        &self,
        deps: DepsMut,
        limit: Option<u32>,
        f: fn(T, &mut Option<R>),
    ) -> StdResult<()>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default,
    {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let mut status = self.status.load(deps.storage)?;
        if !status.is_locked {
            status.is_locked = true;
            status.accumulator = Some(R::default());
        }
        let start = status.last_item.clone().map(Bound::exclusive);

        let result: Vec<Vec<u8>> = self
            .data
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                let (key, element) = item.unwrap();
                f(element, &mut status.accumulator);
                key
            })
            .collect();

        status.last_item = result
            .last()
            .map(|key| String::from(std::str::from_utf8(key).unwrap()));

        self.status.save(
            deps.storage,
            &status
        )?;
        Ok(())
    }
}
