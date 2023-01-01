use std::env::current_dir;
use std::fs::create_dir_all;

use abstract_os::tendermint_staking::{TendermintStakingExecuteMsg, TendermintStakingQueryMsg};
use cosmwasm_schema::{remove_schemas, write_api};

use tendermint_staking::contract::TendermintStakeApi;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    // Write a modified entry point schema for the Tendermint staking API
    write_api! {
        name: "module-schema",
        query: TendermintStakingQueryMsg,
        execute: TendermintStakingExecuteMsg,
        instantiate: Empty,
        migrate: Empty,
    };

    TendermintStakeApi::export_schema(&out_dir);
}
