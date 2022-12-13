use std::fmt::Debug;

use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{DepsMut, MessageInfo, Response};
use cosmwasm_std::{Env, Storage};
use cosmwasm_std::{Order, OwnedDeps};

use cw_storage_plus::{KeyDeserialize, Map, PrimaryKey};
use derive_builder::Builder;
use serde::de::DeserializeOwned;
use serde::Serialize;

type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct CwMapTester<'a, ExecMsg, TError, K, V, UncheckedK, UncheckedV>
where
    V: Serialize + DeserializeOwned + Clone + Debug,
    K: PrimaryKey<'a> + KeyDeserialize + Debug,
    (<K as KeyDeserialize>::Output, V): PartialEq<(K, V)>,
    K::Output: 'static,
    // UncheckedK: From<<K as KeyDeserialize>::Output> + Clone + PartialEq + Debug + Ord,
    // UncheckedV: From<V> + Clone + PartialEq + Debug + Ord,
    UncheckedK: Clone + PartialEq + Debug + Ord,
    UncheckedV: Clone + PartialEq + Debug,
    <K as KeyDeserialize>::Output: Debug,
{
    info: MessageInfo,
    map: Map<'a, K, V>,
    execute:
        fn(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecMsg) -> Result<Response, TError>,
    msg_builder: fn(to_add: Vec<(UncheckedK, UncheckedV)>, to_remove: Vec<UncheckedK>) -> ExecMsg,
    mock_entry_builder: fn() -> (UncheckedK, UncheckedV),
    from_checked_entry: fn((K::Output, V)) -> (UncheckedK, UncheckedV),
}

impl<'a, ExecMsg, TError, K, V, UncheckedK, UncheckedV>
    CwMapTester<'a, ExecMsg, TError, K, V, UncheckedK, UncheckedV>
where
    V: Serialize + DeserializeOwned + Clone + Debug,
    K: PrimaryKey<'a> + KeyDeserialize + Debug,
    (<K as KeyDeserialize>::Output, V): PartialEq<(K, V)>,
    K::Output: 'static,
    UncheckedK: Clone + PartialEq + Debug + Ord,
    UncheckedV: Clone + PartialEq + Debug,
    // UncheckedK: From<<K as KeyDeserialize>::Output> + Clone + PartialEq + Debug + Ord,
    // UncheckedV: From<V> + Clone + PartialEq + Debug + Ord,
    <K as KeyDeserialize>::Output: Debug,
{
    pub fn new(
        info: MessageInfo,
        map: Map<'a, K, V>,
        execute: fn(
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: ExecMsg,
        ) -> Result<Response, TError>,
        msg_builder: fn(
            to_add: Vec<(UncheckedK, UncheckedV)>,
            to_remove: Vec<UncheckedK>,
        ) -> ExecMsg,
        mock_entry_builder: fn() -> (UncheckedK, UncheckedV),
        from_checked_entry: fn((K::Output, V)) -> (UncheckedK, UncheckedV),
    ) -> Self {
        Self {
            info,
            map,
            execute,
            msg_builder,
            mock_entry_builder,
            from_checked_entry,
        }
    }

    pub fn msg_builder(
        &self,
        to_add: Vec<(UncheckedK, UncheckedV)>,
        to_remove: Vec<UncheckedK>,
    ) -> ExecMsg {
        (self.msg_builder)(to_add, to_remove)
    }

    fn mock_entry_builder(&self) -> (UncheckedK, UncheckedV) {
        (self.mock_entry_builder)()
    }

    /// Execute the msg with the mock env
    pub fn execute(&mut self, deps: DepsMut, msg: ExecMsg) -> Result<(), TError> {
        (self.execute)(deps, mock_env(), self.info.clone(), msg)?;
        Ok(())
    }

    pub fn execute_update(
        &mut self,
        deps: DepsMut,
        (to_add, to_remove): (Vec<(UncheckedK, UncheckedV)>, Vec<UncheckedK>),
    ) -> Result<(), TError> {
        let msg = self.msg_builder(to_add, to_remove);
        self.execute(deps, msg)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_checked_entry(&self, entry: (K::Output, V)) -> (UncheckedK, UncheckedV) {
        (self.from_checked_entry)(entry)
    }

    /// Sort the expected entries by *key*
    fn sort_expected(expected: &mut [(UncheckedK, UncheckedV)]) {
        expected.sort_by(|a, b| a.0.cmp(&b.0));
    }

    #[allow(clippy::ptr_arg)]
    pub fn determine_expected(
        &self,
        to_add: &Vec<(UncheckedK, UncheckedV)>,
        to_remove: &[UncheckedK],
    ) -> Vec<(UncheckedK, UncheckedV)> {
        let mut expected = to_add.clone();
        expected.retain(|(k, _)| !to_remove.contains(k));
        Self::sort_expected(&mut expected);
        expected.dedup();
        expected
    }

    pub fn assert_expected_entries(
        &self,
        storage: &'_ dyn Storage,
        expected: Vec<(UncheckedK, UncheckedV)>,
    ) {
        let res: Result<Vec<(K::Output, V)>, _> = self
            .map
            .range(storage, None, None, Order::Ascending)
            .collect();

        let actual = res
            .unwrap()
            .into_iter()
            .map(|(k, v)| self.from_checked_entry((k, v)))
            .collect::<Vec<_>>();

        // Sort, like map entries
        let mut expected = expected;
        Self::sort_expected(&mut expected);

        assert_eq!(actual, expected)
    }

    pub fn test_add_one(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        let entry = self.mock_entry_builder();

        let to_add: Vec<(UncheckedK, UncheckedV)> = vec![entry];
        let to_remove: Vec<UncheckedK> = vec![];
        let msg = self.msg_builder(to_add.clone(), to_remove.clone());

        let expected = self.determine_expected(&to_add, &to_remove);

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    pub fn test_add_one_twice(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        self.test_add_one(deps)?;
        self.test_add_one(deps)
    }

    pub fn test_add_two_same(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        let entry = self.mock_entry_builder();

        let to_add: Vec<(UncheckedK, UncheckedV)> = vec![entry.clone(), entry];
        let to_remove: Vec<UncheckedK> = vec![];
        let msg = self.msg_builder(to_add.clone(), to_remove.clone());

        let expected: Vec<(UncheckedK, UncheckedV)> = self.determine_expected(&to_add, &to_remove);

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    pub fn test_add_and_remove_same(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        let entry = self.mock_entry_builder();

        let to_add: Vec<(UncheckedK, UncheckedV)> = vec![entry.clone()];
        let to_remove: Vec<UncheckedK> = vec![entry.0];
        let msg = self.msg_builder(to_add, to_remove);

        let expected: Vec<(UncheckedK, UncheckedV)> = vec![];

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    pub fn test_remove_nonexistent(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        let entry = self.mock_entry_builder();

        let to_add: Vec<(UncheckedK, UncheckedV)> = vec![];
        let to_remove: Vec<UncheckedK> = vec![entry.0];
        let msg = self.msg_builder(to_add, to_remove);

        let expected: Vec<(UncheckedK, UncheckedV)> = vec![];

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    /// Test the manually provided arguments with the expected behavior, which is removing any duplicate entries that are within both add and remove
    pub fn test_update_auto_expect(
        &mut self,
        deps: &mut MockDeps,
        update: (Vec<(UncheckedK, UncheckedV)>, Vec<UncheckedK>),
    ) -> Result<(), TError> {
        let (to_add, to_remove) = update;
        let msg = self.msg_builder(to_add.clone(), to_remove.clone());

        let expected: Vec<(UncheckedK, UncheckedV)> = self.determine_expected(&to_add, &to_remove);

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    /// Provide an update nad expected result, and test that the expected result is returned
    pub fn test_update_with_expected(
        &mut self,
        deps: &mut MockDeps,
        update: (Vec<(UncheckedK, UncheckedV)>, Vec<UncheckedK>),
        expected: Vec<(UncheckedK, UncheckedV)>,
    ) -> Result<(), TError> {
        let (to_add, to_remove) = update;
        let msg = self.msg_builder(to_add, to_remove);

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }
}
