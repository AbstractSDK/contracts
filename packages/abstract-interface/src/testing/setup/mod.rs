use std::{env, fs, path::Path};

use ctor::{ctor, dtor};


// Config
const JUNO_IMAGE: &str = "ghcr.io/cosmoscontracts/juno:v12.0.0";

// Defaults for env vars
const CONTAINER_NAME: &str = "juno_node_1";
const LOCAL_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";

use uid::Id as IdT;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DeployId(());

#[allow(unused)]
pub type Id = IdT<DeployId>;

pub mod state_file {
    use super::{fs, Path};

    pub fn exists(file: &str) -> bool {
        if Path::new(file).exists() {
            log::info!("File found: {}", file);
            true
        } else {
            log::info!("File not found: {}", file);
            false
        }
    }

    pub fn remove(file: &str) {
        if self::exists(file) {
            log::info!("Removing state file: {}", file);
            let _ = fs::remove_file(file);
        }
    }
}

pub fn test_env_start() {

    // Set environment variables
    // this does not seems to be working in this case
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug");
    }

    if env::var("CONTAINER_NAME").is_err() {
        env::set_var("CONTAINER_NAME", CONTAINER_NAME);
    }
    let container = env::var("CONTAINER_NAME").unwrap();

    if env::var("JUNO_IMAGE").is_err() {
        env::set_var("JUNO_IMAGE", JUNO_IMAGE);
    }

    let temp_dir = env::temp_dir();
    let state_file = temp_dir.join("cw_orch_test.json");

    if env::var("STATE_FILE").is_err() {
        env::set_var("STATE_FILE", state_file);
    }

    if env::var("LOCAL_MNEMONIC").is_err() {
        env::set_var("LOCAL_MNEMONIC", LOCAL_MNEMONIC);
    }

    log::info!("Using RUST_LOG: {}", env::var("RUST_LOG").unwrap());
    log::info!("Using CONTAINER_NAME: {}", container);
    log::info!("Using STATE_FILE: {}", env::var("STATE_FILE").unwrap());
    log::info!(
        "Using LOCAL_MNEMONIC: {}",
        env::var("LOCAL_MNEMONIC").unwrap()
    );
}
pub fn test_env_stop() {
    let temp_dir = env::temp_dir();
    let expected_state_file = temp_dir.join("cw_orch_test_local.json");
    state_file::remove(expected_state_file.to_str().unwrap());
}

#[ctor]
fn common_start() {
    env_logger::init();
    test_env_start()
}

#[dtor]
fn common_stop() {
    test_env_stop()
}
