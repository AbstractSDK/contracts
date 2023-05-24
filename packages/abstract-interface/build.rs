use serde_json::from_reader;
use std::fs::File;
use std::env;
use std::fs;
use std::path::Path;

fn main() {

    dotenv::dotenv().unwrap();
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("add_custom_state.rs");

    let origin_state_path = env::var("STATE_FILE").unwrap();

    // First we load the daemon json file
    if cfg!(feature = "build-script") {
        // We copy the daemon_file defined in .env to the out directory and use that as custom state
        let state_file =
            File::open(origin_state_path.clone()).unwrap_or_else(|_| panic!("File should be present at {}", origin_state_path));
        let json: serde_json::Value = from_reader(state_file).unwrap();
        // Now, we output the json file so that it can be used in the daemon state. We want this load to be non-null when exporting the package
         fs::write(
            dest_path,
            format!(
            "
            use cw_orch::prelude::CwEnv;
            pub fn custom_state<T: CwEnv>(chain: &mut T){{
                chain.custom_state(serde_json::json!({}))
            }}", json)
        ).unwrap();

        // Add your build script logic here
    } else {
        // In the general case, we don't want to introduce a custom state
        fs::write(
            dest_path,
            "
            use cw_orch::prelude::CwEnv;
            pub fn custom_state<T: CwEnv>(_chain: &mut T){
            }"
        ).unwrap();
    }

   


    // However, we cant to introduce a custom state when exporting this crate 


    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}",origin_state_path);
}