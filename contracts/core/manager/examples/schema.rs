use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cw_asset::AssetInfoBase;
use std::{env::current_dir, fs::create_dir_all};

use abstract_os::core::proxy::{
    msg::{
        ConfigResponse, ExecuteMsg, HoldingAmountResponse, HoldingValueResponse, InstantiateMsg,
        QueryMsg, TotalValueResponse, VaultAssetConfigResponse,
    },
    proxy_assets::ProxyAsset,
};
use cosmwasm_std::{Addr, CosmosMsg, Empty};
use manager::state::Config;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(HoldingValueResponse), &out_dir);
    export_schema(&schema_for!(HoldingAmountResponse), &out_dir);
    export_schema(&schema_for!(TotalValueResponse), &out_dir);
    export_schema(&schema_for!(VaultAssetConfigResponse), &out_dir);
    export_schema(&schema_for!(ProxyAsset), &out_dir);
    export_schema_with_title(
        &schema_for!(CosmosMsg<Empty>),
        &out_dir,
        "CosmosMsg_for_Empty",
    );
    export_schema_with_title(
        &schema_for!(AssetInfoBase<Addr>),
        &out_dir,
        "AssetInfoBase_for_Addr",
    );
}
