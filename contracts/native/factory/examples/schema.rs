use std::{env::current_dir, fs::create_dir_all};

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for, write_api};

use abstract_os::{objects::gov_type::GovernanceDetails, os_factory::*};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    };

    // let mut out_dir = current_dir().unwrap();
    // out_dir.push("schema");
    // create_dir_all(&out_dir).unwrap();
    // remove_schemas(&out_dir).unwrap();

    // export_schema(&schema_for!(InstantiateMsg), &out_dir);
    // export_schema(&schema_for!(GovernanceDetails), &out_dir);
    // export_schema(&schema_for!(ExecuteMsg), &out_dir);
    // export_schema(&schema_for!(QueryMsg), &out_dir);
    // export_schema_with_title(&schema_for!(ConfigResponse), &out_dir, "ConfigResponse");
}
