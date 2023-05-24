use std::env;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
const CRATE_PATH: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("add_custom_state.rs");

    let state_path = "daemon_state.json".to_string(); // This is where the custom state comes from, not possible to change that for now

    // First we load the daemon json file
    // We verify that the daemon_file is actually present wher it should be located
    File::open(state_path.clone())
        .unwrap_or_else(|_| panic!("File should be present at {}", state_path));
    // Now, we output the json file so that it can be used in the daemon state. We want this load to be non-null when exporting the package

    // This will be loaded from scripts out of the manifest dir
    let absolute_state_path = PathBuf::from(CRATE_PATH).join(state_path);
    fs::write(
        dest_path,
        format!(
            "
        use cw_orch::prelude::CwEnv;
        pub fn custom_state<T: CwEnv>(chain: &mut T){{
            chain.custom_state_file(\"{}\".to_string())
        }}",
            absolute_state_path.display()
        ),
    )
    .unwrap();

    // We also verify that the local artifacts fir exists
    assert!(std::fs::metadata("./artifacts").is_ok(), "You should create an artifacts dir in your crate to export the wasm files along with the cw-orch library");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", absolute_state_path.display());
}
