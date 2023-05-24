use abstract_core::ans_host::InstantiateMsg;
use cw_orch::daemon::sync::core::Daemon;
use cw_orch::deploy::Deploy;
use std::env::set_var;

use crate::Abstract;
use cw_orch::prelude::*;

#[test]
#[serial_test::serial]
fn test_deploy_abstract() {
    set_var("TEST_MNEMONIC","extra infant liquid afraid lens legend frown horn flame vessel palm nuclear jazz build iron squeeze review stock they snake dawn metal outdoor muffin");

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut daemon = Daemon::builder()
        .chain(networks::UNI_6)
        .handle(runtime.handle())
        .build()
        .unwrap();

    let abstr = Abstract::load_from(&mut daemon).unwrap();

    // We test if the wasm file is present alright
    abstr.ans_host.wasm();
    // Now we upload abstract using the file loaded configuration

    abstr
        .ans_host
        .instantiate(&InstantiateMsg {}, None, None)
        .unwrap();
}
